use crate::comm::{Request, Response};
use crate::session::SessionStore;

pub async fn handle_session_resume_suspended(
    token: &str,
    sessions: &SessionStore,
    _req: &Request,
) -> String {
    // 1) Token required
    if token.is_empty() {
        return Response::err("SESSION_RESUME_SUSPENDED", "Missing token").to_json();
    }

    // 2) Find session and check suspended flag
    let mut result = None;
    {
        let mut store = sessions.lock().unwrap();
        if let Some(sess) = store.get_mut(token) {
            if !sess.suspended {
                result = Some(Err("Session is not suspended".to_string()));
            } else {
                sess.suspended = false;
                sess.authenticated = true;
                result = Some(Ok(()));
            }
        } else {
            result = None;
        }
    }

    match result {
        None => Response::err("SESSION_RESUME_SUSPENDED", "Session not found").to_json(),
        Some(Err(msg)) => Response::err("SESSION_RESUME_SUSPENDED", &msg).to_json(),
        Some(Ok(())) => Response::ok("SESSION_RESUME_SUSPENDED")
            .with_token(token.to_string())
            .with_auth(true)
            .with_msg("Session resumed from suspended state")
            .to_json(),
    }
}
