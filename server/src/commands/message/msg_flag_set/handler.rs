use mongodb::bson::oid::ObjectId;

use crate::comm::{Request, Response};
use crate::session::SessionStore;
use crate::commands::mailbox::db::MailboxRepository;

pub async fn handle_msg_flag_set(
    token: &str,
    sessions: &SessionStore,
    mailbox_repo: &MailboxRepository,
    req: &Request,
) -> String {
    // session_token
    if token.is_empty() {
        return Response::err("MSG_FLAG_SET", "Missing session token").to_json();
    }

    // resolve session
    let session = {
        let store = sessions.lock().unwrap();
        store.get(token).cloned()
    };

    let session = match session {
        Some(s) => s,
        None => return Response::err("MSG_FLAG_SET", "Invalid or expired session").to_json(),
    };

    let user_id: ObjectId = match &session.user_id {
        Some(id) => id.clone(),
        None => return Response::err("MSG_FLAG_SET", "User not authenticated").to_json(),
    };

    if !session.authenticated {
        return Response::err("MSG_FLAG_SET", "User not authenticated").to_json();
    }

    // id
    let id_str = match req.data.get("id").and_then(|v| v.as_str()) {
        Some(s) if !s.is_empty() => s,
        _ => return Response::err("MSG_FLAG_SET", "Missing 'id'").to_json(),
    };

    let msg_id = match ObjectId::parse_str(id_str) {
        Ok(oid) => oid,
        Err(_) => return Response::err("MSG_FLAG_SET", "Invalid 'id' format").to_json(),
    };

    // flags (optional)
    let read = req.data.get("read").and_then(|v| v.as_bool());
    let starred = req.data.get("starred").and_then(|v| v.as_bool());
    let important = req.data.get("important").and_then(|v| v.as_bool());

    if read.is_none() && starred.is_none() && important.is_none() {
        return Response::err("MSG_FLAG_SET", "No flags provided").to_json();
    }

    // update
    let modified = match mailbox_repo
        .set_flags_for_message(&user_id, &msg_id, read, starred, important)
        .await
    {
        Ok(n) => n,
        Err(e) => {
            eprintln!("[MSG_FLAG_SET] DB error: {:?}", e);
            return Response::err("MSG_FLAG_SET", "Database error").to_json();
        }
    };

    if modified == 0 {
        return Response::err("MSG_FLAG_SET", "Message not found").to_json();
    }

    let data = serde_json::json!({
        "id": id_str,
        "flags": {
            "read": read,
            "starred": starred,
            "important": important
        }
    });

    Response::ok("MSG_FLAG_SET")
        .with_msg("Flags updated")
        .with_data(data)
        .to_json()
}
