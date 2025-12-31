use crate::comm::{Request, Response};
use crate::session::SessionStore;

pub async fn handle_session_info(
    token: &str,
    sessions: &SessionStore,
    _req: &Request,
) -> String {
    if token.is_empty() {
        return Response::err("SESSION_INFO", "Missing token").to_json();
    }

    let sess_opt = {
        let store = sessions.lock().unwrap();
        store.get(token).cloned()
    };

    let Some(sess) = sess_opt else {
        return Response::err("SESSION_INFO", "Session not found").to_json();
    };

    Response::ok("SESSION_INFO")
        .with_token(sess.token)
        .with_auth(sess.authenticated)
        .with_email(sess.email.clone())
        .with_msg("Session info")
        .to_json()
}
