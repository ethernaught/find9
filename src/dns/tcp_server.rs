use std::collections::HashMap;
use std::{io, thread};
use std::io::{Read, Write};
use std::net::{Ipv4Addr, Shutdown, SocketAddr, TcpListener, TcpStream, UdpSocket};
use std::sync::{Arc, RwLock};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Sender, TryRecvError};
use std::thread::{sleep, JoinHandle};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use rlibdns::messages::inter::response_codes::ResponseCodes;
use rlibdns::messages::inter::rr_types::RRTypes;
use rlibdns::messages::message_base::MessageBase;
use crate::dns::dns::QueryMap;
use crate::rpc::events::inter::event::Event;
use crate::rpc::events::query_event::QueryEvent;
use crate::utils::spam_throttle::SpamThrottle;

pub struct TcpServer {
    running: Arc<AtomicBool>,
    pub(crate) socket: Option<TcpListener>,
    tx_sender_pool: Option<Sender<(TcpStream, SocketAddr)>>,
    query_mapping: QueryMap
}

impl TcpServer {
    
    pub fn new() -> Self {
        Self {
            running: Arc::new(AtomicBool::new(false)),
            socket: None,
            tx_sender_pool: None,
            query_mapping: Arc::new(RwLock::new(HashMap::new()))
        }
    }

    pub fn run(&mut self, port: u16) -> io::Result<JoinHandle<()>> {
        if self.is_running() {
            return Err(io::Error::new(io::ErrorKind::Unsupported, "Dns is already running"));
        }

        self.socket = Some(TcpListener::bind(SocketAddr::from((Ipv4Addr::UNSPECIFIED, port)))?);
        self.socket.as_ref().unwrap().set_nonblocking(true)?;

        let (tx_sender_pool, rx_sender_pool) = channel();
        self.tx_sender_pool = Some(tx_sender_pool);

        let on_receive = self.on_receive();

        self.running.store(true, Ordering::Relaxed);

        Ok(thread::spawn({
            let socket = self.socket.as_ref().unwrap().try_clone()?;
            let running = Arc::clone(&self.running);
            let throttle = SpamThrottle::new();

            move || {
                let mut last_decay_time = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("Time went backwards")
                    .as_millis();

                while running.load(Ordering::Relaxed) {
                    match socket.accept() {
                        Ok((stream, src_addr)) => {
                            if !throttle.add_and_test(src_addr.ip()) {
                                on_receive(stream, src_addr);
                            }
                        }
                        Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {}
                        _ => break
                    }

                    let now = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .expect("Time went backwards")
                        .as_millis();

                    if now - last_decay_time >= 1000 {
                        throttle.decay();
                        last_decay_time = now;
                    }

                    sleep(Duration::from_millis(1));
                }
            }
        }))
    }

    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }

    pub fn kill(&self) {
        self.running.store(false, Ordering::Relaxed);
    }

    fn on_receive(&self) -> impl Fn(TcpStream, SocketAddr) {
        let query_mapping = self.query_mapping.clone();

        move |mut stream, src_addr| {


            println!("{src_addr}");

            let mut len_buf = [0u8; 2];
            stream.read(&mut len_buf).unwrap();

            let mut buf = vec![0x8; u16::from_be_bytes(len_buf) as usize];

            let len = stream.read(&mut buf).unwrap();

            println!("{:x?}", &buf[..len]);


            match MessageBase::from_bytes(&buf[..len]) {
                Ok(mut message) => {
                    message.set_origin(src_addr);

                    if message.is_qr() {
                        return;
                    }

                    let mut response = MessageBase::new(message.get_id());
                    response.set_op_code(message.get_op_code());
                    response.set_qr(true);
                    response.set_authoritative(true);
                    //response.set_origin(message.get_destination().unwrap());
                    response.set_destination(message.get_origin().unwrap());

                    //let mut query_event = QueryEvent::new(message);
                    //query_event.set_response(response);

                    //let is_bogon = is_bogon(message.get_origin().unwrap());

                    for query in message.get_queries() {
                        if let Some(callbacks) = query_mapping.read().unwrap().get(&query.get_type()) {
                            let mut query_event = QueryEvent::new(query);

                            for callback in callbacks {
                                callback(&mut query_event);
                            }

                            if query_event.is_prevent_default() {
                                continue;
                            }

                            response.add_query(query_event.get_query().clone());

                            if query_event.has_answers() {
                                for (query, answers) in query_event.get_answers_mut().drain() {
                                    for answer in answers {
                                        response.add_answer(&query, answer);
                                    }
                                }
                            }
                        }
                    }

                    if !response.has_answers() {
                        //DOES DOMAIN EXIST FOR US...? - IF SO ADD AUTHORITY RESPONSE SOA

                        //return;
                    }

                    //IF NAME DOES EXIST BUT NO DATA RETURN
                    // NoError

                    //If QName does not exist
                    // NxDomain

                    if !response.has_name_servers() &&
                        !response.has_additional_records() {
                        response.set_response_code(ResponseCodes::NxDomain);
                    }

                    let buf = message.to_bytes();
                    stream.write(&(buf.len() as u16).to_be_bytes()).unwrap();
                    stream.write(&buf).unwrap();
                    stream.flush().unwrap();
                }
                Err(_) => {}
            }

            stream.shutdown(Shutdown::Both).unwrap();
        }
    }

    /*
    fn send_message(&self, sender_throttle: &SpamThrottle) -> impl Fn(&MessageBase) -> io::Result<()> {
        let tx = self.tx_sender_pool.as_ref().unwrap().clone();
        let sender_throttle = sender_throttle.clone();

        move |message| {
            if message.get_destination().is_none() {
                return Err(io::Error::new(io::ErrorKind::InvalidData, "Message destination set to null"));
            }

            if !sender_throttle.add_and_test(message.get_destination().unwrap().ip()) {
                //tx.send((message.to_bytes(), message.get_destination().unwrap())).unwrap();
            }

            Ok(())
        }
    }
    */

    pub fn register_query_listener<F>(&self, key: RRTypes, callback: F)
    where
        F: Fn(&mut QueryEvent) -> io::Result<()> + Send + Sync + 'static
    {
        if self.query_mapping.read().unwrap().contains_key(&key) {
            self.query_mapping.write().unwrap().get_mut(&key).unwrap().push(Box::new(callback));
            return;
        }
        self.query_mapping.write().unwrap().insert(key, vec![Box::new(callback)]);
    }

    pub fn get_socket(&self) -> Option<&TcpListener> {
        self.socket.as_ref()
    }
}
