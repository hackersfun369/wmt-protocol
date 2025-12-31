use crate::comm::{Request, Response};
use chrono::Utc;
use serde_json::json;

pub async fn handle_time(_req: &Request) -> String {
    let now = Utc::now();
    Response::ok("TIME")
        .with_data(json!({
            "utc": now.to_rfc3339(),
            "timestamp": now.timestamp(),
            "timezone": "UTC"
        }))
        .to_json()
}
