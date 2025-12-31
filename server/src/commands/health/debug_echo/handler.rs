use crate::comm::{Request, Response};

pub async fn handle_debug_echo(req: &Request) -> String {
    Response::ok("DEBUG_ECHO")
        .with_data(req.data.clone())
        .to_json()
}
