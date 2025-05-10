use std::{io, thread};
use std::collections::HashMap;
use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr, UdpSocket};
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Sender, TryRecvError};
use std::thread::JoinHandle;
use std::time::{SystemTime, UNIX_EPOCH};
use rlibdns::messages::inter::record_types::RecordTypes;
use rlibdns::messages::message_base::MessageBase;
use rlibdns::records::a_record::ARecord;
use rlibdns::records::cname_record::CNameRecord;
use rlibdns::records::inter::record_base::RecordBase;
use rlibdns::utils::dns_query::DnsQuery;
use crate::database::sqlite::Database;
use crate::rpc::events::inter::dns_message_event::DnsMessageEvent;
use crate::rpc::events::inter::event::Event;
use crate::rpc::events::request_event::RequestEvent;
use crate::rpc::response_tracker::ResponseTracker;
use crate::utils::net::address_utils::is_bogon;
use crate::utils::spam_throttle::SpamThrottle;

pub struct Dns {
    server: Option<UdpSocket>,
    fallback: Vec<SocketAddr>,
    running: Arc<AtomicBool>,
    tx_sender_pool: Option<Sender<(Vec<u8>, SocketAddr)>>,
    request_mapping: Arc<Mutex<HashMap<RecordTypes, Vec<Box<dyn Fn(&mut RequestEvent) -> io::Result<()> + Send>>>>>,
    sender_throttle: SpamThrottle,
    receiver_throttle: SpamThrottle
}

impl Dns {

    pub fn new() -> Self {
        Self {
            server: None,
            fallback: Vec::new(),
            running: Arc::new(AtomicBool::new(false)),
            tx_sender_pool: None,
            request_mapping: Arc::new(Mutex::new(HashMap::new())),
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
            let fallback = self.fallback.clone();
            let running = Arc::clone(&self.running);
            let sender_throttle = self.sender_throttle.clone();
            let receiver_throttle = self.receiver_throttle.clone();

            move || {
                let mut tracker = ResponseTracker::new();

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

    pub fn add_fallback(&mut self, addr: SocketAddr) {
        self.fallback.push(addr);
    }

    pub fn remove_fallback(&mut self, addr: SocketAddr) {
        self.fallback.retain(|&x| x != addr);
    }

    pub fn register_request_listener<F>(&mut self, key: RecordTypes, callback: F)
    where
        F: Fn(&mut RequestEvent) -> io::Result<()> + Send + 'static
    {
        if self.request_mapping.lock().unwrap().contains_key(&key) {
            self.request_mapping.lock().unwrap().get_mut(&key).unwrap().push(Box::new(callback));
            return;
        }
        self.request_mapping.lock().unwrap().insert(key, vec![Box::new(callback)]);
    }

    fn on_receive<F>(&self, send: F) -> impl Fn(&[u8], SocketAddr)
    where
        F: Fn(&MessageBase) -> io::Result<()> + Send + 'static
    {
        let request_mapping = self.request_mapping.clone();
        //let send = self.send();
        move |data, src_addr| {
            match MessageBase::from_bytes(&data) {
                Ok(mut message) => {
                    message.set_origin(src_addr);
                    //message.set_destination(server.local_addr().unwrap());

                    if message.is_qr() {
                        //continue;
                        return;
                    }

                    let mut response = MessageBase::new(message.get_id());
                    response.set_op_code(message.get_op_code());
                    response.set_qr(true);
                    response.set_authoritative(true);
                    //response.set_origin(message.get_destination().unwrap());
                    response.set_destination(message.get_origin().unwrap());

                    let mut request_event = RequestEvent::new(message);
                    request_event.set_response(response);

                    //let is_bogon = is_bogon(message.get_origin().unwrap());//if  { "network < 2" } else { "network > 0" };

                    for query in request_event.get_message().get_queries() {
                        if let Some(callbacks) = request_mapping.lock().unwrap().get(&query.get_type()) {
                            for callback in callbacks {
                                callback(&mut request_event);
                            }
                        }
                    }

                    if request_event.is_prevent_default() {
                        return;
                    }

                    send(&request_event.get_response().unwrap());
                }
                Err(_) => {}
            }
        }
    }

    pub fn send(&self, message: &MessageBase) -> io::Result<()> {
        if message.get_destination().is_none() {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Message destination set to null"));
        }

        if !self.sender_throttle.add_and_test(message.get_destination().unwrap().ip()) {
            self.tx_sender_pool.as_ref().unwrap().send((message.to_bytes(), message.get_destination().unwrap())).unwrap();
        }

        Ok(())
    }

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

fn on_response(database: Option<Database>) -> impl Fn(&MessageBase) -> io::Result<MessageBase> {
    move |request| {
        if database.is_none() {
            return Err(io::Error::new(io::ErrorKind::Other, "Database not set"));
        }

        let mut response = MessageBase::new(request.get_id());
        response.set_op_code(request.get_op_code());
        response.set_qr(true);
        response.set_authoritative(true);
        response.set_origin(request.get_destination().unwrap());
        response.set_destination(request.get_origin().unwrap());

        let is_bogon = is_bogon(request.get_origin().unwrap());//if  { "network < 2" } else { "network > 0" };

        for query in request.get_queries() {
            match query.get_type() {
                RecordTypes::A => get_a_records(&database.as_ref().unwrap(), &query, &mut response, is_bogon)?,
                /*
                RecordTypes::Aaaa => {}
                RecordTypes::Ns => {}
                RecordTypes::Cname => {}
                RecordTypes::Soa => {}
                RecordTypes::Ptr => {}
                RecordTypes::Mx => {}
                RecordTypes::Txt => {}
                RecordTypes::Srv => {}
                RecordTypes::Opt => {}
                RecordTypes::Rrsig => {}
                RecordTypes::Nsec => {}
                RecordTypes::DnsKey => {}
                RecordTypes::Https => {}
                RecordTypes::Spf => {}
                RecordTypes::Tsig => {}
                RecordTypes::Any => {}
                RecordTypes::Caa => {}
                */
                _ => todo!()
            }
        }

        Ok(response)
    }
}

pub fn get_a_records(database: &Database, query: &DnsQuery, response: &mut MessageBase, is_bogon: bool) -> io::Result<()> {
    let records = database.get(
        "cname",
        Some(vec!["class", "ttl", "target", "network"]),
        Some(format!("class = {} AND name = '{}' AND {}", query.get_dns_class().get_code(), query.get_query().unwrap().to_lowercase(), is_bogon).as_str())
    );
    response.add_query(query.clone());

    if !records.is_empty() {
        for record in records {
            let ttl = record.get("ttl").unwrap().parse::<u32>().unwrap();
            let target = record.get("target").unwrap().to_string();
            response.add_answers(query.get_query().unwrap(), Box::new(CNameRecord::new(query.get_dns_class(), ttl, &target)));

            let records = database.get(
                "a",
                Some(vec!["class", "ttl", "address", "network"]),
                Some(format!("class = {} AND name = '{}' AND {}", query.get_dns_class().get_code(), target, is_bogon).as_str())
            );

            if records.is_empty() {
                return Err(io::Error::new(io::ErrorKind::Other, "Document not found"));
            }

            for record in records {
                let ttl = record.get("ttl").unwrap().parse::<u32>().unwrap();
                let address = record.get("address").unwrap().parse::<u32>().unwrap();

                response.add_answers(&target, Box::new(ARecord::new(query.get_dns_class(), false, ttl, Ipv4Addr::from(address))));
            }
        }

    } else {
        let records = database.get(
            "a",
            Some(vec!["class", "ttl", "address", "network"]),
            Some(format!("class = {} AND name = '{}' AND {}", query.get_dns_class().get_code(), query.get_query().unwrap().to_lowercase(), is_bogon).as_str())
        );

        if records.is_empty() {
            return Err(io::Error::new(io::ErrorKind::Other, "Document not found"));
        }

        for record in records {
            let ttl = record.get("ttl").unwrap().parse::<u32>().unwrap();
            let address = record.get("address").unwrap().parse::<u32>().unwrap();

            response.add_answers(query.get_query().unwrap(), Box::new(ARecord::new(query.get_dns_class(), false, ttl, Ipv4Addr::from(address))));
        }
    }

    Ok(())
}

pub fn get_aaaa_records(database: &Database, query: &DnsQuery, response: &mut MessageBase, is_bogon: bool) -> io::Result<()> {
    let records = database.get(
        "cname",
        Some(vec!["class", "ttl", "target", "network"]),
        Some(format!("class = {} AND name = '{}' AND {}", query.get_dns_class().get_code(), query.get_query().unwrap().to_lowercase(), is_bogon).as_str())
    );
    response.add_query(query.clone());

    if !records.is_empty() {
        for record in records {
            let ttl = record.get("ttl").unwrap().parse::<u32>().unwrap();
            let target = record.get("target").unwrap().to_string();
            response.add_answers(query.get_query().unwrap(), Box::new(CNameRecord::new(query.get_dns_class(), ttl, &target)));

            let records = database.get(
                "aaaa",
                Some(vec!["class", "ttl", "address", "network"]),
                Some(format!("class = {} AND name = '{}' AND {}", query.get_dns_class().get_code(), target, is_bogon).as_str())
            );

            if records.is_empty() {
                return Err(io::Error::new(io::ErrorKind::Other, "Document not found"));
            }

            for record in records {
                let ttl = record.get("ttl").unwrap().parse::<u32>().unwrap();
                let address = record.get("address").unwrap().parse::<u32>().unwrap();

                response.add_answers(&target, Box::new(ARecord::new(query.get_dns_class(), false, ttl, Ipv4Addr::from(address))));
            }
        }

    } else {
        let records = database.get(
            "aaaa",
            Some(vec!["class", "ttl", "address", "network"]),
            Some(format!("class = {} AND name = '{}' AND {}", query.get_dns_class().get_code(), query.get_query().unwrap().to_lowercase(), is_bogon).as_str())
        );

        if records.is_empty() {
            return Err(io::Error::new(io::ErrorKind::Other, "Document not found"));
        }

        for record in records {
            let ttl = record.get("ttl").unwrap().parse::<u32>().unwrap();
            let address = record.get("address").unwrap().parse::<u32>().unwrap();

            response.add_answers(query.get_query().unwrap(), Box::new(ARecord::new(query.get_dns_class(), false, ttl, Ipv4Addr::from(address))));
        }
    }

    Ok(())
}
