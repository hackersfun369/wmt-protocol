use crate::comm::{Request, Response};
use serde_json::json;

pub async fn handle_version_check(req: &Request) -> String {
    let client_version = req.data.get("version").and_then(|v| v.as_str()).unwrap_or("unknown");
    let server_version = "1.0.0";
    
    Response::ok("VERSION_CHECK")
        .with_data(json!({
            "client_version": client_version,
            "server_version": server_version,
            "compatible": true
        }))
        .to_json()
}
