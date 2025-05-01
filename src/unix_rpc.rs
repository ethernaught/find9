use std::{fs, io, thread};
use std::os::unix::net::UnixDatagram;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::JoinHandle;
use rlibbencode::variables::bencode_object::{BencodeObject, PutObject};
use rlibbencode::variables::inter::bencode_variable::BencodeVariable;
use rlibdns::messages::inter::dns_classes::DnsClasses;
use crate::dns_ext::messages::inter::dns_classes_ext::DnsClassesExt;

const UNIX_RPC_PATH: &str = "/tmp/find9.sock";

pub struct UnixRpc {
    server: Option<UnixDatagram>,
    running: Arc<AtomicBool>
}

impl UnixRpc {

    pub fn new() -> io::Result<Self> {
        Ok(Self {
            server: None,
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
            let running = Arc::clone(&self.running);
            move || {
                let mut buf = [0u8; 65535];

                while running.load(Ordering::Relaxed) {
                    match server.recv_from(&mut buf) {
                        Ok((size, src_addr)) => {
                            if let Ok(bencode) = BencodeObject::decode(&buf[..size]) {
                                println!("{}", bencode.to_string());

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
}

fn on_request(bencode: BencodeObject) -> io::Result<u16> {
    match bencode.get_string("t")? {
        "create" => {
            let record = bencode.get_string("record")?;
            let class = DnsClasses::from_str(bencode.get_string("class")?)?;

        }
        _ => unreachable!()
    }

    Ok(0)
}
