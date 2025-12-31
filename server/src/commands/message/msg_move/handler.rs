use mongodb::bson::oid::ObjectId;

use crate::comm::{Request, Response};
use crate::session::SessionStore;
use crate::commands::mailbox::db::MailboxRepository;

pub async fn handle_msg_move(
    token: &str,
    sessions: &SessionStore,
    mailbox_repo: &MailboxRepository,
    req: &Request,
) -> String {
    // 1) session_token
    if token.is_empty() {
        return Response::err("MSG_MOVE", "Missing session token").to_json();
    }

    // 2) resolve session
    let session = {
        let store = sessions.lock().unwrap();
        store.get(token).cloned()
    };

    let session = match session {
        Some(s) => s,
        None => return Response::err("MSG_MOVE", "Invalid or expired session").to_json(),
    };

    let user_id: ObjectId = match &session.user_id {
        Some(id) => id.clone(),
        None => return Response::err("MSG_MOVE", "User not authenticated").to_json(),
    };

    if !session.authenticated {
        return Response::err("MSG_MOVE", "User not authenticated").to_json();
    }

    // 3) parse id
    let id_str = match req.data.get("id").and_then(|v| v.as_str()) {
        Some(s) if !s.is_empty() => s,
        _ => return Response::err("MSG_MOVE", "Missing 'id'").to_json(),
    };

    let msg_id = match ObjectId::parse_str(id_str) {
        Ok(oid) => oid,
        Err(_) => return Response::err("MSG_MOVE", "Invalid 'id' format").to_json(),
    };

    // 4) target_folder
    let target_folder = match req.data.get("target_folder").and_then(|v| v.as_str()) {
        Some(s) if !s.is_empty() => s.to_uppercase(),
        _ => return Response::err("MSG_MOVE", "Missing 'target_folder'").to_json(),
    };

    // Optionally, validate against known folders or user custom folders here.

    // 5) move
    let from_folder_opt = match mailbox_repo
        .move_message_for_user(&user_id, &msg_id, &target_folder)
        .await
    {
        Ok(v) => v,
        Err(e) => {
            eprintln!("[MSG_MOVE] DB error: {:?}", e);
            return Response::err("MSG_MOVE", "Database error").to_json();
        }
    };

    let from_folder = match from_folder_opt {
        Some(f) => f,
        None => return Response::err("MSG_MOVE", "Message not found").to_json(),
    };

    let data = serde_json::json!({
        "id": id_str,
        "from_folder": from_folder,
        "to_folder": target_folder,
    });

    Response::ok("MSG_MOVE")
        .with_msg("Message moved")
        .with_data(data)
        .to_json()
}
