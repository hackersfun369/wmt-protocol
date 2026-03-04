use crate::comm::{Request, Response};
use crate::session::SessionStore;
use mongodb::Collection;
use mongodb::bson::doc as bson_doc;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct PreferencesDoc {
    #[serde(rename = "userId")]
    pub user_id: mongodb::bson::oid::ObjectId,
    pub theme: String,
    pub notifications_enabled: bool,
    pub language: String,
}

pub async fn handle_pref_get(
    token: &str,
    sessions: &SessionStore,
    prefs_coll: &Collection<PreferencesDoc>,
    req: &Request,
) -> String {
    if token.is_empty() {
        return Response::err("PREF_GET", "Missing session token").to_json();
    }

    let user_id = {
        let store = sessions.lock().unwrap();
        match store.get(token).and_then(|s| s.user_id.clone()) {
            Some(id) => id,
            None => return Response::err("PREF_GET", "Invalid session or not authenticated").to_json(),
        }
    };

    match prefs_coll.find_one(bson_doc! { "userId": user_id }).await {
        Ok(Some(prefs)) => Response::ok("PREF_GET_OK")
            .with_data(serde_json::to_value(prefs).unwrap_or_default())
            .to_json(),
        Ok(None) => {
            // Return default preferences
            Response::ok("PREF_GET_OK")
                .with_data(serde_json::json!({
                    "theme": "light",
                    "notifications_enabled": true,
                    "language": "en"
                }))
                .to_json()
        }
        Err(e) => Response::err("PREF_GET", &format!("Database error: {:?}", e)).to_json(),
    }
}
