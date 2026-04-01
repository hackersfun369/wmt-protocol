use crate::comm::{Request, Response};
use crate::session::SessionStore;

pub async fn handle_logout(
    req: &Request,
    sessions: &SessionStore,
) -> String {
    // 1) Get token — accept either "session_token" or "token" for flexibility
    let token = match req.data.get("session_token")
        .or_else(|| req.data.get("token"))
        .and_then(|v| v.as_str())
    {
        Some(t) if !t.trim().is_empty() => t.trim().to_string(),
        _ => {
            return Response::err("LOGOUT", "Missing or empty token").to_json();
        }
    };

    // 2) Remove from in-memory store
    {
        let mut store = sessions.lock().unwrap();
        store.remove(&token);
    }

    // 3) Reply OK
    Response::ok("LOGOUT_OK")
        .with_msg("Logged out")
        .to_json()
}
