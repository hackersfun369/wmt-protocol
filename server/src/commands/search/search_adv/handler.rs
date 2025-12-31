use std::time::SystemTime;
use chrono::{DateTime, Utc};
use mongodb::bson::{oid::ObjectId, DateTime as BsonDateTime};
use serde::Serialize;

use crate::comm::{Request, Response};
use crate::session::SessionStore;
use crate::commands::mailbox::db::{MailboxRepository, Message};

#[derive(Serialize)]
struct AdvSearchMsgItem {
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

pub async fn handle_search_adv(
    token: &str,
    sessions: &SessionStore,
    mailbox_repo: &MailboxRepository,
    req: &Request,
) -> String {
    // session_token
    if token.is_empty() {
        return Response::err("SEARCH_ADV", "Missing session token").to_json();
    }

    // resolve session
    let session = {
        let store = sessions.lock().unwrap();
        store.get(token).cloned()
    };

    let session = match session {
        Some(s) => s,
        None => return Response::err("SEARCH_ADV", "Invalid or expired session").to_json(),
    };

    let user_id: ObjectId = match &session.user_id {
        Some(id) => id.clone(),
        None => return Response::err("SEARCH_ADV", "User not authenticated").to_json(),
    };

    if !session.authenticated {
        return Response::err("SEARCH_ADV", "User not authenticated").to_json();
    }

    // filters
    let folder_code = req.data.get("folder_code").and_then(|v| v.as_str());
    let q = req.data.get("q").and_then(|v| v.as_str());
    let from_filter = req.data.get("from").and_then(|v| v.as_str());
    let to_filter = req.data.get("to").and_then(|v| v.as_str());

    let unread = req.data.get("unread").and_then(|v| v.as_bool());
    let starred = req.data.get("starred").and_then(|v| v.as_bool());
    let important = req.data.get("important").and_then(|v| v.as_bool());

    let date_from = req
        .data
        .get("date_from")
        .and_then(|v| v.as_str())
        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| {
            let utc: DateTime<Utc> = dt.with_timezone(&Utc);
            BsonDateTime::from_millis(utc.timestamp_millis())
        });

    let date_to = req
        .data
        .get("date_to")
        .and_then(|v| v.as_str())
        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| {
            let utc: DateTime<Utc> = dt.with_timezone(&Utc);
            BsonDateTime::from_millis(utc.timestamp_millis())
        });

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
        .search_messages_advanced(
            &user_id,
            folder_code,
            q,
            from_filter,
            to_filter,
            unread,
            starred,
            important,
            date_from,
            date_to,
            offset,
            limit,
        )
        .await
    {
        Ok(ms) => ms,
        Err(e) => {
            eprintln!("[SEARCH_ADV] DB error: {:?}", e);
            return Response::err("SEARCH_ADV", "Database error").to_json();
        }
    };

    let items: Vec<AdvSearchMsgItem> = messages
        .into_iter()
        .map(|m| {
            let st: SystemTime = m.received_at.to_system_time();
            let dt: DateTime<Utc> = st.into();
            AdvSearchMsgItem {
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
        "cmd": "SEARCH_ADV",
        "status": "OK",
        "msg": "Advanced search results",
        "offset": offset,
        "limit": limit,
        "messages": items,
    });

    json.to_string()
}
