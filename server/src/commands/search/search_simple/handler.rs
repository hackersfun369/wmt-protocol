use std::time::SystemTime;
use chrono::{DateTime, Utc};
use mongodb::bson::oid::ObjectId;
use serde::Serialize;

use crate::comm::{Request, Response};
use crate::session::SessionStore;
use crate::commands::mailbox::db::{MailboxRepository, Message};

#[derive(Serialize)]
struct SearchMsgItem {
    id: String,
    from: String,
    to: Vec<String>,
    subject: String,
    snippet: Option<String>,
    received_at: String,
    unread: bool,
    starred: bool,
    important: bool,
}

pub async fn handle_search_simple(
    token: &str,
    sessions: &SessionStore,
    mailbox_repo: &MailboxRepository,
    req: &Request,
) -> String {
    // session_token
    if token.is_empty() {
        return Response::err("SEARCH", "Missing session token").to_json();
    }

    // resolve session
    let session = {
        let store = sessions.lock().unwrap();
        store.get(token).cloned()
    };

    let session = match session {
        Some(s) => s,
        None => return Response::err("SEARCH", "Invalid or expired session").to_json(),
    };

    let user_id: ObjectId = match &session.user_id {
        Some(id) => id.clone(),
        None => return Response::err("SEARCH", "User not authenticated").to_json(),
    };

    if !session.authenticated {
        return Response::err("SEARCH", "User not authenticated").to_json();
    }

    // folder_code
    let folder_code = match req.data.get("folder_code").and_then(|v| v.as_str()) {
        Some(c) if !c.is_empty() => c,
        _ => return Response::err("SEARCH", "Missing folder_code").to_json(),
    };

    // q and from
    let q = req.data.get("q").and_then(|v| v.as_str());
    let from_filter = req.data.get("from").and_then(|v| v.as_str());

    // paging
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

    // run search
    let messages: Vec<Message> = match mailbox_repo
        .search_messages_simple(&user_id, folder_code, q, from_filter, offset, limit)
        .await
    {
        Ok(ms) => ms,
        Err(e) => {
            eprintln!("[SEARCH] DB error: {:?}", e);
            return Response::err("SEARCH", "Database error").to_json();
        }
    };

    // map to DTOs
    let items: Vec<SearchMsgItem> = messages
        .into_iter()
        .map(|m| {
            let st: SystemTime = m.received_at.to_system_time();
            let dt: DateTime<Utc> = st.into();
            SearchMsgItem {
                id: m.id.to_hex(),
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
        "cmd": "SEARCH",
        "status": "OK",
        "msg": "Search results",
        "folder_code": folder_code,
        "offset": offset,
        "limit": limit,
        "messages": items,
    });

    json.to_string()
}
