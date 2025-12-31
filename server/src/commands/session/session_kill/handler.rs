use crate::comm::{Request, Response};
use crate::session::SessionStore;

pub async fn handle_session_kill(
    req: &Request,
    sessions: &SessionStore,
) -> String {
    // 1) Validate token
    let token = match req.data.get("token").and_then(|v| v.as_str()) {
        Some(t) if !t.trim().is_empty() => t.trim().to_string(),
        _ => {
            return Response::err("SESSION_KILL", "Missing or empty token").to_json();
        }
    };

    // 2) Remove session
    let removed = {
        let mut store = sessions.lock().unwrap();
        store.remove(&token)
    };

    if removed.is_none() {
        return Response::err("SESSION_KILL", "Session not found").to_json();
    }

    // 3) Reply OK with token, auth:false
    Response::ok("SESSION_KILL")
        .with_token(token)
        .with_auth(false)
        .with_msg("Session killed")
        .to_json()
}
