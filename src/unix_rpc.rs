use std::{fs, io, thread};
use std::collections::HashMap;
use std::os::unix::net::UnixDatagram;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::JoinHandle;
use rlibbencode::bencode;
use rlibbencode::variables::bencode_array::{AddArray, BencodeArray};
use rlibbencode::variables::bencode_bytes::BencodeBytes;
use rlibbencode::variables::bencode_number::BencodeNumber;
use rlibbencode::variables::bencode_object::{BencodeObject, GetObject, PutObject};
use rlibbencode::variables::inter::bencode_variable::{BencodeVariable, FromBencode, ToBencode};
use rlibdns::messages::inter::dns_classes::DnsClasses;
use crate::database::sqlite::Database;
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
            let database = self.database.clone();
            let running = Arc::clone(&self.running);
            move || {
                let mut buf = [0u8; 65535];

                while running.load(Ordering::Relaxed) {
                    match server.recv_from(&mut buf) {
                        Ok((size, src_addr)) => {
                            if let Ok(bencode) = BencodeObject::from_bencode(&buf[..size]) {
                                //let response = on_request(&database.as_ref().unwrap(), bencode);

                                let bencode = match on_request(&database.as_ref().unwrap(), bencode) {
                                    Ok(bencode) => {
                                        bencode
                                    }
                                    Err(e) => {
                                        let m = e.to_string();
                                        let v = env!("CARGO_PKG_VERSION");
                                        bencode!({
                                            "s": 100,
                                            "m": m,
                                            "v": v
                                        })
                                    }
                                };

                                server.send_to_addr(&bencode.to_bencode(), &src_addr).unwrap();
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

fn on_request(database: &Database, bencode: BencodeObject) -> io::Result<BencodeObject> {
    Ok(match bencode.get::<BencodeBytes>("t").ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Type not found"))?.as_str() {
        "create" => {
            on_create_record(database, bencode.get::<BencodeObject>("q").unwrap())?;

            let v = env!("CARGO_PKG_VERSION");
            bencode!({
                "s": 0,
                "v": v
            })
        }
        "get" => {
            let (status, records) = on_get_record(database, bencode.get::<BencodeObject>("q").unwrap())?;

            let mut r = BencodeArray::new();
            for record in records {
                let mut obj = BencodeObject::new();
                for (key, value) in record {
                    obj.put(key, value);
                }
                r.push(obj);
            }

            let v = env!("CARGO_PKG_VERSION");
            bencode!({
                "s": 0,
                "v": v,
                "r": r
            })
        }
        "remove" => {
            on_remove_record(database, bencode.get::<BencodeObject>("q").unwrap())?;

            let v = env!("CARGO_PKG_VERSION");
            bencode!({
                "s": 0,
                "v": v
            })
        }
        _ => unreachable!()
    })
}

fn on_create_record(database: &Database, bencode: &BencodeObject) -> io::Result<u16> {
    let record = bencode.get::<BencodeBytes>("record").ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Record not found"))?.to_string();
    let class = DnsClasses::from_str(bencode.get::<BencodeBytes>("class").ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Class not found"))?.as_str())?;

    let name = bencode.get::<BencodeBytes>("name").ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Name not found"))?.to_string();
    let ttl = bencode.get::<BencodeNumber>("ttl").ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "TTL not found"))?.parse::<u32>().unwrap();

    let network = {
        let local = match bencode.get::<BencodeNumber>("local") {
            Some(b) => b.parse::<u8>().unwrap() != 0,
            None => false
        };
        let external = match bencode.get::<BencodeNumber>("external") {
            Some(b) => b.parse::<u8>().unwrap() != 0,
            None => false
        };

        match (local, external) {
            (true, true) | (false, false) => 1,
            (false, true) => 2,
            (true, false) => 0
        }
    };

    match record.as_str() {
        "a" => {
            let address = bencode.get::<BencodeNumber>("address").ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "IP Address not found"))?.parse::<u32>().unwrap();

            let mut row = HashMap::new();
            row.insert("class", class.get_code().into());
            row.insert("name", name.into());
            row.insert("ttl", ttl.into());
            row.insert("address", address.into());
            row.insert("network", network.into());
            database.insert("a", &row);
        }
        "aaaa" => {
            let address = bencode.get::<BencodeNumber>("address").ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "IP Address not found"))?.parse::<u128>().unwrap();

            let mut row = HashMap::new();
            row.insert("class", class.get_code().into());
            row.insert("name", name.into());
            row.insert("ttl", ttl.into());
            row.insert("address", address.into());
            row.insert("network", network.into());
            database.insert("aaaa", &row);
        }
        "cname" => {
            let target = bencode.get::<BencodeBytes>("target").ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Target not found"))?.to_string();

            let mut row = HashMap::new();
            row.insert("class", class.get_code().into());
            row.insert("name", name.into());
            row.insert("ttl", ttl.into());
            row.insert("target", target.into());
            row.insert("network", network.into());
            database.insert("cname", &row);
        }
        "dnskey" => {
            let flags = bencode.get::<BencodeNumber>("flags").ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Flags not found"))?.parse::<u16>().unwrap();
            let protocol = bencode.get::<BencodeNumber>("protocol").ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Protocol not found"))?.parse::<u8>().unwrap();
            let algorithm = bencode.get::<BencodeNumber>("algorithm").ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Algorithm not found"))?.parse::<u8>().unwrap();
            let public_key = bencode.get::<BencodeBytes>("public_key").ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Public Key not found"))?.to_string();

            let mut row = HashMap::new();
            row.insert("class", class.get_code().into());
            row.insert("name", name.into());
            row.insert("ttl", ttl.into());
            row.insert("flags", flags.into());
            row.insert("protocol", protocol.into());
            row.insert("algorithm", algorithm.into());
            row.insert("public_key", public_key.into());
            row.insert("network", network.into());
            database.insert("dnskey", &row);
        }
        "https" => {
            let priority = bencode.get::<BencodeNumber>("priority").ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Priority not found"))?.parse::<u16>().unwrap();
            let target = bencode.get::<BencodeBytes>("target").ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Target not found"))?.to_string();
            //let params = bencode.get::<BencodeNumber>("params").ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Params not found"))?.parse::<u8>().unwrap();

            let mut row = HashMap::new();
            row.insert("class", class.get_code().into());
            row.insert("name", name.into());
            row.insert("ttl", ttl.into());
            row.insert("priority", priority.into());
            row.insert("target", target.into());
            //row.insert("params", params.into());
            row.insert("network", network.into());
            database.insert("https", &row);
        }
        "mx" => {
            let priority = bencode.get::<BencodeNumber>("priority").ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Priority not found"))?.parse::<u16>().unwrap();
            let server = bencode.get::<BencodeBytes>("server").ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Server not found"))?.to_string();

            let mut row = HashMap::new();
            row.insert("class", class.get_code().into());
            row.insert("name", name.into());
            row.insert("ttl", ttl.into());
            row.insert("priority", priority.into());
            row.insert("server", server.into());
            row.insert("network", network.into());
            database.insert("mx", &row);
        }
        "ns" => {
            let server = bencode.get::<BencodeBytes>("server").ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Server not found"))?.to_string();

            let mut row = HashMap::new();
            row.insert("class", class.get_code().into());
            row.insert("name", name.into());
            row.insert("ttl", ttl.into());
            row.insert("server", server.into());
            row.insert("network", network.into());
            database.insert("ns", &row);
        }
        "ptr" => {
            let domain = bencode.get::<BencodeBytes>("domain").ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Domain not found"))?.to_string();

            let mut row = HashMap::new();
            row.insert("class", class.get_code().into());
            row.insert("name", name.into());
            row.insert("ttl", ttl.into());
            row.insert("domain", domain.into());
            row.insert("network", network.into());
            database.insert("ptr", &row);
        }
        "srv" => {
            let priority = bencode.get::<BencodeNumber>("priority").ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Priority not found"))?.parse::<u16>().unwrap();
            let weight = bencode.get::<BencodeNumber>("weight").ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Weight not found"))?.parse::<u16>().unwrap();
            let port = bencode.get::<BencodeNumber>("port").ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Port not found"))?.parse::<u16>().unwrap();
            let target = bencode.get::<BencodeBytes>("target").ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Target not found"))?.to_string();

            let mut row = HashMap::new();
            row.insert("class", class.get_code().into());
            row.insert("name", name.into());
            row.insert("ttl", ttl.into());
            row.insert("priority", priority.into());
            row.insert("weight", weight.into());
            row.insert("port", port.into());
            row.insert("target", target.into());
            row.insert("network", network.into());
            database.insert("srv", &row);
        }
        "txt" => {
            let content = bencode.get::<BencodeBytes>("content").ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Content not found"))?.to_string();

            let mut row = HashMap::new();
            row.insert("class", class.get_code().into());
            row.insert("name", name.into());
            row.insert("ttl", ttl.into());
            row.insert("content", content.into());
            row.insert("network", network.into());
            database.insert("txt", &row);
        }
        _ => unreachable!()
    }

    Ok(0)
}

fn on_get_record(database: &Database, bencode: &BencodeObject) -> io::Result<(u16, Vec<HashMap<String, String>>)> {
    let record = bencode.get::<BencodeObject>("q").unwrap().get::<BencodeBytes>("record").ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Record not found"))?.to_string();
    let class = DnsClasses::from_str(bencode.get::<BencodeObject>("q").unwrap().get::<BencodeBytes>("class").ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Class not found"))?.as_str())?;

    let name = bencode.get::<BencodeObject>("q").unwrap().get::<BencodeBytes>("name").ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Name not found"))?.to_string();
    //let ttl = bencode.get::<BencodeObject>("q").unwrap().get::<BencodeNumber>("ttl").ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "TTL not found"))?.parse::<u32>().unwrap();

    /*
    let is_bogon = {
        let local = match bencode.get::<BencodeObject>("q").unwrap().get::<BencodeNumber>("local") {
            Some(b) => b.parse::<u8>().unwrap() != 0,
            None => false
        };
        let external = match bencode.get::<BencodeObject>("q").unwrap().get::<BencodeNumber>("external") {
            Some(b) => b.parse::<u8>().unwrap() != 0,
            None => false
        };

        match (local, external) {
            (true, true) | (false, false) => "",
            (false, true) => " AND network > 0",
            (true, false) => " AND network < 2"
        }
    };
    */

    Ok((0, match record.as_str() {
        "a" => {
            //let address = bencode.get::<BencodeObject>("q").unwrap().get::<BencodeNumber>("address").ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "IP Address not found"))?.parse::<u32>().unwrap();

            database.get(
                "a",
                Some(vec!["class", "ttl", "address", "network"]),
                Some(format!("class = {} AND name = '{}'", class.get_code(), name).as_str())
            )
        }
        "aaaa" => {
            //let address = bencode.get::<BencodeObject>("q").unwrap().get::<BencodeNumber>("address").ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "IP Address not found"))?.parse::<u128>().unwrap();

            database.get(
                "aaaa",
                Some(vec!["class", "ttl", "address", "network"]),
                Some(format!("class = {} AND name = '{}'", class.get_code(), name).as_str())
            )
        }
        "cname" => {
            //let target = bencode.get::<BencodeObject>("q").unwrap().get::<BencodeBytes>("target").ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Target not found"))?.to_string();
            todo!()
        }
        "dnskey" => {
            //let flags = bencode.get::<BencodeObject>("q").unwrap().get::<BencodeNumber>("flags").ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Flags not found"))?.parse::<u16>().unwrap();
            //let protocol = bencode.get::<BencodeObject>("q").unwrap().get::<BencodeNumber>("protocol").ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Protocol not found"))?.parse::<u8>().unwrap();
            //let algorithm = bencode.get::<BencodeObject>("q").unwrap().get::<BencodeNumber>("algorithm").ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Algorithm not found"))?.parse::<u8>().unwrap();
            //let public_key = bencode.get::<BencodeObject>("q").unwrap().get::<BencodeBytes>("public_key").ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Public Key not found"))?.to_string();
            todo!()
        }
        "https" => {
            //let priority = bencode.get::<BencodeObject>("q").unwrap().get::<BencodeNumber>("priority").ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Priority not found"))?.parse::<u16>().unwrap();
            //let target = bencode.get::<BencodeObject>("q").unwrap().get::<BencodeBytes>("target").ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Target not found"))?.to_string();
            //let params = bencode.get::<BencodeObject>("q").unwrap().get::<BencodeNumber>("params").ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Params not found"))?.parse::<u8>().unwrap();
            todo!()
        }
        "mx" => {
            //let priority = bencode.get::<BencodeObject>("q").unwrap().get::<BencodeNumber>("priority").ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Priority not found"))?.parse::<u16>().unwrap();
            //let server = bencode.get::<BencodeObject>("q").unwrap().get::<BencodeBytes>("server").ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Server not found"))?.to_string();
            todo!()
        }
        "ns" => {
            //let server = bencode.get::<BencodeObject>("q").unwrap().get::<BencodeBytes>("server").ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Server not found"))?.to_string();
            todo!()
        }
        "ptr" => {
            //let domain = bencode.get::<BencodeObject>("q").unwrap().get::<BencodeBytes>("domain").ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Domain not found"))?.to_string();
            todo!()
        }
        "srv" => {
            //let priority = bencode.get::<BencodeObject>("q").unwrap().get::<BencodeNumber>("priority").ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Priority not found"))?.parse::<u16>().unwrap();
            //let weight = bencode.get::<BencodeObject>("q").unwrap().get::<BencodeNumber>("weight").ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Weight not found"))?.parse::<u16>().unwrap();
            //let port = bencode.get::<BencodeObject>("q").unwrap().get::<BencodeNumber>("port").ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Port not found"))?.parse::<u16>().unwrap();
            //let target = bencode.get::<BencodeObject>("q").unwrap().get::<BencodeBytes>("target").ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Target not found"))?.to_string();
            todo!()
        }
        "txt" => {
            //let content = bencode.get::<BencodeObject>("q").unwrap().get::<BencodeBytes>("content").ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Content not found"))?.to_string();
            todo!()
        }
        _ => unreachable!()
    }))
}

fn on_remove_record(database: &Database, bencode: &BencodeObject) -> io::Result<u16> {
    Ok(0)
}
