use mongodb::Collection;
use mongodb::bson::oid::ObjectId;
use tracing::warn;
use crate::comm::{Request, Response};
use crate::session::SessionStore;
use crate::commands::mailbox::db::{MailboxRepository, AttachmentMeta};
use crate::commands::attachment::attach_upload_init::handler::PendingUpload;
use crate::commands::session::auth::handler::UserDoc;
use mongodb::bson::doc as bson_doc;

pub async fn handle_msg_send(
    token: &str,
    sessions: &SessionStore,
    mailbox_repo: &MailboxRepository,
    uploads_coll: &Collection<PendingUpload>,
    users_coll: &Collection<UserDoc>,
    req: &Request,
) -> String {
    if token.is_empty() {
        return Response::err("MSG_SEND", "Missing session token").to_json();
    }

    // 1) resolve session
    let session = {
        let store = sessions.lock().unwrap();
        store.get(token).cloned()
    };

    let session = match session {
        Some(s) => s,
        None => return Response::err("MSG_SEND", "Invalid or expired session").to_json(),
    };

    let user_id: ObjectId = match &session.user_id {
        Some(id) => id.clone(),
        None => return Response::err("MSG_SEND", "User not authenticated").to_json(),
    };

    if !session.authenticated {
        return Response::err("MSG_SEND", "User not authenticated").to_json();
    }

    // 2) derive 'from' from session email (ignore client 'from')
    let from = match &session.email {
        Some(e) if !e.is_empty() => e.as_str(),
        _ => return Response::err("MSG_SEND", "Session email missing").to_json(),
    };

    // 3) parse 'to'
    let to_vals = match req.data.get("to") {
        Some(v) if v.is_array() => v.as_array().unwrap(),
        _ => return Response::err("MSG_SEND", "Missing or invalid 'to' array").to_json(),
    };

    let mut to: Vec<String> = Vec::new();
    for item in to_vals {
        if let Some(s) = item.as_str() {
            if !s.is_empty() {
                to.push(s.to_string());
            }
        }
    }
    if to.is_empty() {
        return Response::err("MSG_SEND", "Empty 'to' list").to_json();
    }

    // 4) subject & body
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

    // 5) attachments
    let mut attachments: Vec<AttachmentMeta> = Vec::new();
    if let Some(att_ids) = req.data.get("attachment_ids").and_then(|v| v.as_array()) {
        for id_val in att_ids {
            if let Some(upload_id) = id_val.as_str() {
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
                        warn!("[MSG_SEND] Attachment {} not found or not completed", upload_id);
                    }
                }
            }
        }
    }

    // 6) Store in sender's SENT folder
    let msg_id = match mailbox_repo
        .insert_sent_message(&user_id, from, to.clone(), &subject, &body, attachments)
        .await
    {
        Ok(id) => id,
        Err(e) => {
            eprintln!("[MSG_SEND] DB error storing sent message: {:?}", e);
            return Response::err("MSG_SEND", "Database error").to_json();
        }
    };

    // 7) Deliver to each recipient's INBOX
    // Look up each recipient's user_id by email and insert into their inbox.
    // Recipients who don't exist yet as users are silently skipped (external addresses).
    let mut delivered_count = 0usize;
    for recipient_email in &to {
        let recipient_email_lower = recipient_email.to_lowercase();

        match users_coll
            .find_one(bson_doc! { "email": &recipient_email_lower })
            .await
        {
            Ok(Some(recipient_user)) => {
                match mailbox_repo
                    .insert_incoming_into_inbox(
                        &recipient_user.id,
                        from,
                        to.clone(),
                        &subject,
                        &body,
                    )
                    .await
                {
                    Ok(_) => {
                        delivered_count += 1;
                        eprintln!(
                            "[MSG_SEND] Delivered to inbox of {}",
                            recipient_email_lower
                        );
                    }
                    Err(e) => {
                        warn!(
                            "[MSG_SEND] Failed to deliver to inbox of {}: {:?}",
                            recipient_email_lower, e
                        );
                    }
                }
            }
            Ok(None) => {
                // Recipient not a registered WMTP user — skip silently
                warn!(
                    "[MSG_SEND] Recipient {} not found in users collection (external?)",
                    recipient_email_lower
                );
            }
            Err(e) => {
                warn!(
                    "[MSG_SEND] DB error looking up recipient {}: {:?}",
                    recipient_email_lower, e
                );
            }
        }
    }

    // 8) Response
    let data = serde_json::json!({
        "id": msg_id.to_hex(),
        "folder_code": "SENT",
        "from": from,
        "to": to,
        "subject": subject,
        "delivered": delivered_count,
    });

    Response::ok("MSG_SEND")
        .with_msg("Message sent and delivered")
        .with_data(data)
        .to_json()
}
