use crate::comm::{Request, Response};
use crate::session::SessionStore;

pub async fn handle_session_suspend(
    token: &str,
    sessions: &SessionStore,
    _req: &Request,
) -> String {
    // 1) Token required
    if token.is_empty() {
        return Response::err("SESSION_SUSPEND", "Missing token").to_json();
    }

    // 2) Mark suspended
    let mut found = false;
    {
        let mut store = sessions.lock().unwrap();
        if let Some(sess) = store.get_mut(token) {
            sess.suspended = true;
            sess.authenticated = false;
            found = true;
        }
    }

    if !found {
        return Response::err("SESSION_SUSPEND", "Session not found").to_json();
    }

    // 3) Reply OK
    Response::ok("SESSION_SUSPEND")
        .with_token(token.to_string())
        .with_auth(false)
        .with_msg("Session suspended")
        .to_json()
}
