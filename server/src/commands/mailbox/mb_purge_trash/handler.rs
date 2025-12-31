use mongodb::bson::oid::ObjectId;
use crate::comm::{Request, Response};
use crate::session::SessionStore;
use crate::commands::mailbox::db::MailboxRepository;

pub async fn handle_mb_purge_trash(
    req: &Request,
    sessions: &SessionStore,
    mailbox_repo: &MailboxRepository,
) -> String {
    // session_token
    let token = match req.data.get("session_token").and_then(|v| v.as_str()) {
        Some(t) if !t.is_empty() => t,
        _ => return Response::err("MB_PURGE_TRASH", "Missing session_token").to_json(),
    };

    // resolve session
    let session = {
        let store = sessions.lock().unwrap();
        store.get(token).cloned()
    };

    let session = match session {
        Some(s) => s,
        None => return Response::err("MB_PURGE_TRASH", "Invalid or expired session").to_json(),
    };

    let user_id: ObjectId = match &session.user_id {
        Some(id) => id.clone(),
        None => return Response::err("MB_PURGE_TRASH", "User not authenticated").to_json(),
    };

    if !session.authenticated {
        return Response::err("MB_PURGE_TRASH", "User not authenticated").to_json();
    }

    // delete all BIN messages for this user
    let deleted = match mailbox_repo.purge_trash_for_user(&user_id).await {
        Ok(n) => n,
        Err(e) => {
            eprintln!("[MB_PURGE_TRASH] DB error: {:?}", e);
            return Response::err("MB_PURGE_TRASH", "Database error").to_json();
        }
    };

    let data = serde_json::json!({ "deleted": deleted });

    Response::ok("MB_PURGE_TRASH")
        .with_msg("Trash emptied")
        .with_data(data)
        .to_json()
}
