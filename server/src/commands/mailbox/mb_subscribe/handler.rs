use crate::comm::{Request, Response};

pub async fn handle_mb_subscribe(req: &Request) -> String {
    let folder = req.data.get("folder").and_then(|v| v.as_str()).unwrap_or("unknown");
    Response::ok("MB_SUBSCRIBE_OK")
        .with_msg(&format!("Subscribed to notifications for folder: {}", folder))
        .to_json()
}
