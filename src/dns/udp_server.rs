use std::{io, thread};
use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket};
use std::sync::{Arc, RwLock};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Sender, TryRecvError};
use std::thread::{sleep, JoinHandle};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use rlibdns::messages::inter::response_codes::ResponseCodes;
use rlibdns::messages::inter::rr_types::RRTypes;
use rlibdns::messages::message_base::MessageBase;
use rlibdns::records::inter::opt_codes::OptCodes;
use rlibdns::records::inter::record_base::RecordBase;
use rlibdns::records::opt_record::OptRecord;
use crate::dns::dns::QueryMap;
use crate::{COOKIE_SECRET, MAX_QUERIES};
use crate::dns::listeners::errors::response_error::ResponseResult;
use crate::rpc::events::inter::event::Event;
use crate::rpc::events::query_event::QueryEvent;
use crate::utils::hash::hmac::hmac;
use crate::utils::hash::sha256::Sha256;
use crate::utils::spam_throttle::SpamThrottle;

pub const MAX_UDP_MESSAGE_SIZE: usize = 512;

pub struct UdpServer {
    running: Arc<AtomicBool>,
    pub(crate) socket: Option<UdpSocket>,
    tx_sender_pool: Option<Sender<(Vec<u8>, SocketAddr)>>,
    query_mapping: QueryMap
}

impl UdpServer {

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

        self.socket = Some(UdpSocket::bind(SocketAddr::from((Ipv4Addr::UNSPECIFIED, port)))?);
        self.socket.as_ref().unwrap().set_nonblocking(true)?;

        let sender_throttle = SpamThrottle::new();

        let (tx_sender_pool, rx_sender_pool) = channel();
        self.tx_sender_pool = Some(tx_sender_pool);

        let on_receive = self.on_receive(self.send_message(&sender_throttle));

        self.running.store(true, Ordering::Relaxed);

        Ok(thread::spawn({
            let socket = self.socket.as_ref().unwrap().try_clone()?;
            let running = Arc::clone(&self.running);
            let sender_throttle = sender_throttle.clone();
            let receiver_throttle = SpamThrottle::new();

            move || {
                let mut buf = [0u8; 65535];
                let mut last_decay_time = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("Time went backwards")
                    .as_millis();

                while running.load(Ordering::Relaxed) {
                    match socket.recv_from(&mut buf) {
                        Ok((len, src_addr)) => {
                            if !receiver_throttle.add_and_test(src_addr.ip()) {
                                on_receive(&buf[..len], src_addr);
                            }
                        }
                        Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {}
                        _ => break
                    }

                    match rx_sender_pool.try_recv() {
                        Ok((data, dst_addr)) => {
                            if !sender_throttle.test(dst_addr.ip()) {
                                socket.send_to(data.as_slice(), dst_addr);
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

    fn on_receive<F>(&self, send: F) -> impl Fn(&[u8], SocketAddr)
    where
        F: Fn(&MessageBase) -> io::Result<()> + Send + 'static
    {
        let query_mapping = self.query_mapping.clone();

        move |buf, src_addr| {
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
                        send(&response);
                        return;
                    }

                    //EVENT SHOULD PROBABLY PASS THE RESPONSE NOT JUST HOLD INFORMATION TO ADD LATER ON...

                    println!("{}", message);

                    for (i, query) in message.get_queries_mut().drain(..).enumerate() {
                        if i >= MAX_QUERIES {
                            break;
                        }

                        if let Some(callbacks) = query_mapping.read().unwrap().get(&query.get_type()) {
                            let mut event = QueryEvent::new(query.clone());

                            for callback in callbacks {
                                match callback(&mut event) {
                                    Ok(_) => {}
                                    Err(e) => {
                                        response.set_response_code(e.get_code());
                                        break;
                                    }
                                }
                            }

                            if event.is_prevent_default() {
                                break;
                            }

                            response.add_query(event.get_query().clone());
                            response.set_authoritative(event.is_authoritative());

                            if event.has_answers() {
                                for (query, records) in event.get_answers_mut().drain() {
                                    for record in records {
                                        response.add_answer(&query, record);
                                    }
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



                    /*
                    if response.has_answers() {
                        response.set_authoritative(true);

                    } else {
                        response.set_authoritative(false);
                        response.set_response_code(ResponseCodes::Refused);

                        let mut record = OptRecord::new(512, 0, 0, 0);
                        record.insert_option(OptCodes::EDnsError, vec![0x00, 0x14]);
                        response.add_additional_record("", record.upcast());
                    }
                    */

                    println!("{}", response);

                    //IF NAME DOES EXIST BUT NO DATA RETURN
                    // NoError

                    //If QName does not exist
                    // NxDomain

                    if !response.has_name_servers() &&
                        !response.has_additional_records() {
                        //response.set_response_code(ResponseCodes::NxDomain);
                    }

                    send(&response);
                }
                Err(_) => {}
            }
        }
    }

    fn send_message(&self, sender_throttle: &SpamThrottle) -> impl Fn(&MessageBase) -> io::Result<()> {
        let tx = self.tx_sender_pool.as_ref().unwrap().clone();
        let sender_throttle = sender_throttle.clone();

        move |message| {
            if message.get_destination().is_none() {
                return Err(io::Error::new(io::ErrorKind::InvalidData, "Message destination set to null"));
            }

            if !sender_throttle.add_and_test(message.get_destination().unwrap().ip()) {
                tx.send((message.to_bytes(MAX_UDP_MESSAGE_SIZE), message.get_destination().unwrap())).unwrap();
            }

            Ok(())
        }
    }

    pub fn register_query_listener<F>(&self, key: RRTypes, callback: F)
    where
        F: Fn(&mut QueryEvent) -> ResponseResult<()> + Send + Sync + 'static
    {
        if self.query_mapping.read().unwrap().contains_key(&key) {
            self.query_mapping.write().unwrap().get_mut(&key).unwrap().push(Box::new(callback));
            return;
        }
        self.query_mapping.write().unwrap().insert(key, vec![Box::new(callback)]);
    }

    pub fn get_socket(&self) -> Option<&UdpSocket> {
        self.socket.as_ref()
    }
}
