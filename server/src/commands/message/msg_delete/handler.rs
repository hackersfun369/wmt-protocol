use mongodb::bson::oid::ObjectId;

use crate::comm::{Request, Response};
use crate::session::SessionStore;
use crate::commands::mailbox::db::MailboxRepository;

pub async fn handle_msg_delete(
    token: &str,
    sessions: &SessionStore,
    mailbox_repo: &MailboxRepository,
    req: &Request,
) -> String {
    // session_token
    if token.is_empty() {
        return Response::err("MSG_DELETE", "Missing session token").to_json();
    }

    // resolve session
    let session = {
        let store = sessions.lock().unwrap();
        store.get(token).cloned()
    };

    let session = match session {
        Some(s) => s,
        None => return Response::err("MSG_DELETE", "Invalid or expired session").to_json(),
    };

    let user_id: ObjectId = match &session.user_id {
        Some(id) => id.clone(),
        None => return Response::err("MSG_DELETE", "User not authenticated").to_json(),
    };

    if !session.authenticated {
        return Response::err("MSG_DELETE", "User not authenticated").to_json();
    }

    // id
    let id_str = match req.data.get("id").and_then(|v| v.as_str()) {
        Some(s) if !s.is_empty() => s,
        _ => return Response::err("MSG_DELETE", "Missing 'id'").to_json(),
    };

    let msg_id = match ObjectId::parse_str(id_str) {
        Ok(oid) => oid,
        Err(_) => return Response::err("MSG_DELETE", "Invalid 'id' format").to_json(),
    };

    // move to BIN
    let from_folder_opt = match mailbox_repo
        .soft_delete_message_for_user(&user_id, &msg_id)
        .await
    {
        Ok(v) => v,
        Err(e) => {
            eprintln!("[MSG_DELETE] DB error: {:?}", e);
            return Response::err("MSG_DELETE", "Database error").to_json();
        }
    };

    let from_folder = match from_folder_opt {
        Some(f) => f,
        None => return Response::err("MSG_DELETE", "Message not found").to_json(),
    };

    let data = serde_json::json!({
        "id": id_str,
        "from_folder": from_folder,
        "to_folder": "BIN",
    });

    Response::ok("MSG_DELETE")
        .with_msg("Message moved to Bin")
        .with_data(data)
        .to_json()
}
