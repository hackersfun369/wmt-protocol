use crate::comm::{Request, Response};
use crate::session::SessionStore;
use crate::commands::user::pref_get::handler::PreferencesDoc;
use mongodb::Collection;
use mongodb::bson::doc as bson_doc;
use mongodb::options::UpdateOptions;

pub async fn handle_pref_set(
    token: &str,
    sessions: &SessionStore,
    prefs_coll: &Collection<PreferencesDoc>,
    req: &Request,
) -> String {
    if token.is_empty() {
        return Response::err("PREF_SET", "Missing session token").to_json();
    }

    let user_id = {
        let store = sessions.lock().unwrap();
        match store.get(token).and_then(|s| s.user_id.clone()) {
            Some(id) => id,
            None => return Response::err("PREF_SET", "Invalid session or not authenticated").to_json(),
        }
    };

    let theme = req.data.get("theme").and_then(|v| v.as_str()).unwrap_or("light");
    let notifications_enabled = req.data.get("notifications_enabled").and_then(|v| v.as_bool()).unwrap_or(true);
    let language = req.data.get("language").and_then(|v| v.as_str()).unwrap_or("en");

    let prefs = bson_doc! {
        "theme": theme,
        "notifications_enabled": notifications_enabled,
        "language": language
    };

    match prefs_coll.update_one(
        bson_doc! { "userId": user_id },
        bson_doc! { "$set": prefs }
    )
    .upsert(true)
    .await {
        Ok(_) => Response::ok("PREF_SET_OK")
            .with_msg("Preferences updated successfully")
            .to_json(),
        Err(e) => Response::err("PREF_SET", &format!("Database error: {:?}", e)).to_json(),
    }
}
