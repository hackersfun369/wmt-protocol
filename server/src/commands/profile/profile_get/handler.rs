use std::time::SystemTime;
use chrono::{DateTime, Utc};
use mongodb::bson::{doc as bson_doc, oid::ObjectId};
use mongodb::Collection;
use serde::Serialize;

use crate::comm::{Request, Response};
use crate::session::SessionStore;
use crate::commands::session::auth::handler::UserDoc;

#[derive(Serialize)]
struct ProfileDto {
    id: String,
    email: String,
    name: Option<String>,
    avatar_url: Option<String>,
    timezone: Option<String>,
    signature: Option<String>,
    created_at: String,
}

pub async fn handle_profile_get(
    token: &str,
    sessions: &SessionStore,
    users_coll: &Collection<UserDoc>,
    req: &Request,
) -> String {
    if token.is_empty() {
        return Response::err("PROFILE_GET", "Missing session token").to_json();
    }

    // 2) resolve session
    let session = {
        let store = sessions.lock().unwrap();
        store.get(token).cloned()
    };

    let session = match session {
        Some(s) => s,
        None => return Response::err("PROFILE_GET", "Invalid or expired session").to_json(),
    };

    let user_id: ObjectId = match &session.user_id {
        Some(id) => id.clone(),
        None => return Response::err("PROFILE_GET", "User not authenticated").to_json(),
    };

    if !session.authenticated {
        return Response::err("PROFILE_GET", "User not authenticated").to_json();
    }

    // 3) load user doc
    let user = match users_coll
        .find_one(bson_doc! { "_id": &user_id })
        .await
    {
        Ok(Some(u)) => u,
        Ok(None) => return Response::err("PROFILE_GET", "User not found").to_json(),
        Err(e) => {
            eprintln!("[PROFILE_GET] DB error: {:?}", e);
            return Response::err("PROFILE_GET", "Database error").to_json();
        }
    };

    let st: SystemTime = user.created_at.to_system_time();
    let dt: DateTime<Utc> = st.into();

    let dto = ProfileDto {
        id: user.id.to_hex(),
        email: user.email,
        name: user.name,
        avatar_url: user.avatar_url,
        timezone: user.timezone,
        signature: user.signature,
        created_at: dt.to_rfc3339(),
    };

    let json = serde_json::json!({
        "cmd": "PROFILE_GET",
        "status": "OK",
        "msg": "User profile",
        "profile": dto,
    });

    json.to_string()
}
