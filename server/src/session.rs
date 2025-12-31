// src/session.rs
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use mongodb::bson::oid::ObjectId;

#[derive(Debug, Clone)]
pub struct WmtpSession {
     pub token: String,
    pub email: Option<String>,
    pub authenticated: bool,
    pub suspended: bool,
    pub user_id: Option<ObjectId>,
}

impl WmtpSession {
    pub fn new_ephemeral(token: String) -> Self {
        Self { token,
            email: None,
            authenticated: false,
            suspended: false,
            user_id: None,
        }
    }

    pub fn new_authenticated(token: String, email: String) -> Self {
        Self { token,
            email: Some(email),
            authenticated: true,
            suspended: false,
            user_id: None,
        }
    }
}

pub type SessionStore = Arc<Mutex<HashMap<String, WmtpSession>>>;

pub fn create_session_store() -> SessionStore {
    Arc::new(Mutex::new(HashMap::new()))
}
