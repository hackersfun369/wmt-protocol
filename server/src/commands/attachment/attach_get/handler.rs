use mongodb::bson::doc as bson_doc;
use mongodb::{Collection, Database};
use serde::{Deserialize, Serialize};

use crate::comm::{Request, Response};
use crate::session::SessionStore;
use crate::commands::attachment::attach_upload_init::handler::PendingUpload;

#[derive(Debug, Deserialize)]
struct AttachGetData {
    session_token: String,
    attachment_id: String, // upload_id
}

#[derive(Debug, Serialize)]
struct AttachGetPayload {
    attachment_id: String,
    filename: String,
    mime_type: String,
    size_bytes: u64,
}

pub async fn handle_attach_get(
    token: &str,
    req: &Request,
    sessions: &SessionStore,
    uploads_coll: &Collection<PendingUpload>,
    db: &Database,
) -> String {
    if token.is_empty() {
        return Response::err("ATTACH_GET", "Missing session token").to_json();
    }

    let attachment_id = match req.data.get("attachment_id").and_then(|v| v.as_str()) {
        Some(id) if !id.is_empty() => id,
        _ => return Response::err("ATTACH_GET", "Missing attachment_id").to_json(),
    };

    // Session
    let session = {
        let store = sessions.lock().unwrap();
        store.get(token).cloned()
    };

    let session = match session {
        Some(s) => s,
        None => return Response::err("ATTACH_GET", "Invalid or expired session").to_json(),
    };

    let user_id = match &session.user_id {
        Some(id) => id.clone(),
        None => return Response::err("ATTACH_GET", "User not authenticated").to_json(),
    };

    if !session.authenticated {
        return Response::err("ATTACH_GET", "User not authenticated").to_json();
    }

    // Find PendingUpload by upload_id for this user
    let pending = match uploads_coll
        .find_one(bson_doc! { "user_id": &user_id, "upload_id": attachment_id })
        .await
    {
        Ok(Some(p)) => p,
        Ok(None) => {
            return Response::err("ATTACH_GET", "Attachment not found").to_json();
        }
        Err(e) => {
            eprintln!("[ATTACH_GET] DB error finding upload: {:?}", e);
            return Response::err("ATTACH_GET", "Database error").to_json();
        }
    };

    if !pending.completed {
        return Response::err("ATTACH_GET", "Attachment upload not completed").to_json();
    }

    let payload = AttachGetPayload {
        attachment_id: pending.upload_id.clone(),
        filename: pending.filename.clone(),
        mime_type: pending.mime_type.clone(),
        size_bytes: pending.size_bytes,
    };

    let data_value = serde_json::to_value(payload).unwrap_or_default();

    Response::ok("ATTACH_GET")
        .with_msg("Attachment fetched")
        .with_data(data_value)
        .to_json()
}
