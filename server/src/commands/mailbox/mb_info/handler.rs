use mongodb::bson::oid::ObjectId;

use crate::comm::{Request, Response};
use crate::session::SessionStore;
use crate::commands::mailbox::db::{MailboxRepository, MbFolderInfoDto};

pub async fn handle_mb_info(
    token: &str,
    sessions: &SessionStore,
    mailbox_repo: &MailboxRepository,
    req: &Request,
) -> String {
    if token.is_empty() {
        return Response::err("MB_INFO", "Missing session token").to_json();
    }

    // 2) resolve session
    let session = {
        let store = sessions.lock().unwrap();
        store.get(token).cloned()
    };

    let session = match session {
        Some(s) => s,
        None => return Response::err("MB_INFO", "Invalid or expired session").to_json(),
    };

    let user_id: ObjectId = match &session.user_id {
        Some(id) => id.clone(),
        None => return Response::err("MB_INFO", "User not authenticated").to_json(),
    };

    if !session.authenticated {
        return Response::err("MB_INFO", "User not authenticated").to_json();
    }

    // 3) folder_code (handle 'folder_code' or 'mailbox')
    let folder_code = match req.data.get("folder_code").or_else(|| req.data.get("mailbox")).and_then(|v| v.as_str()) {
        Some(c) if !c.is_empty() => c,
        _ => return Response::err("MB_INFO", "Missing folder_code").to_json(),
    };

    // 4) query Mongo
    let info: MbFolderInfoDto = match mailbox_repo
        .get_folder_info_for_user(&user_id, folder_code)
        .await
    {
        Ok(i) => i,
        Err(e) => {
            eprintln!("[MB_INFO] DB error: {:?}", e);
            return Response::err("MB_INFO", "Database error").to_json();
        }
    };

    let json = serde_json::json!({
        "cmd": "MB_INFO",
        "status": "OK",
        "msg": "Folder info",
        "folder": info,
    });

    json.to_string()
}
