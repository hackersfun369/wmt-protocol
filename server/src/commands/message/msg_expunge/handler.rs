use mongodb::bson::oid::ObjectId;

use crate::comm::{Request, Response};
use crate::session::SessionStore;
use crate::commands::mailbox::db::MailboxRepository;

pub async fn handle_msg_expunge(
    token: &str,
    sessions: &SessionStore,
    mailbox_repo: &MailboxRepository,
    req: &Request,
) -> String {
    // session_token
    if token.is_empty() {
        return Response::err("MSG_EXPUNGE", "Missing session token").to_json();
    }

    // resolve session
    let session = {
        let store = sessions.lock().unwrap();
        store.get(token).cloned()
    };

    let session = match session {
        Some(s) => s,
        None => return Response::err("MSG_EXPUNGE", "Invalid or expired session").to_json(),
    };

    let user_id: ObjectId = match &session.user_id {
        Some(id) => id.clone(),
        None => return Response::err("MSG_EXPUNGE", "User not authenticated").to_json(),
    };

    if !session.authenticated {
        return Response::err("MSG_EXPUNGE", "User not authenticated").to_json();
    }

    // id
    let id_str = match req.data.get("id").and_then(|v| v.as_str()) {
        Some(s) if !s.is_empty() => s,
        _ => return Response::err("MSG_EXPUNGE", "Missing 'id'").to_json(),
    };

    let msg_id = match ObjectId::parse_str(id_str) {
        Ok(oid) => oid,
        Err(_) => return Response::err("MSG_EXPUNGE", "Invalid 'id' format").to_json(),
    };

    // hard delete
    let deleted = match mailbox_repo
        .hard_delete_message_for_user(&user_id, &msg_id)
        .await
    {
        Ok(n) => n,
        Err(e) => {
            eprintln!("[MSG_EXPUNGE] DB error: {:?}", e);
            return Response::err("MSG_EXPUNGE", "Database error").to_json();
        }
    };

    if deleted == 0 {
        return Response::err("MSG_EXPUNGE", "Message not found").to_json();
    }

    let data = serde_json::json!({
        "id": id_str
    });

    Response::ok("MSG_EXPUNGE")
        .with_msg("Message permanently deleted")
        .with_data(data)
        .to_json()
}
