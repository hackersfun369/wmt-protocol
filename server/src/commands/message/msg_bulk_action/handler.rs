use mongodb::bson::oid::ObjectId;

use crate::comm::{Request, Response};
use crate::session::SessionStore;
use crate::commands::mailbox::db::MailboxRepository;

pub async fn handle_msg_bulk_action(
    token: &str,
    sessions: &SessionStore,
    mailbox_repo: &MailboxRepository,
    req: &Request,
) -> String {
    // 1) session_token
    if token.is_empty() {
        return Response::err("MSG_BULK_ACTION", "Missing session token").to_json();
    }

    // 2) resolve session
    let session = {
        let store = sessions.lock().unwrap();
        store.get(token).cloned()
    };

    let session = match session {
        Some(s) => s,
        None => return Response::err("MSG_BULK_ACTION", "Invalid or expired session").to_json(),
    };

    let user_id = match &session.user_id {
        Some(id) => id.clone(),
        None => return Response::err("MSG_BULK_ACTION", "User not authenticated").to_json(),
    };

    if !session.authenticated {
        return Response::err("MSG_BULK_ACTION", "User not authenticated").to_json();
    }

    // 3) action
    let action = match req.data.get("action").and_then(|v| v.as_str()) {
        Some(a) if !a.is_empty() => a.to_uppercase(),
        _ => return Response::err("MSG_BULK_ACTION", "Missing 'action'").to_json(),
    };

    // 4) ids
    let ids_val = match req.data.get("ids") {
        Some(v) if v.is_array() => v.as_array().unwrap(),
        _ => return Response::err("MSG_BULK_ACTION", "Missing or invalid 'ids'").to_json(),
    };

    let mut ids: Vec<ObjectId> = Vec::new();
    for v in ids_val {
        if let Some(s) = v.as_str() {
            if let Ok(oid) = ObjectId::parse_str(s) {
                ids.push(oid);
            }
        }
    }
    if ids.is_empty() {
        return Response::err("MSG_BULK_ACTION", "No valid ids provided").to_json();
    }

    let total_ids = ids.len() as u64;

    // 5) branch by action
    let affected = match action.as_str() {
        "MOVE" => {
            let target_folder = match req.data.get("target_folder").and_then(|v| v.as_str()) {
                Some(s) if !s.is_empty() => s.to_uppercase(),
                _ => return Response::err("MSG_BULK_ACTION", "Missing 'target_folder'").to_json(),
            };
            match mailbox_repo.bulk_move_for_user(&user_id, &ids, &target_folder).await {
                Ok(n) => n,
                Err(e) => {
                    eprintln!("[MSG_BULK_ACTION MOVE] DB error: {:?}", e);
                    return Response::err("MSG_BULK_ACTION", "Database error").to_json();
                }
            }
        }
        "DELETE" => {
            match mailbox_repo.bulk_soft_delete_for_user(&user_id, &ids).await {
                Ok(n) => n,
                Err(e) => {
                    eprintln!("[MSG_BULK_ACTION DELETE] DB error: {:?}", e);
                    return Response::err("MSG_BULK_ACTION", "Database error").to_json();
                }
            }
        }
        "EXPUNGE" => {
            match mailbox_repo.bulk_expunge_for_user(&user_id, &ids).await {
                Ok(n) => n,
                Err(e) => {
                    eprintln!("[MSG_BULK_ACTION EXPUNGE] DB error: {:?}", e);
                    return Response::err("MSG_BULK_ACTION", "Database error").to_json();
                }
            }
        }
        "FLAG_SET" | "FLAG_CLEAR" => {
            let flags = req.data.get("flags").cloned().unwrap_or_else(|| serde_json::json!({}));

            let read = flags.get("read").and_then(|v| v.as_bool());
            let starred = flags.get("starred").and_then(|v| v.as_bool());
            let important = flags.get("important").and_then(|v| v.as_bool());

            if read.is_none() && starred.is_none() && important.is_none() {
                return Response::err("MSG_BULK_ACTION", "No flags provided").to_json();
            }

            let (read_eff, starred_eff, important_eff) = if action == "FLAG_CLEAR" {
                (
                    read.map(|_| false),
                    starred.map(|_| false),
                    important.map(|_| false),
                )
            } else {
                (read, starred, important)
            };

            match mailbox_repo
                .bulk_set_flags_for_user(&user_id, &ids, read_eff, starred_eff, important_eff)
                .await
            {
                Ok(n) => n,
                Err(e) => {
                    eprintln!("[MSG_BULK_ACTION FLAG] DB error: {:?}", e);
                    return Response::err("MSG_BULK_ACTION", "Database error").to_json();
                }
            }
        }
        _ => {
            return Response::err("MSG_BULK_ACTION", "Unsupported action").to_json();
        }
    };

    let data = serde_json::json!({
        "action": action,
        "total_ids": total_ids,
        "affected": affected,
    });

    Response::ok("MSG_BULK_ACTION")
        .with_msg("Bulk action applied")
        .with_data(data)
        .to_json()
}
