use crate::comm::{Request, Response};

pub async fn handle_mb_unsubscribe(req: &Request) -> String {
    let folder = req.data.get("folder").and_then(|v| v.as_str()).unwrap_or("unknown");
    Response::ok("MB_UNSUBSCRIBE_OK")
        .with_msg(&format!("Unsubscribed from notifications for folder: {}", folder))
        .to_json()
}
