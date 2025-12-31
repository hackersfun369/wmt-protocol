// src/commands/mailbox/mb_list/handler.rs
use mongodb::bson::oid::ObjectId;

use crate::comm::Response;
use crate::session::SessionStore;
use crate::commands::mailbox::db::{MailboxRepository, MbListFolderDto};

pub async fn handle_mb_list(
    session_token: &str,
    sessions: &SessionStore,
    mailbox_repo: &MailboxRepository,
) -> String {
    // 1) Resolve session
    let maybe_session = {
        let store = sessions.lock().unwrap();
        store.get(session_token).cloned()
    };

    let session = match maybe_session {
        Some(s) => s,
        None => {
            return Response::err("MB_LIST", "Invalid or expired session").to_json();
        }
    };

    // 2) Require authenticated user with user_id
    let user_id: ObjectId = match &session.user_id {
        Some(id) => id.clone(),
        None => {
            return Response::err("MB_LIST", "User not authenticated").to_json();
        }
    };

    if !session.authenticated {
        return Response::err("MB_LIST", "User not authenticated").to_json();
    }

    // 3) Ensure global folders exist
    if let Err(e) = mailbox_repo.ensure_default_folders_global().await {
        eprintln!("[MB_LIST] ensure_default_folders_global error: {:?}", e);
        return Response::err("MB_LIST", "Database error").to_json();
    }

    // 4) Query per-user counts by folderCode
    let folders: Vec<MbListFolderDto> = match mailbox_repo.get_mb_list_for_user(&user_id).await {
        Ok(list) => list,
        Err(e) => {
            eprintln!("[MB_LIST] get_mb_list_for_user error: {:?}", e);
            return Response::err("MB_LIST", "Database error").to_json();
        }
    };

    let json = serde_json::json!({
        "cmd": "MB_LIST",
        "status": "OK",
        "msg": "Mailbox folders",
        "folders": folders,
    });

    json.to_string()
}
