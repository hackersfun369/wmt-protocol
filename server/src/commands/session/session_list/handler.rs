use crate::comm::{Request, Response, SessionSummary};
use crate::session::SessionStore;

pub async fn handle_session_list(
    _req: &Request,
    sessions: &SessionStore,
) -> String {
    let list: Vec<SessionSummary> = {
        let store = sessions.lock().unwrap();
        store
            .values()
            .map(|s| SessionSummary {
                session_token: s.token.clone(),
                auth: s.authenticated,
                email: s.email.clone(),
            })
            .collect()
    };

    Response::ok("SESSION_LIST")
        .with_msg("Active sessions")
        .with_sessions(list)
        .to_json()
}
