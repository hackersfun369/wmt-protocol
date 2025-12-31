use crate::comm::{Request, Response};

pub async fn handle_pong(_req: &Request) -> String {
    Response::ok("PONG")
        .with_msg("Server received PING, replying with PONG")
        .to_json()
}
