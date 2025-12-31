use crate::comm::{Request, Response};
use serde_json::json;

pub async fn handle_connection_info(_req: &Request, remote_addr: &str) -> String {
    Response::ok("CONNECTION_INFO")
        .with_data(json!({
            "remote_addr": remote_addr,
            "protocol": "WebTransport/QUIC",
            "version": "1.0"
        }))
        .to_json()
}
