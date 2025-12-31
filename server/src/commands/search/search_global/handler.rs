use std::time::SystemTime;
use chrono::{DateTime, Utc};
use mongodb::bson::oid::ObjectId;
use serde::Serialize;

use crate::comm::{Request, Response};
use crate::session::SessionStore;
use crate::commands::mailbox::db::{MailboxRepository, Message};

#[derive(Serialize)]
struct GlobalSearchMsgItem {
    id: String,
    folder_code: String,
    from: String,
    to: Vec<String>,
    subject: String,
    snippet: Option<String>,
    received_at: String,
    unread: bool,
    starred: bool,
    important: bool,
}

pub async fn handle_search_global(
    token: &str,
    sessions: &SessionStore,
    mailbox_repo: &MailboxRepository,
    req: &Request,
) -> String {
    if token.is_empty() {
        return Response::err("SEARCH_GLOBAL", "Missing session token").to_json();
    }

    let session = {
        let store = sessions.lock().unwrap();
        store.get(token).cloned()
    };

    let session = match session {
        Some(s) => s,
        None => return Response::err("SEARCH_GLOBAL", "Invalid or expired session").to_json(),
    };

    let user_id: ObjectId = match &session.user_id {
        Some(id) => id.clone(),
        None => return Response::err("SEARCH_GLOBAL", "User not authenticated").to_json(),
    };

    if !session.authenticated {
        return Response::err("SEARCH_GLOBAL", "User not authenticated").to_json();
    }

    let q = req.data.get("q").and_then(|v| v.as_str());
    let from_filter = req.data.get("from").and_then(|v| v.as_str());

    let offset = req
        .data
        .get("offset")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);

    let limit = req
        .data
        .get("limit")
        .and_then(|v| v.as_u64())
        .unwrap_or(50);

    let messages: Vec<Message> = match mailbox_repo
        .search_messages_global_simple(&user_id, q, from_filter, offset, limit)
        .await
    {
        Ok(ms) => ms,
        Err(e) => {
            eprintln!("[SEARCH_GLOBAL] DB error: {:?}", e);
            return Response::err("SEARCH_GLOBAL", "Database error").to_json();
        }
    };

    let items: Vec<GlobalSearchMsgItem> = messages
        .into_iter()
        .map(|m| {
            let st: SystemTime = m.received_at.to_system_time();
            let dt: DateTime<Utc> = st.into();
            GlobalSearchMsgItem {
                id: m.id.to_hex(),
                folder_code: m.folder_code,
                from: m.from,
                to: m.to,
                subject: m.subject,
                snippet: m.snippet,
                received_at: dt.to_rfc3339(),
                unread: m.unread,
                starred: m.starred,
                important: m.important,
            }
        })
        .collect();

    let json = serde_json::json!({
        "cmd": "SEARCH_GLOBAL",
        "status": "OK",
        "msg": "Global search results",
        "offset": offset,
        "limit": limit,
        "messages": items,
    });

    json.to_string()
}
