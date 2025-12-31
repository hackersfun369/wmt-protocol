// src/connection.rs
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub struct ConnectionInfo {
    pub id: u64,
    pub remote_addr: Option<String>,
    pub session_token: Option<String>,
    pub authenticated: bool,
}

pub type ConnectionStore = Arc<Mutex<HashMap<u64, ConnectionInfo>>>;

pub fn create_connection_store() -> ConnectionStore {
    Arc::new(Mutex::new(HashMap::new()))
}

pub fn make_connection_info(id: u64, remote: Option<SocketAddr>) -> ConnectionInfo {
    ConnectionInfo {
        id,
        remote_addr: remote.map(|a| a.to_string()),
        session_token: None,
        authenticated: false,
    }
}
