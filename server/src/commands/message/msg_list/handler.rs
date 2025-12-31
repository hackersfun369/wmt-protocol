use std::time::SystemTime;
use chrono::{DateTime, Utc};
use mongodb::bson::oid::ObjectId;
use serde::Serialize;

use crate::comm::{Request, Response};
use crate::session::SessionStore;
use crate::commands::mailbox::db::{MailboxRepository, Message};

#[derive(Serialize)]
struct MsgListItem {
    id: String,
    from: String,
    to: Vec<String>,
    subject: String,
    snippet: Option<String>,
    received_at: String,
    unread: bool,
}

pub async fn handle_msg_list(
    token: &str,
    sessions: &SessionStore,
    mailbox_repo: &MailboxRepository,
    req: &Request,
) -> String {
    if token.is_empty() {
        return Response::err("MSG_LIST", "Missing session token").to_json();
    }

    // 2) resolve session
    let session = {
        let store = sessions.lock().unwrap();
        store.get(token).cloned()
    };

    let session = match session {
        Some(s) => s,
        None => return Response::err("MSG_LIST", "Invalid or expired session").to_json(),
    };

    let user_id: ObjectId = match &session.user_id {
        Some(id) => id.clone(),
        None => return Response::err("MSG_LIST", "User not authenticated").to_json(),
    };

    if !session.authenticated {
        return Response::err("MSG_LIST", "User not authenticated").to_json();
    }

    // 3) folder_code, offset, limit
    let folder_code = match req.data.get("folder_code").and_then(|v| v.as_str()) {
        Some(c) if !c.is_empty() => c,
        _ => return Response::err("MSG_LIST", "Missing folder_code").to_json(),
    };

    let offset = req
        .data
        .get("offset")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);

    let limit = req
        .data
        .get("limit")
        .and_then(|v| v.as_u64())
        .unwrap_or(20);

    // 4) query Mongo via list_messages
    let messages: Vec<Message> = match mailbox_repo
        .list_messages(&user_id, folder_code, offset, limit)
        .await
    {
        Ok(ms) => ms,
        Err(e) => {
            eprintln!("[MSG_LIST] DB error: {:?}", e);
            return Response::err("MSG_LIST", "Database error").to_json();
        }
    };

    // 5) map to DTOs
    let items: Vec<MsgListItem> = messages
        .into_iter()
        .map(|m| {
            let st: SystemTime = m.received_at.to_system_time();
            let dt: DateTime<Utc> = st.into();
            MsgListItem {
                id: m.id.to_hex(),
                from: m.from,
                to: m.to,
                subject: m.subject,
                snippet: m.snippet,
                received_at: dt.to_rfc3339(),
                unread: m.unread,
            }
        })
        .collect();

    let json = serde_json::json!({
        "cmd": "MSG_LIST",
        "status": "OK",
        "msg": "Messages",
        "folder_code": folder_code,
        "offset": offset,
        "limit": limit,
        "messages": items,
    });

    json.to_string()
}
