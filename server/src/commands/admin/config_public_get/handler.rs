use crate::comm::{Request, Response};
use serde_json::json;

pub async fn handle_config_public_get(_req: &Request) -> String {
    Response::ok("CONFIG_PUBLIC_GET")
        .with_data(json!({
            "max_messages_per_page": 100,
            "max_attachments": 10,
            "rate_limits": {
                "auth": "5/min",
                "send": "100/hour"
            },
            "environment": "development"
        }))
        .to_json()
}
