// src/commands/session/init/handler.rs
use crate::comm::{Request, Response};
use crate::session::{SessionStore, WmtpSession};
use crate::token::generate_ephemeral_token;

pub async fn handle_init(
    _req: &Request,
    sessions: &SessionStore,
) -> String {
    let token = generate_ephemeral_token();

    {
        let mut store = sessions.lock().unwrap();
        store.insert(token.clone(), WmtpSession::new_ephemeral(token.clone()));
    }

    Response::ok("SESSION_INIT")
        .with_token(token)
        .with_auth(false)
        .with_msg("Session initialized")
        .to_json()
}
