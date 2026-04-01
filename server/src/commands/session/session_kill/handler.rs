use crate::comm::{Request, Response};
use crate::session::SessionStore;

pub async fn handle_session_kill(
    req: &Request,
    sessions: &SessionStore,
) -> String {
    // 1) Validate token — accept session_token or token (both fields)
    let token = req.data.get("session_token")
        .or_else(|| req.data.get("token"))
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|t| !t.is_empty())
        .map(str::to_string);

    let token = match token {
        Some(t) => t,
        None => return Response::err("SESSION_KILL", "Missing or empty token").to_json(),
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
