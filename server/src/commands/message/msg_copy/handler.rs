use mongodb::bson::oid::ObjectId;

use crate::comm::{Request, Response};
use crate::session::SessionStore;
use crate::commands::mailbox::db::MailboxRepository;

pub async fn handle_msg_copy(
    token: &str,
    sessions: &SessionStore,
    mailbox_repo: &MailboxRepository,
    req: &Request,
) -> String {
    // 1) session_token
    if token.is_empty() {
        return Response::err("MSG_COPY", "Missing session token").to_json();
    }

    // 2) resolve session
    let session = {
        let store = sessions.lock().unwrap();
        store.get(token).cloned()
    };

    let session = match session {
        Some(s) => s,
        None => return Response::err("MSG_COPY", "Invalid or expired session").to_json(),
    };

    let user_id: ObjectId = match &session.user_id {
        Some(id) => id.clone(),
        None => return Response::err("MSG_COPY", "User not authenticated").to_json(),
    };

    if !session.authenticated {
        return Response::err("MSG_COPY", "User not authenticated").to_json();
    }

    // 3) source id
    let id_str = match req.data.get("id").and_then(|v| v.as_str()) {
        Some(s) if !s.is_empty() => s,
        _ => return Response::err("MSG_COPY", "Missing 'id'").to_json(),
    };

    let msg_id = match ObjectId::parse_str(id_str) {
        Ok(oid) => oid,
        Err(_) => return Response::err("MSG_COPY", "Invalid 'id' format").to_json(),
    };

    // 4) target_folder
    let target_folder = match req.data.get("target_folder").and_then(|v| v.as_str()) {
        Some(s) if !s.is_empty() => s.to_uppercase(),
        _ => return Response::err("MSG_COPY", "Missing 'target_folder'").to_json(),
    };

    // 5) copy
    let result = match mailbox_repo
        .copy_message_for_user(&user_id, &msg_id, &target_folder)
        .await
    {
        Ok(r) => r,
        Err(e) => {
            eprintln!("[MSG_COPY] DB error: {:?}", e);
            return Response::err("MSG_COPY", "Database error").to_json();
        }
    };

    let (source_folder, new_id) = match result {
        Some(v) => v,
        None => return Response::err("MSG_COPY", "Message not found").to_json(),
    };

    let data = serde_json::json!({
        "source_id": id_str,
        "source_folder": source_folder,
        "new_id": new_id.to_hex(),
        "target_folder": target_folder,
    });

    Response::ok("MSG_COPY")
        .with_msg("Message copied")
        .with_data(data)
        .to_json()
}
