use std::collections::HashMap;
use std::{io, thread};
use std::io::{Read, Write};
use std::net::{IpAddr, Ipv4Addr, Shutdown, SocketAddr, TcpListener, TcpStream};
use std::sync::{Arc, RwLock};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::JoinHandle;
use std::time::{SystemTime, UNIX_EPOCH};
use rlibdns::messages::inter::response_codes::ResponseCodes;
use rlibdns::messages::inter::rr_types::RRTypes;
use rlibdns::messages::message_base::MessageBase;
use crate::dns::dns::{QueryMap, ResponseResult};
use crate::{COOKIE_SECRET, MAX_QUERIES};
use crate::dns::server::Server;
use crate::rpc::events::inter::event::Event;
use crate::rpc::events::query_event::QueryEvent;
use crate::utils::spam_throttle::SpamThrottle;

pub const MAX_TCP_MESSAGE_SIZE: usize = 65535;

pub struct TcpServer {
    running: Arc<AtomicBool>,
    pub(crate) socket: Option<TcpListener>,
    query_mapping: QueryMap
}

impl TcpServer {
    
    pub fn new() -> Self {
        Self {
            running: Arc::new(AtomicBool::new(false)),
            socket: None,
            query_mapping: Arc::new(RwLock::new(HashMap::new()))
        }
    }

    fn on_receive(&self) -> impl Fn(TcpStream, SocketAddr) {
        let query_mapping = self.query_mapping.clone();

        move |mut stream, src_addr| {
            let mut len_buf = [0u8; 2];
            stream.read_exact(&mut len_buf).unwrap();

            let mut buf = vec![0x8; u16::from_be_bytes(len_buf) as usize];
            stream.read_exact(&mut buf).unwrap();

            match MessageBase::from_bytes(&buf) {
                Ok(mut message) => {
                    message.set_origin(src_addr);

                    if message.is_qr() {
                        return;
                    }

                    let mut response = MessageBase::new(message.get_id());
                    response.set_op_code(message.get_op_code());
                    response.set_qr(true);
                    response.set_recursion_desired(message.is_recursion_desired());
                    response.set_recursion_available(false);
                    //response.set_origin(message.get_destination().unwrap());
                    response.set_destination(message.get_origin().unwrap());

                    //let mut query_event = QueryEvent::new(message);
                    //query_event.set_response(response);

                    //let is_bogon = is_bogon(message.get_origin().unwrap());

                    if !message.has_queries() {
                        response.set_response_code(ResponseCodes::FormErr);
                        let buf = response.to_bytes(MAX_TCP_MESSAGE_SIZE);
                        stream.write(&(buf.len() as u16).to_be_bytes()).unwrap();
                        stream.write(&buf).unwrap();
                        stream.flush().unwrap();
                        return;
                    }

                    for (i, query) in message.get_queries_mut().drain(..).enumerate() {
                        if i >= MAX_QUERIES {
                            break;
                        }

                        if let Some(callback) = query_mapping.read().unwrap().get(&query.get_type()) {
                            let mut event = QueryEvent::new(query.clone());

                            match callback(&mut event) {
                                Ok(_) => {
                                    if event.is_prevent_default() {
                                        return;
                                    }

                                    response.add_query(query);
                                    response.set_authoritative(event.is_authoritative());

                                    if event.has_answers() {
                                        for (query, record) in event.get_answers_mut().drain(..) {
                                            response.add_answer(&query, record);
                                        }
                                    }

                                    if event.has_authority_records() {
                                        for (query, record) in event.get_authority_records_mut().drain(..) {
                                            response.add_authority_record(&query, record);
                                        }
                                    }

                                    if event.has_additional_records() {
                                        for (query, record) in event.get_additional_records_mut().drain(..) {
                                            response.add_additional_record(&query, record);
                                        }
                                    }
                                }
                                Err(e) => {
                                    response.set_response_code(e);
                                    response.add_query(query);
                                    response.set_authoritative(event.is_authoritative());

                                    if event.has_authority_records() {
                                        for (query, record) in event.get_authority_records_mut().drain(..) {
                                            response.add_authority_record(&query, record);
                                        }
                                    }

                                    if event.has_additional_records() {
                                        for (query, record) in event.get_additional_records_mut().drain(..) {
                                            response.add_additional_record(&query, record);
                                        }
                                    }

                                    break;
                                }
                            }
                        }
                    }
/*
                    if message.has_additional_records() {
                        for (query, mut records) in message.get_additional_records_mut().drain() {
                            if query.is_empty() && !records.is_empty() {
                                println!("EDNS");

                                let mut record = records.get_mut(0).unwrap();
                                if !record.get_type().eq(&RRTypes::Opt) {
                                    continue;
                                }

                                let record = record.as_any().downcast_ref::<OptRecord>().unwrap();

                                if let Some(cookie) = record.get_option(&OptCodes::Cookie) {
                                    match cookie.len() {
                                        8 => { //CLIENT ONLY
                                            let hmac = match src_addr.ip() {
                                                IpAddr::V4(addr) => hmac::<Sha256>(COOKIE_SECRET, &[cookie, addr.octets().as_slice()].concat()),
                                                IpAddr::V6(addr) => hmac::<Sha256>(COOKIE_SECRET, &[cookie, addr.octets().as_slice()].concat())
                                            };

                                            let mut c = vec![0u8; 24];
                                            c[..8].copy_from_slice(cookie);
                                            c[8..].copy_from_slice(&hmac[..16]);
                                            println!("UNVERIFIED");

                                            let mut record = OptRecord::new(record.get_payload_size(), record.get_ext_rcode(), record.get_edns_version(), record.get_flags());
                                            record.insert_option(OptCodes::Cookie, c);
                                            response.add_additional_record("", record.upcast());

                                        }
                                        24 => { //CLIENT + SERVER
                                            let client_cookie = &cookie[..8];
                                            let server_cookie = &cookie[8..];

                                            let hmac = match src_addr.ip() {
                                                IpAddr::V4(addr) => hmac::<Sha256>(COOKIE_SECRET, &[client_cookie, addr.octets().as_slice()].concat()),
                                                IpAddr::V6(addr) => hmac::<Sha256>(COOKIE_SECRET, &[client_cookie, addr.octets().as_slice()].concat())
                                            };

                                            //VERIFY SERVER_COOKIE - EXISTS
                                            if server_cookie == &hmac[..16] {
                                                // Valid — echo the same cookie
                                                println!("VERIFIED");

                                                let mut record = OptRecord::new(record.get_payload_size(), record.get_ext_rcode(), record.get_edns_version(), record.get_flags());
                                                record.insert_option(OptCodes::Cookie, cookie.to_vec());
                                                response.add_additional_record("", record.upcast());

                                            } else {
                                                let mut c = vec![0u8; 24];
                                                c[..8].copy_from_slice(cookie);
                                                c[8..].copy_from_slice(&hmac[..16]);

                                                let mut record = OptRecord::new(record.get_payload_size(), record.get_ext_rcode(), record.get_edns_version(), record.get_flags());
                                                record.insert_option(OptCodes::Cookie, c);
                                                response.add_additional_record("", record.upcast());

                                                println!("FALSE COOKIE");
                                            }
                                        }
                                        _ => unimplemented!()
                                    }
                                    println!("{:x?}", cookie);
                                }



                                //VERIFY ITS OPT

                                //CHECK IF COOKIE IS PRESENT

                                //CHECK BYTE LENGTH OF COOKIE - 8 = CLIENT ONLY - 24 = CLIENT + SERVER

                                //VALIDATE COOKIE FROM OUR HASHMAP???
                            }
                        }
                    }
*/

                    //let buf = response.to_bytes(MAX_TCP_MESSAGE_SIZE);

                    //stream.write(&(buf.len() as u16).to_be_bytes()).unwrap();
                    //stream.write(&buf).unwrap();
                    //stream.flush().unwrap();

                    println!("{}", response);

                    for buf in response.wire_chunks(MAX_TCP_MESSAGE_SIZE) {
                        stream.write(&(buf.len() as u16).to_be_bytes()).unwrap();
                        stream.write(&buf).unwrap();
                        stream.flush().unwrap();
                    }
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

    pub fn get_socket(&self) -> Option<&TcpListener> {
        self.socket.as_ref()
    }
}

impl Server for TcpServer {

    fn run(&mut self, port: u16) -> io::Result<JoinHandle<()>> {
        if self.is_running() {
            return Err(io::Error::new(io::ErrorKind::Unsupported, "Dns is already running"));
        }

        self.socket = Some(TcpListener::bind(SocketAddr::from((Ipv4Addr::UNSPECIFIED, port)))?);

        self.running.store(true, Ordering::Relaxed);

        Ok(thread::spawn({
            let socket = self.socket.as_ref().unwrap().try_clone()?;
            let running = Arc::clone(&self.running);
            let throttle = SpamThrottle::new();
            let on_receive = self.on_receive();

            move || {
                let mut last_decay_time = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("Time went backwards")
                    .as_millis();

                while running.load(Ordering::Relaxed) {
                    match socket.accept() {
                        Ok((stream, src_addr)) => {
                            let now = SystemTime::now()
                                .duration_since(UNIX_EPOCH)
                                .expect("Time went backwards")
                                .as_millis();

                            if now - last_decay_time >= 1000 {
                                throttle.decay();
                                last_decay_time = now;
                            }

                            if !throttle.add_and_test(src_addr.ip()) {
                                on_receive(stream, src_addr);
                            }
                        }
                        Err(_) => break
                    }
                }
            }
        }))
    }

    fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }

    fn kill(&self) {
        self.running.store(false, Ordering::Relaxed);
    }

    fn register_query_listener<F>(&self, key: RRTypes, callback: F)
    where
        F: Fn(&mut QueryEvent) -> ResponseResult<()> + Send + Sync + 'static
    {
        self.query_mapping.write().unwrap().insert(key, Box::new(callback));
    }
}
