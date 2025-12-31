use std::time::SystemTime;
use chrono::{DateTime, Utc};
use mongodb::bson::oid::ObjectId;
use serde::Serialize;

use crate::comm::{Request, Response};
use crate::session::SessionStore;
use crate::commands::mailbox::db::{MailboxRepository, Message};

#[derive(Serialize)]
struct MsgHeadersDto {
    id: String,
    folder_code: String,
    from: String,
    to: Vec<String>,
    subject: String,
    snippet: Option<String>,
    received_at: String,
    unread: bool,
}

pub async fn handle_msg_headers(
    token: &str,
    sessions: &SessionStore,
    mailbox_repo: &MailboxRepository,
    req: &Request,
) -> String {
    if token.is_empty() {
        return Response::err("MSG_HEADERS", "Missing session token").to_json();
    }

    // 2) resolve session
    let session = {
        let store = sessions.lock().unwrap();
        store.get(token).cloned()
    };

    let session = match session {
        Some(s) => s,
        None => return Response::err("MSG_HEADERS", "Invalid or expired session").to_json(),
    };

    let user_id: ObjectId = match &session.user_id {
        Some(id) => id.clone(),
        None => return Response::err("MSG_HEADERS", "User not authenticated").to_json(),
    };

    if !session.authenticated {
        return Response::err("MSG_HEADERS", "User not authenticated").to_json();
    }

    // 3) parse message id
    let id_str = match req.data.get("id").and_then(|v| v.as_str()) {
        Some(s) if !s.is_empty() => s,
        _ => return Response::err("MSG_HEADERS", "Missing 'id'").to_json(),
    };

    let msg_id = match ObjectId::parse_str(id_str) {
        Ok(oid) => oid,
        Err(_) => return Response::err("MSG_HEADERS", "Invalid 'id' format").to_json(),
    };

    // 4) query Mongo
    let msg_opt: Option<Message> = match mailbox_repo
        .get_message_headers_for_user(&user_id, &msg_id)
        .await
    {
        Ok(m) => m,
        Err(e) => {
            eprintln!("[MSG_HEADERS] DB error: {:?}", e);
            return Response::err("MSG_HEADERS", "Database error").to_json();
        }
    };

    let msg: Message = match msg_opt {
        Some(m) => m,
        None => return Response::err("MSG_HEADERS", "Message not found").to_json(),
    };

    // 5) map to DTO (no body)
    let st: SystemTime = msg.received_at.to_system_time();
    let dt: DateTime<Utc> = st.into();

    let dto = MsgHeadersDto {
        id: msg.id.to_hex(),
        folder_code: msg.folder_code,
        from: msg.from,
        to: msg.to,
        subject: msg.subject,
        snippet: msg.snippet,
        received_at: dt.to_rfc3339(),
        unread: msg.unread,
    };

    let json = serde_json::json!({
        "cmd": "MSG_HEADERS",
        "status": "OK",
        "msg": "Message headers",
        "headers": dto,
    });

    json.to_string()
}
