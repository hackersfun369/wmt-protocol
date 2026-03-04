use mongodb::Collection;
use mongodb::bson::oid::ObjectId;
use tracing::warn;
use crate::comm::{Request, Response};
use crate::session::SessionStore;
use crate::commands::mailbox::db::{MailboxRepository, AttachmentMeta};
use crate::commands::attachment::attach_upload_init::handler::PendingUpload;
use mongodb::bson::doc as bson_doc;

pub async fn handle_msg_send_draft(
    token: &str,
    sessions: &SessionStore,
    mailbox_repo: &MailboxRepository,
    uploads_coll: &Collection<PendingUpload>,
    req: &Request,
) -> String {
    // 1) session_token
    if token.is_empty() {
        return Response::err("MSG_SEND_DRAFT", "Missing session token").to_json();
    }

    // 2) resolve session
    let session = {
        let store = sessions.lock().unwrap();
        store.get(token).cloned()
    };

    let session = match session {
        Some(s) => s,
        None => return Response::err("MSG_SEND_DRAFT", "Invalid or expired session").to_json(),
    };

    let user_id: ObjectId = match &session.user_id {
        Some(id) => id.clone(),
        None => return Response::err("MSG_SEND_DRAFT", "User not authenticated").to_json(),
    };

    if !session.authenticated {
        return Response::err("MSG_SEND_DRAFT", "User not authenticated").to_json();
    }

    // 3) derive 'from' from session email
    let from = match &session.email {
        Some(e) if !e.is_empty() => e.as_str(),
        _ => return Response::err("MSG_SEND_DRAFT", "Session email missing").to_json(),
    };

    // 4) parse 'to' (can be empty for drafts)
    let mut to: Vec<String> = Vec::new();
    if let Some(v) = req.data.get("to") {
        if let Some(arr) = v.as_array() {
            for item in arr {
                if let Some(s) = item.as_str() {
                    if !s.is_empty() {
                        to.push(s.to_string());
                    }
                }
            }
        }
    }
    // allow empty 'to' for drafts

    // 5) subject & body
    let subject = req
        .data
        .get("subject")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let body = req
        .data
        .get("body")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    // 6) attachments
    let mut attachments: Vec<AttachmentMeta> = Vec::new();
    if let Some(att_ids) = req.data.get("attachment_ids").and_then(|v| v.as_array()) {
        for id_val in att_ids {
            if let Some(upload_id) = id_val.as_str() {
                // look up in PendingUpload
                match uploads_coll.find_one(bson_doc! { "upload_id": upload_id }).await {
                    Ok(Some(pending)) if pending.completed => {
                        attachments.push(AttachmentMeta {
                            id: upload_id.to_string(),
                            filename: pending.filename,
                            mime_type: pending.mime_type,
                            size_bytes: pending.size_bytes,
                            gridfs_id: format!("attach:{}", upload_id),
                        });
                    }
                    _ => {
                        warn!("[MSG_SEND_DRAFT] Attachment {} not found or not completed", upload_id);
                    }
                }
            }
        }
    }

    // 7) store in DRAFTS folder
    let msg_id = match mailbox_repo
        .insert_draft_message(&user_id, from, to.clone(), &subject, &body, attachments)
        .await
    {
        Ok(id) => id,
        Err(e) => {
            eprintln!("[MSG_SEND_DRAFT] DB error: {:?}", e);
            return Response::err("MSG_SEND_DRAFT", "Database error").to_json();
        }
    };

    // 7) response
    let data = serde_json::json!({
        "id": msg_id.to_hex(),
        "folder_code": "DRAFTS",
        "from": from,
        "to": to,
        "subject": subject,
    });

    Response::ok("MSG_SEND_DRAFT")
        .with_msg("Draft stored in Drafts")
        .with_data(data)
        .to_json()
}
