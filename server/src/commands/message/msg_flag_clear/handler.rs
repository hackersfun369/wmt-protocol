use mongodb::bson::oid::ObjectId;

use crate::comm::{Request, Response};
use crate::session::SessionStore;
use crate::commands::mailbox::db::MailboxRepository;

pub async fn handle_msg_flag_clear(
    token: &str,
    sessions: &SessionStore,
    mailbox_repo: &MailboxRepository,
    req: &Request,
) -> String {
    // 1) session_token
    if token.is_empty() {
        return Response::err("MSG_FLAG_CLEAR", "Missing session token").to_json();
    }

    // 2) resolve session
    let session = {
        let store = sessions.lock().unwrap();
        store.get(token).cloned()
    };

    let session = match session {
        Some(s) => s,
        None => return Response::err("MSG_FLAG_CLEAR", "Invalid or expired session").to_json(),
    };

    let user_id: ObjectId = match &session.user_id {
        Some(id) => id.clone(),
        None => return Response::err("MSG_FLAG_CLEAR", "User not authenticated").to_json(),
    };

    if !session.authenticated {
        return Response::err("MSG_FLAG_CLEAR", "User not authenticated").to_json();
    }

    // 3) message id
    let id_str = match req.data.get("id").and_then(|v| v.as_str()) {
        Some(s) if !s.is_empty() => s,
        _ => return Response::err("MSG_FLAG_CLEAR", "Missing 'id'").to_json(),
    };

    let msg_id = match ObjectId::parse_str(id_str) {
        Ok(oid) => oid,
        Err(_) => return Response::err("MSG_FLAG_CLEAR", "Invalid 'id' format").to_json(),
    };

    // 4) which flags to clear
    let clear_read = req.data.get("read").and_then(|v| v.as_bool()).unwrap_or(false);
    let clear_starred = req.data.get("starred").and_then(|v| v.as_bool()).unwrap_or(false);
    let clear_important = req.data.get("important").and_then(|v| v.as_bool()).unwrap_or(false);

    if !clear_read && !clear_starred && !clear_important {
        return Response::err("MSG_FLAG_CLEAR", "No flags specified to clear").to_json();
    }

    // For read: clearing means mark as unread => unread = true internally
    let new_read = if clear_read { Some(false) } else { None };
    let new_starred = if clear_starred { Some(false) } else { None };
    let new_important = if clear_important { Some(false) } else { None };

    // 5) update via same repo method as MSG_FLAG_SET
    let modified = match mailbox_repo
        .set_flags_for_message(&user_id, &msg_id, new_read, new_starred, new_important)
        .await
    {
        Ok(n) => n,
        Err(e) => {
            eprintln!("[MSG_FLAG_CLEAR] DB error: {:?}", e);
            return Response::err("MSG_FLAG_CLEAR", "Database error").to_json();
        }
    };

    if modified == 0 {
        return Response::err("MSG_FLAG_CLEAR", "Message not found").to_json();
    }

    let data = serde_json::json!({
        "id": id_str,
        "flags": {
            "read": new_read,
            "starred": new_starred,
            "important": new_important
        }
    });

    Response::ok("MSG_FLAG_CLEAR")
        .with_msg("Flags cleared")
        .with_data(data)
        .to_json()
}
