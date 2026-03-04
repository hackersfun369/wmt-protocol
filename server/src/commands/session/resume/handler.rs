use crate::comm::{Request, Response};
use crate::session::SessionStore;
use crate::token::verify_jwt;

pub async fn handle_resume(
    req: &Request,
    sessions: &SessionStore,
) -> String {
    // 1) Get token from request (may be a JWT or a raw session token)
    let raw_token = match req.data.get("token").and_then(|v| v.as_str()) {
        Some(t) if !t.trim().is_empty() => t.trim().to_string(),
        _ => {
            return Response::err("RESUME", "Missing or empty token").to_json();
        }
    };

    // 2) Try to extract inner session_token from JWT; fall back to raw token
    let session_token = if let Some(claims) = verify_jwt(&raw_token) {
        claims.session_token
    } else {
        // Raw token (legacy / non-JWT) — use as-is
        raw_token.clone()
    };

    // 3) Look up session
    let session_opt = {
        let store = sessions.lock().unwrap();
        store.get(&session_token).cloned()
    };

    let Some(sess) = session_opt else {
        return Response::err("RESUME", "Session not found or expired").to_json();
    };

    // 4) Re-issue a fresh JWT wrapping the same session token
    let jwt = crate::token::issue_jwt(
        sess.email.as_deref().unwrap_or(""),
        &sess.token,
    );

    Response::ok("SESSION_RESUMED")
        .with_token(jwt)
        .with_auth(sess.authenticated)
        .with_email(sess.email.clone())
        .with_msg("Session resumed")
        .to_json()
}
