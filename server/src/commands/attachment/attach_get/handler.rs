use futures_util::io::AsyncReadExt as FuturesAsyncReadExt;
use mongodb::bson::doc;
use mongodb::gridfs::GridFsBucket;
use mongodb::{Collection, Database};
use serde::{Deserialize, Serialize};
use base64::Engine;
use base64::engine::general_purpose::STANDARD;

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
    content_b64: String,
}

pub async fn handle_attach_get(
    req: &Request,
    sessions: &SessionStore,
    uploads_coll: &Collection<PendingUpload>,
    db: &Database,
) -> String {
    // robust field extraction
    let token = match req.data.get("session_token").and_then(|v| v.as_str()) {
        Some(t) if !t.is_empty() => t,
        _ => return Response::err("ATTACH_GET", "Missing session_token").to_json(),
    };

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
        .find_one(doc! { "user_id": &user_id, "upload_id": attachment_id })
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

    // GridFS bucket
    let bucket: GridFsBucket = db.gridfs_bucket(None);

    // Use same name convention as upload: "attach:<upload_id>"
    let gridfs_name = format!("attach:{}", pending.upload_id);

    // Download by name
    let mut download_stream = match bucket.open_download_stream_by_name(gridfs_name).await {
        Ok(s) => s,
        Err(e) => {
            eprintln!("[ATTACH_GET] GridFS open_download_stream_by_name error: {:?}", e);
            return Response::err("ATTACH_GET", "Attachment storage error").to_json();
        }
    };

    let mut bytes: Vec<u8> = Vec::new();
    if let Err(e) = download_stream.read_to_end(&mut bytes).await {
        eprintln!("[ATTACH_GET] GridFS read_to_end error: {:?}", e);
        return Response::err("ATTACH_GET", "Attachment read error").to_json();
    }

    let content_b64 = STANDARD.encode(&bytes);

    let payload = AttachGetPayload {
        attachment_id: pending.upload_id.clone(),
        filename: pending.filename.clone(),
        mime_type: pending.mime_type.clone(),
        size_bytes: pending.size_bytes,
        content_b64,
    };

    let data_value = serde_json::to_value(payload).unwrap_or_default();

    Response::ok("ATTACH_GET")
        .with_msg("Attachment fetched")
        .with_data(data_value)
        .to_json()
}
