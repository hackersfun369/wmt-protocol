use crate::comm::{Request, Response};
use crate::commands::mailbox::db::MailboxRepository;
use crate::session::SessionStore;
use serde_json::json;

pub async fn handle_msg_thread(
    token: &str,
    sessions: &SessionStore,
    mailbox_repo: &MailboxRepository,
    req: &Request,
) -> String {
    if token.is_empty() {
        return Response::err("MSG_THREAD", "Missing session token").to_json();
    }

    let user_id = {
        let store = sessions.lock().unwrap();
        match store.get(token).and_then(|s| s.user_id.clone()) {
            Some(id) => id,
            None => return Response::err("MSG_THREAD", "Invalid session or not authenticated").to_json(),
        }
    };

    let msg_id = match req.data.get("msg_id").and_then(|v| v.as_str()) {
        Some(id) => id,
        None => return Response::err("MSG_THREAD", "Message ID is required").to_json(),
    };

    // For now, we returns just the message itself in the thread list
    // In a real system, we'd search for parent_id or thread_id.
    match mailbox_repo.get_message_for_user(&user_id, &msg_id.parse().unwrap_or_default()).await {
        Ok(Some(msg)) => {
            Response::ok("MSG_THREAD_OK")
                .with_data(json!({
                    "thread_id": msg_id,
                    "messages": [msg]
                }))
                .to_json()
        }
        _ => Response::err("MSG_THREAD", "Message not found").to_json(),
    }
}
