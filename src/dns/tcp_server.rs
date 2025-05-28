use std::collections::HashMap;
use std::net::{SocketAddr, UdpSocket};
use std::sync::{Arc, RwLock};
use std::sync::atomic::AtomicBool;
use std::sync::mpsc::Sender;
use crate::dns::dns::QueryMap;

pub struct TcpServer {
    running: Arc<AtomicBool>,
    pub(crate) socket: Option<UdpSocket>,
    tx_sender_pool: Option<Sender<(Vec<u8>, SocketAddr)>>,
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
}
