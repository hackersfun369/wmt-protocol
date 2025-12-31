use mongodb::bson::oid::ObjectId;
use serde_json::Value;

use crate::comm::{Request, Response};
use crate::session::SessionStore;
use crate::commands::mailbox::db::MailboxRepository;

pub async fn handle_mb_create(
    sessiontoken: &str,
    sessions: SessionStore,
    mailbox_repo: &MailboxRepository,
    request: &Request,
) -> String {
    // 1. Resolve session & authenticated user
    let session = {
        // lock is scoped to this block and dropped before any await
        let store = sessions.lock().unwrap();
        store.get(sessiontoken).cloned()
    };

    let session = match session {
        Some(s) => s,
        None => return Response::err("MB_CREATE", "Invalid or expired session").to_json(),
    };

    let userid: ObjectId = match session.user_id {
        Some(ref id) => id.clone(),
        None => return Response::err("MB_CREATE", "User not authenticated").to_json(),
    };

    // 2. Parse request data
    let data = &request.data;
    if data.is_null() {
        return Response::err("MB_CREATE", "Missing data field").to_json();
    }

    let name = match data.get("name") {
        Some(Value::String(name)) => name.clone(),
        _ => return Response::err("MB_CREATE", "Missing or invalid 'name' field").to_json(),
    };

    let code = match data.get("code") {
        Some(Value::String(code)) => code.clone(),
        _ => name.to_uppercase().replace(" ", "_"), // Auto-generate code
    };

    // 3. Validation
    if code.len() < 2 || code.len() > 20 {
        return Response::err("MB_CREATE", "Code must be 2-20 characters").to_json();
    }
    if name.is_empty() || name.len() > 50 {
        return Response::err("MB_CREATE", "Name must be 1-50 characters").to_json();
    }
    if code.to_uppercase() == code.to_lowercase() {
        return Response::err("MB_CREATE", "Code must contain letters").to_json();
    }

    let reserved_codes = ["INBOX", "SENT", "DRAFTS", "BIN", "SPAM"];
    if reserved_codes.contains(&code.to_uppercase().as_str()) {
        return Response::err("MB_CREATE", "Reserved folder code").to_json();
    }

    // 4. Create custom folder (now safe to await)
    match mailbox_repo.create_user_folder(userid, &code, &name).await {
        Ok(folder_id) => {
            let response_data = serde_json::json!({
                "code": code,
                "name": name,
                "id": folder_id.to_hex()
            });

            Response::ok("MB_CREATE")
                .with_msg("Folder created successfully")
                .with_data(response_data)
                .to_json()
        }
        Err(_) => Response::err("MB_CREATE", "Database error creating folder").to_json(),
    }
}
