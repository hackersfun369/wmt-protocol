use crate::comm::{Request, Response};
use crate::commands::mailbox::db::MailboxRepository;
use crate::session::SessionStore;
use mongodb::bson::oid::ObjectId;

pub async fn handle_mb_delete(
    token: &str,
    sessions: &SessionStore,
    mailbox_repo: &MailboxRepository,
    req: &Request,
) -> String {
    if token.is_empty() {
        return Response::err("MB_DELETE", "Missing session token").to_json();
    }

    let user_id = {
        let store = sessions.lock().unwrap();
        match store.get(token).and_then(|s| s.user_id.clone()) {
            Some(id) => id,
            None => return Response::err("MB_DELETE", "Invalid session or not authenticated").to_json(),
        }
    };

    let folder_code = match req.data.get("folder").and_then(|v| v.as_str()) {
        Some(f) => f,
        None => return Response::err("MB_DELETE", "Folder code is required").to_json(),
    };

    match mailbox_repo.delete_user_folder(&user_id, folder_code).await {
        Ok(count) if count > 0 => Response::ok("MB_DELETE_OK")
            .with_msg("Folder deleted successfully")
            .to_json(),
        Ok(_) => Response::err("MB_DELETE", "Folder not found").to_json(),
        Err(e) => Response::err("MB_DELETE", &format!("Database error: {:?}", e)).to_json(),
    }
}
