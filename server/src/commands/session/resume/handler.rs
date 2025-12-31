use crate::comm::{Request, Response};
use crate::session::SessionStore;

pub async fn handle_resume(
    req: &Request,
    sessions: &SessionStore,
) -> String {
    // 1) Get token from request
    let token = match req.data.get("token").and_then(|v| v.as_str()) {
        Some(t) if !t.trim().is_empty() => t.trim().to_string(),
        _ => {
            return Response::err("RESUME", "Missing or empty token").to_json();
        }
    };

    // 2) Look up session
    let session_opt = {
        let store = sessions.lock().unwrap();
        store.get(&token).cloned()
    };

    let Some(sess) = session_opt else {
        return Response::err("RESUME", "Session not found").to_json();
    };

    // 3) Build response – treat as resumed & authenticated if session says so
    Response::ok("SESSION_RESUMED")
        .with_token(sess.token)
        .with_auth(sess.authenticated)
        .with_msg("Session resumed")
        .to_json()
}
