use mongodb::bson::{doc as bson_doc, oid::ObjectId, DateTime as BsonDateTime};
use mongodb::Collection;
use serde::{Deserialize, Serialize};

use crate::comm::{Request, Response};
use crate::session::SessionStore;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PendingUpload {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub user_id: ObjectId,
    pub upload_id: String,
    pub filename: String,
    pub mime_type: String,
    pub size_bytes: u64,
    pub created_at: BsonDateTime,
    pub completed: bool,
}


fn gen_upload_id() -> String {
    use rand::{distributions::Alphanumeric, Rng};
    let s: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(16)
        .map(char::from)
        .collect();
    format!("upl_{}", s)
}

pub async fn handle_attach_upload_init(
    token: &str,
    req: &Request,
    sessions: &SessionStore,
    uploads_coll: &Collection<PendingUpload>,
) -> String {
    if token.is_empty() {
        return Response::err("ATTACH_UPLOAD_INIT", "Missing session token").to_json();
    }

    // session
    let session = {
        let store = sessions.lock().unwrap();
        store.get(token).cloned()
    };

    let session = match session {
        Some(s) => s,
        None => return Response::err("ATTACH_UPLOAD_INIT", "Invalid or expired session").to_json(),
    };

    let user_id = match &session.user_id {
        Some(id) => id.clone(),
        None => return Response::err("ATTACH_UPLOAD_INIT", "User not authenticated").to_json(),
    };

    if !session.authenticated {
        return Response::err("ATTACH_UPLOAD_INIT", "User not authenticated").to_json();
    }

    // fields
    let filename = match req.data.get("filename").and_then(|v| v.as_str()) {
        Some(f) if !f.is_empty() => f.to_string(),
        _ => return Response::err("ATTACH_UPLOAD_INIT", "Missing filename").to_json(),
    };

    let mime_type = req
        .data
        .get("mime_type")
        .and_then(|v| v.as_str())
        .unwrap_or("application/octet-stream")
        .to_string();

    let size_bytes = req
        .data
        .get("size_bytes")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);

    let upload_id = gen_upload_id();

    let doc = PendingUpload {
        id: ObjectId::new(),
        user_id,
        upload_id: upload_id.clone(),
        filename: filename.clone(),
        mime_type: mime_type.clone(),
        size_bytes,
        created_at: BsonDateTime::now(),
        completed: false,
    };

    if let Err(e) = uploads_coll.insert_one(&doc).await {
        eprintln!("[ATTACH_UPLOAD_INIT] DB error: {:?}", e);
        return Response::err("ATTACH_UPLOAD_INIT", "Database error").to_json();
    }

    let data = serde_json::json!({
        "upload": {
            "upload_id": upload_id,
            "filename": filename,
            "mime_type": mime_type,
            "size_bytes": size_bytes
        }
    });

    Response::ok("ATTACH_UPLOAD_INIT")
        .with_msg("Attachment upload initialized")
        .with_data(data)
        .to_json()
}
