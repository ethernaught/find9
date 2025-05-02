use std::{fs, io, thread};
use std::collections::HashMap;
use std::os::unix::net::UnixDatagram;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::JoinHandle;
use rlibbencode::variables::bencode_object::{BencodeObject, PutObject};
use rlibbencode::variables::inter::bencode_variable::BencodeVariable;
use rlibdns::messages::inter::dns_classes::DnsClasses;
use crate::database::sqlite::{Database, SqlValue};
use crate::dns_ext::messages::inter::dns_classes_ext::DnsClassesExt;

const UNIX_RPC_PATH: &str = "/tmp/find9.sock";

pub struct UnixRpc {
    server: Option<UnixDatagram>,
    database: Option<Database>,
    running: Arc<AtomicBool>
}

impl UnixRpc {

    pub fn new() -> io::Result<Self> {
        Ok(Self {
            server: None,
            database: None,
            running: Arc::new(AtomicBool::new(false))
        })
    }

    pub fn start(&mut self) -> io::Result<JoinHandle<()>> {
        if self.is_running() {
            return Err(io::Error::new(io::ErrorKind::Other, "Server is already running"));
        }

        self.running.store(true, Ordering::Relaxed);
        if Path::new(UNIX_RPC_PATH).exists() {
            fs::remove_file(UNIX_RPC_PATH).unwrap();
        }

        self.server = Some(UnixDatagram::bind(UNIX_RPC_PATH)?);
        self.server.as_ref().unwrap().set_nonblocking(true)?;

        Ok(thread::spawn({
            let server = self.server.as_ref().unwrap().try_clone()?;
            let mut database = self.database.clone();
            let running = Arc::clone(&self.running);
            move || {
                let mut buf = [0u8; 65535];

                while running.load(Ordering::Relaxed) {
                    match server.recv_from(&mut buf) {
                        Ok((size, src_addr)) => {
                            if let Ok(bencode) = BencodeObject::decode(&buf[..size]) {
                                println!("{}", bencode.to_string());

                                let bencode = on_request(&mut database.as_mut().unwrap(), bencode);

                                let mut bencode = BencodeObject::new();
                                bencode.put("v", env!("CARGO_PKG_VERSION"));
                                bencode.put("s", 0);

                                server.send_to_addr(&bencode.encode(), &src_addr).unwrap();
                            }
                        }
                        Err(_) => {}
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

    pub fn set_database(&mut self, database: Database) {
        self.database = Some(database);
    }
}

fn on_request(database: &mut Database, bencode: BencodeObject) -> io::Result<u16> {
    match bencode.get_string("t").ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Type not found"))? {
        "create" => {
            println!("{:?}", bencode.get_string("t"));
            let record = bencode.get_object("q").unwrap().get_string("record").ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Record not found"))?;
            let class = DnsClasses::from_str(bencode.get_object("q").unwrap().get_string("class").ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Class not found"))?)?;

            let domain = bencode.get_object("q").unwrap().get_string("domain").ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Domain not found"))?;
            //let record = bencode.get_string("record").ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Record not found"))?;

            let local = bencode.get_object("q").unwrap().get_number::<u8>("local");
            let external = bencode.get_object("q").unwrap().get_number::<u8>("external");

            match record {
                "a" => {
                    let address = bencode.get_object("q").unwrap().get_number::<u32>("address").ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "IP Address not found"))?;
                    let ttl = bencode.get_object("q").unwrap().get_number::<u32>("ttl").ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "TTL not found"))?;

                    let mut row = HashMap::new();
                    row.insert("class", SqlValue::Uint(class.get_code() as u128));
                    row.insert("domain", SqlValue::Str(domain.to_string()));
                    row.insert("ttl", SqlValue::Uint(ttl as u128));
                    row.insert("address", SqlValue::Uint(address as u128));
                    row.insert("cache_flush", SqlValue::Str("false".to_string()));
                    row.insert("network", SqlValue::Int(0));

                    database.insert("a", &row);


                    println!("{} {} {} {} {}", record, class.to_string(), domain.to_string(), address.to_string(), ttl);

                }
                "aaaa" => {
                    let address = bencode.get_object("q").unwrap().get_number::<u128>("address").ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "IP Address not found"))?;
                    let ttl = bencode.get_object("q").unwrap().get_number::<u32>("ttl").ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "TTL not found"))?;

                }
                _ => unreachable!()
            }

            println!("{}  {}", record, class.to_string());
        }
        _ => unreachable!()
    }

    Ok(0)
}
