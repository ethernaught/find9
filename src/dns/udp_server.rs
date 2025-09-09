use std::{io, thread};
use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket};
use std::sync::{Arc, RwLock};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::JoinHandle;
use std::time::{SystemTime, UNIX_EPOCH};
use rlibdns::messages::inter::response_codes::ResponseCodes;
use rlibdns::messages::inter::rr_types::RRTypes;
use rlibdns::messages::message_base::MessageBase;
use rlibdns::records::inter::opt_codes::OptCodes;
use rlibdns::records::inter::record_base::RecordBase;
use rlibdns::records::opt_record::OptRecord;
use crate::dns::dns::{QueryMap, ResponseResult};
use crate::{BOGON_ALLOWED, COOKIE_SECRET, MAX_QUERIES};
use crate::dns::server::Server;
use crate::rpc::events::inter::event::Event;
use crate::rpc::events::query_event::QueryEvent;
use crate::utils::hash::hmac::hmac;
use crate::utils::hash::sha256::Sha256;
use crate::utils::net::address_utils::is_bogon;
use crate::utils::spam_throttle::SpamThrottle;

pub const MAX_UDP_MESSAGE_SIZE: usize = 512;

pub struct UdpServer {
    running: Arc<AtomicBool>,
    pub(crate) socket: Option<UdpSocket>,
    sender_throttle: SpamThrottle,
    query_mapping: QueryMap
}

impl UdpServer {

    pub fn new() -> Self {
        Self {
            running: Arc::new(AtomicBool::new(false)),
            socket: None,
            sender_throttle: SpamThrottle::new(),
            query_mapping: Arc::new(RwLock::new(HashMap::new()))
        }
    }

    fn on_receive(&self) -> impl Fn(&[u8], SocketAddr) {
        let socket = self.socket.as_ref().unwrap().try_clone().unwrap();
        let sender_throttle = self.sender_throttle.clone();

        let send = move |message: &MessageBase| {
            if message.get_destination().is_none() {
                return Err::<(), io::Error>(io::Error::new(io::ErrorKind::InvalidData, "Message destination set to null"));
            }

            if !sender_throttle.add_and_test(message.get_destination().unwrap().ip()) {
                socket.send_to(message.to_bytes(MAX_UDP_MESSAGE_SIZE).as_slice(), message.get_destination().unwrap())?;
            }

            Err(io::Error::new(io::ErrorKind::TooManyLinks, "Too many outgoing messages to ip"))
        };

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

                    for (i, query) in message.get_queries().enumerate() {
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
                                }
                                Err(e) => {
                                    response.set_response_code(e);
                                    break;
                                }
                            }

                            response.add_query(query.clone());
                            response.set_authoritative(event.is_authoritative());

                            if event.has_answers() {
                                for (query, record) in event.records[0].drain(..) {
                                    response.add_answer(&query, record);
                                }
                            }

                            if event.has_authority_records() {
                                for (query, record) in event.records[1].drain(..) {
                                    response.add_authority_record(&query, record);
                                }
                            }

                            if event.has_additional_records() {
                                for (query, record) in event.records[2].drain(..) {
                                    response.add_additional_record(&query, record);
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
                    if !response.has_answers() {
                        response.set_authoritative(false);
                        response.set_response_code(ResponseCodes::Refused);

                        let mut record = OptRecord::new(512, 0, 0, 0);
                        record.insert_option(OptCodes::EDnsError, vec![0x00, 0x14]);
                        response.add_additional_record("", record.upcast());
                    }
                    */

                    //println!("{}", response);

                    send(&response);
                }
                Err(_) => {}
            }
        }
    }

    fn send(&self, message: &MessageBase) -> io::Result<()> {
        if message.get_destination().is_none() {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Message destination set to null"));
        }

        if !self.sender_throttle.add_and_test(message.get_destination().unwrap().ip()) {
            self.socket.as_ref().unwrap().send_to(message.to_bytes(MAX_UDP_MESSAGE_SIZE).as_slice(), message.get_destination().unwrap())?;
        }

        Err(io::Error::new(io::ErrorKind::TooManyLinks, "Too many outgoing messages to ip"))
    }

    pub fn get_socket(&self) -> Option<&UdpSocket> {
        self.socket.as_ref()
    }
}

impl Server for UdpServer {
    
    fn run(&mut self, port: u16) -> io::Result<JoinHandle<()>> {
        if self.is_running() {
            return Err(io::Error::new(io::ErrorKind::Unsupported, "Dns is already running"));
        }

        self.socket = Some(UdpSocket::bind(SocketAddr::from((Ipv4Addr::UNSPECIFIED, port)))?);

        self.running.store(true, Ordering::Relaxed);

        Ok(thread::spawn({
            let socket = self.socket.as_ref().unwrap().try_clone()?;
            let running = Arc::clone(&self.running);
            let sender_throttle = self.sender_throttle.clone();
            let receiver_throttle = SpamThrottle::new();
            let on_receive = self.on_receive();

            move || {
                let mut buf = [0u8; 65535];
                let mut last_decay_time = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("Time went backwards")
                    .as_millis();

                while running.load(Ordering::Relaxed) {
                    match socket.recv_from(&mut buf) {
                        Ok((len, src_addr)) => {
                            let now = SystemTime::now()
                                .duration_since(UNIX_EPOCH)
                                .expect("Time went backwards")
                                .as_millis();

                            if now - last_decay_time >= 1000 {
                                receiver_throttle.decay();
                                sender_throttle.decay();
                                last_decay_time = now;
                            }

                            if !BOGON_ALLOWED && is_bogon(src_addr) {
                                continue;
                            }

                            if !receiver_throttle.add_and_test(src_addr.ip()) {
                                on_receive(&buf[..len], src_addr);
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
