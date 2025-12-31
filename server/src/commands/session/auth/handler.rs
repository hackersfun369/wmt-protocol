use crate::comm::{Request, Response};
use crate::session::{SessionStore, WmtpSession};
use crate::token::token_from_email;
use crate::commands::mailbox::db::MailboxRepository;

use mongodb::bson::{doc, oid::ObjectId, DateTime as BsonDateTime};
use mongodb::Collection;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserDoc {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub email: String,
    pub name: Option<String>,
    pub avatar_url: Option<String>,
    pub timezone: Option<String>,
    pub signature: Option<String>,
    pub created_at: BsonDateTime,
}

pub async fn handle_auth(
    req: &Request,
    sessions: &SessionStore,
    mailbox_repo: &MailboxRepository,
    users_coll: &Collection<UserDoc>,
) -> String {
    // 1) Validate email
    let email = match req.data.get("email").and_then(|v| v.as_str()) {
        Some(e) if !e.trim().is_empty() => e.trim().to_lowercase(),
        _ => {
            return Response::err("AUTH", "Missing or empty email").to_json();
        }
    };

    if !email.contains('@') || !email.contains('.') {
        return Response::err("AUTH", "Invalid email format").to_json();
    }

    // 2) Stable token from email
    let token = token_from_email(&email);

    // 3) Find or create user in Mongo
    let user_id = match users_coll
        .find_one(doc! { "email": &email })
        .await
    {
        Ok(Some(user)) => user.id,
        Ok(None) => {
            // create new user with default profile fields
            let new_user = UserDoc {
                id: ObjectId::new(),
                email: email.clone(),
                name: None,
                avatar_url: None,
                timezone: None,
                signature: None,
                created_at: BsonDateTime::now(),
            };
            if let Err(e) = users_coll.insert_one(&new_user).await {
                eprintln!("[AUTH] Failed to insert user: {:?}", e);
                return Response::err("AUTH", "Database error").to_json();
            }
            new_user.id
        }
        Err(e) => {
            eprintln!("[AUTH] DB error on find user: {:?}", e);
            return Response::err("AUTH", "Database error").to_json();
        }
    };

    // 4) Ensure global default folders exist (no per-user folders now)
    if let Err(e) = mailbox_repo.ensure_default_folders_global().await {
        eprintln!("[AUTH] ensure_default_folders_global error: {:?}", e);
        return Response::err("AUTH", "Database error").to_json();
    }

    // 5) Insert or update in-memory session (now including user_id)
    {
        let mut store = sessions.lock().unwrap();
        store
            .entry(token.clone())
            .and_modify(|s| {
                s.email = Some(email.clone());
                s.authenticated = true;
                s.user_id = Some(user_id.clone());
            })
            .or_insert_with(|| {
                let mut s = WmtpSession::new_authenticated(token.clone(), email.clone());
                s.user_id = Some(user_id.clone());
                s
            });
    }

    // 6) Response
    Response::ok("AUTH_OK")
        .with_token(token)
        .with_auth(true)
        .with_msg("Authentication successful")
        .to_json()
}
