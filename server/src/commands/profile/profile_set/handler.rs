use mongodb::bson::{self,doc, oid::ObjectId};
use mongodb::Collection;

use crate::comm::{Request, Response};
use crate::session::SessionStore;
use crate::commands::session::auth::handler::UserDoc;

pub async fn handle_profile_set(
    token: &str,
    sessions: &SessionStore,
    users_coll: &Collection<UserDoc>,
    req: &Request,
) -> String {
    if token.is_empty() {
        return Response::err("PROFILE_SET", "Missing session token").to_json();
    }

    // 2) resolve session
    let session = {
        let store = sessions.lock().unwrap();
        store.get(token).cloned()
    };

    let session = match session {
        Some(s) => s,
        None => return Response::err("PROFILE_SET", "Invalid or expired session").to_json(),
    };

    let user_id: ObjectId = match &session.user_id {
        Some(id) => id.clone(),
        None => return Response::err("PROFILE_SET", "User not authenticated").to_json(),
    };

    if !session.authenticated {
        return Response::err("PROFILE_SET", "User not authenticated").to_json();
    }

    // 3) collect fields to update
    let mut set_doc = bson::Document::new();

    if let Some(name) = req.data.get("name").and_then(|v| v.as_str()) {
        set_doc.insert("name", name.to_string());
    }
    if let Some(avatar) = req.data.get("avatar_url").and_then(|v| v.as_str()) {
        set_doc.insert("avatar_url", avatar.to_string());
    }
    if let Some(tz) = req.data.get("timezone").and_then(|v| v.as_str()) {
        set_doc.insert("timezone", tz.to_string());
    }
    if let Some(sig) = req.data.get("signature").and_then(|v| v.as_str()) {
        set_doc.insert("signature", sig.to_string());
    }

    if set_doc.is_empty() {
        return Response::err("PROFILE_SET", "No fields to update").to_json();
    }

    // 4) updateOne in Mongo
    let res = match users_coll
        .update_one(
            doc! { "_id": &user_id },
            doc! { "$set": set_doc },
        )
        .await
    {
        Ok(r) => r,
        Err(e) => {
            eprintln!("[PROFILE_SET] DB error: {:?}", e);
            return Response::err("PROFILE_SET", "Database error").to_json();
        }
    };

    if res.matched_count == 0 {
        return Response::err("PROFILE_SET", "User not found").to_json();
    }

    Response::ok("PROFILE_SET")
        .with_msg("Profile updated")
        .to_json()
}
