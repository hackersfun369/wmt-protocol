use crate::comm::{Request, Response};
use crate::commands::mailbox::db::MailboxRepository;
use crate::session::SessionStore;
use mongodb::bson::oid::ObjectId;

pub async fn handle_mb_rename(
    token: &str,
    sessions: &SessionStore,
    mailbox_repo: &MailboxRepository,
    req: &Request,
) -> String {
    if token.is_empty() {
        return Response::err("MB_RENAME", "Missing session token").to_json();
    }

    let user_id = {
        let store = sessions.lock().unwrap();
        match store.get(token).and_then(|s| s.user_id.clone()) {
            Some(id) => id,
            None => return Response::err("MB_RENAME", "Invalid session or not authenticated").to_json(),
        }
    };

    let folder_code = match req.data.get("folder").and_then(|v| v.as_str()) {
        Some(f) => f,
        None => return Response::err("MB_RENAME", "Folder code is required").to_json(),
    };

    let new_name = match req.data.get("new_name").and_then(|v| v.as_str()) {
        Some(n) => n,
        None => return Response::err("MB_RENAME", "New name is required").to_json(),
    };

    match mailbox_repo.rename_user_folder(&user_id, folder_code, new_name).await {
        Ok(count) if count > 0 => Response::ok("MB_RENAME_OK")
            .with_msg("Folder renamed successfully")
            .to_json(),
        Ok(_) => Response::err("MB_RENAME", "Folder not found or no change").to_json(),
        Err(e) => Response::err("MB_RENAME", &format!("Database error: {:?}", e)).to_json(),
    }
}
