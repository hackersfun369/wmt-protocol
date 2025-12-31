use crate::comm::{Request, Response};

pub async fn handle_hb(_req: &Request) -> String {
    Response::ok("HB")
        .with_msg("Heartbeat acknowledged")
        .to_json()
}
