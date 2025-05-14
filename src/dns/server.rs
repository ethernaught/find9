use std::{io, thread};
use std::collections::HashMap;
use std::net::{Ipv4Addr, SocketAddr, UdpSocket};
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Sender, TryRecvError};
use std::thread::{sleep, JoinHandle};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use rlibdns::messages::inter::record_types::RecordTypes;
use rlibdns::messages::inter::response_codes::ResponseCodes;
use rlibdns::messages::message_base::MessageBase;
use crate::rpc::events::inter::event::Event;
use crate::rpc::events::query_event::QueryEvent;
use crate::rpc::response_tracker::ResponseTracker;
use crate::utils::spam_throttle::SpamThrottle;

pub struct Server {
    server: Option<UdpSocket>,
    tracker: ResponseTracker,
    running: Arc<AtomicBool>,
    tx_sender_pool: Option<Sender<(Vec<u8>, SocketAddr)>>,
    query_mapping: Arc<Mutex<HashMap<RecordTypes, Vec<Box<dyn Fn(&mut QueryEvent) -> io::Result<()> + Send>>>>>,
    sender_throttle: SpamThrottle,
    receiver_throttle: SpamThrottle
}

impl Server {

    pub fn new() -> Self {
        Self {
            server: None,
            tracker: ResponseTracker::new(),
            running: Arc::new(AtomicBool::new(false)),
            tx_sender_pool: None,
            query_mapping: Arc::new(Mutex::new(HashMap::new())),
            sender_throttle: SpamThrottle::new(),
            receiver_throttle: SpamThrottle::new()
        }
    }

    pub fn start(&mut self, port: u16) -> io::Result<JoinHandle<()>> {
        if self.is_running() {
            return Err(io::Error::new(io::ErrorKind::Other, "Server is already running"));
        }

        self.running.store(true, Ordering::Relaxed);
        self.server = Some(UdpSocket::bind(SocketAddr::from((Ipv4Addr::UNSPECIFIED, port)))?);
        self.server.as_ref().unwrap().set_nonblocking(true)?;

        let (tx_sender_pool, rx_sender_pool) = channel();
        let on_receive = self.on_receive(self.send_message(&tx_sender_pool));
        self.tx_sender_pool = Some(tx_sender_pool);

        Ok(thread::spawn({
            let server = self.server.as_ref().unwrap().try_clone()?;
            let running = Arc::clone(&self.running);
            let sender_throttle = self.sender_throttle.clone();
            let receiver_throttle = self.receiver_throttle.clone();
            let tracker = self.tracker.clone();

            move || {
                let mut buf = [0u8; 65535];
                let mut last_decay_time = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("Time went backwards")
                    .as_millis();

                while running.load(Ordering::Relaxed) {
                    match server.recv_from(&mut buf) {
                        Ok((size, src_addr)) => {
                            if !receiver_throttle.add_and_test(src_addr.ip()) {
                                on_receive(&buf[..size], src_addr);
                            }
                        }
                        Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {}
                        _ => break
                    }

                    match rx_sender_pool.try_recv() {
                        Ok((data, dst_addr)) => {
                            if !sender_throttle.test(dst_addr.ip()) {
                                server.send_to(data.as_slice(), dst_addr);
                            }
                        }
                        Err(TryRecvError::Empty) => {}
                        Err(TryRecvError::Disconnected) => break
                    }

                    let now = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .expect("Time went backwards")
                        .as_millis();

                    if now - last_decay_time >= 1000 {
                        receiver_throttle.decay();
                        sender_throttle.decay();
                        tracker.remove_stalled();

                        last_decay_time = now;
                    }

                    sleep(Duration::from_millis(1));
                }
            }
        }))
    }

    pub fn stop(&self) {
        self.running.store(false, Ordering::Relaxed);
    }

    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }

    pub fn register_query_listener<F>(&self, key: RecordTypes, callback: F)
    where
        F: Fn(&mut QueryEvent) -> io::Result<()> + Send + 'static
    {
        if self.query_mapping.lock().unwrap().contains_key(&key) {
            self.query_mapping.lock().unwrap().get_mut(&key).unwrap().push(Box::new(callback));
            return;
        }
        self.query_mapping.lock().unwrap().insert(key, vec![Box::new(callback)]);
    }

    fn on_receive<F>(&self, send: F) -> impl Fn(&[u8], SocketAddr)
    where
        F: Fn(&MessageBase) -> io::Result<()> + Send + 'static
    {
        let query_mapping = self.query_mapping.clone();
        let tracker = self.tracker.clone();

        move |data, src_addr| {
            match MessageBase::from_bytes(&data) {
                Ok(mut message) => {
                    message.set_origin(src_addr);

                    if message.is_qr() {
                        if let Some(call) = tracker.poll(message.get_id()) {
                            message.set_authoritative(false);
                            message.set_destination(*call.get_address());
                            send(&message).unwrap();
                        }

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

                    //let is_bogon = is_bogon(message.get_origin().unwrap());//if  { "network < 2" } else { "network > 0" };

                    for query in message.get_queries() {
                        if let Some(callbacks) = query_mapping.lock().unwrap().get(&query.get_type()) {
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

                    send(&response);
                }
                Err(_) => {}
            }
        }
    }

    /*
    pub fn send(&self, message: &MessageBase) -> io::Result<()> {
        if message.get_destination().is_none() {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Message destination set to null"));
        }

        if !self.sender_throttle.add_and_test(message.get_destination().unwrap().ip()) {
            self.tx_sender_pool.as_ref().unwrap().send((message.to_bytes(), message.get_destination().unwrap())).unwrap();
        }

        Ok(())
    }
    */

    fn send_message(&self, tx: &Sender<(Vec<u8>, SocketAddr)>) -> impl Fn(&MessageBase) -> io::Result<()> {
        let tx = tx.clone();
        let sender_throttle = self.sender_throttle.clone();
        move |message| {
            if message.get_destination().is_none() {
                return Err(io::Error::new(io::ErrorKind::InvalidData, "Message destination set to null"));
            }

            if !sender_throttle.add_and_test(message.get_destination().unwrap().ip()) {
                tx.send((message.to_bytes(), message.get_destination().unwrap())).unwrap();
            }

            Ok(())
        }
    }
}
