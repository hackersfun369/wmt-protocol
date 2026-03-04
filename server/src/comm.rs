// src/commands.rs
use serde::{Deserialize, Serialize}; // Cleaned unused imports

pub mod cmd {
    pub const INIT: &str = "INIT";
    pub const AUTH: &str = "AUTH";
    pub const RESUME: &str = "RESUME";
    pub const LOGOUT: &str = "LOGOUT";
    pub const SESSION_INFO: &str = "SESSION_INFO";
    pub const SESSION_LIST: &str = "SESSION_LIST";
    pub const CONNECTION_LIST: &str = "CONNECTION_LIST";
    pub const SESSION_KILL: &str = "SESSION_KILL";
    pub const SESSION_SUSPEND: &str = "SESSION_SUSPEND";
    pub const SESSION_RESUME_SUSPENDED: &str = "SESSION_RESUME_SUSPENDED";
    pub const PING: &str = "PING";
    pub const PONG: &str = "PONG";
    pub const HB: &str = "HB";
    pub const UPTIME: &str = "UPTIME";
    pub const LATENCY_PING: &str = "LATENCY_PING";
    pub const CONNECTION_INFO: &str = "CONNECTION_INFO";
    pub const RATE_LIMIT_INFO: &str = "RATE_LIMIT_INFO";
    pub const DEBUG_ECHO: &str = "DEBUG_ECHO";

    pub const INFO: &str = "INFO";
    pub const STATUS: &str = "STATUS";
    pub const CAPABILITIES: &str = "CAPABILITIES";
    pub const TIME: &str = "TIME";
    pub const VERSION_CHECK: &str = "VERSION_CHECK";
    pub const CONFIG_PUBLIC_GET: &str = "CONFIG_PUBLIC_GET";

    pub const MB_LIST: &str = "MB_LIST";
    pub const MAIL_LIST: &str = "MAIL_LIST";
    pub const MB_CREATE: &str = "MB_CREATE";
    pub const MB_RENAME: &str = "MB_RENAME";
    pub const MB_DELETE: &str = "MB_DELETE";
    pub const MB_INFO: &str = "MB_INFO";
    pub const MB_PURGE_TRASH: &str = "MB_PURGE_TRASH";
    pub const MB_SUBSCRIBE: &str = "MB_SUBSCRIBE";
    pub const MB_UNSUBSCRIBE: &str = "MB_UNSUBSCRIBE";

    pub const MSG_SEND: &str = "MSG_SEND";
    pub const MSG_SEND_DRAFT: &str = "MSG_SEND_DRAFT";
    pub const MSG_LIST: &str = "MSG_LIST";
    pub const MSG_GET: &str = "MSG_GET";
    pub const MSG_HEADERS: &str = "MSG_HEADERS";
    pub const MSG_THREAD: &str = "MSG_THREAD";
    pub const MSG_MOVE: &str = "MSG_MOVE";
    pub const MSG_COPY: &str = "MSG_COPY";
    pub const MSG_DELETE: &str = "MSG_DELETE";
    pub const MSG_EXPUNGE: &str = "MSG_EXPUNGE";
    pub const MSG_UNDELETE: &str = "MSG_UNDELETE";
    pub const MSG_FLAG_SET: &str = "MSG_FLAG_SET";
    pub const MSG_FLAG_CLEAR: &str = "MSG_FLAG_CLEAR";
    pub const MSG_BULK_ACTION: &str = "MSG_BULK_ACTION";

    pub const SEARCH: &str = "SEARCH";
    pub const SEARCH_GLOBAL: &str = "SEARCH_GLOBAL";
    pub const SEARCH_ADV: &str = "SEARCH_ADV";
    pub const SEARCH_SUGGEST: &str = "SEARCH_SUGGEST";

    pub const PROFILE_GET: &str = "PROFILE_GET";
    pub const PROFILE_SET: &str = "PROFILE_SET";
    pub const PREF_GET: &str = "PREF_GET";
    pub const PREF_SET: &str = "PREF_SET";

    pub const ATTACH_UPLOAD_INIT: &str = "ATTACH_UPLOAD_INIT";
    pub const ATTACH_GET: &str = "ATTACH_GET";
}

#[derive(Debug, Deserialize)]
pub struct Request {
    pub cmd: String,
    #[serde(default)]
    pub data: serde_json::Value,
}

impl Request {
    pub fn from_json(text: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(text)
    }
}
// connections info 

#[derive(Debug, Serialize)]
pub struct ConnectionSummary {
    pub id: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remote_addr: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_token: Option<String>,
    pub authenticated: bool,
}

// session info
#[derive(Debug, Serialize)]
pub struct SessionSummary {
    pub session_token: String,
    pub auth: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct Response {
    pub cmd: String,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub msg: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sessions: Option<Vec<SessionSummary>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub connections: Option<Vec<ConnectionSummary>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server_time: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uptime_seconds: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}


impl Response {
    pub fn ok(cmd: &str) -> Self {
        Self {
            cmd: cmd.to_string(),
            status: "OK".to_string(),
            msg: None,
            session_token: None,
            auth: None,
            email: None,
            sessions: None,
            connections: None,
            total: None,
            server_time: None,
            uptime_seconds: None,
            data: None,
        }
    }

    pub fn err(cmd: &str, msg: &str) -> Self {
        Self {
            cmd: cmd.to_string(),
            status: "ERR".to_string(),
            msg: Some(msg.to_string()),
            session_token: None,
            auth: None,
            email: None,
            sessions: None,
            connections: None,
            total: None,
            server_time: None,
            uptime_seconds: None,
            data: None,
        }
    }

    pub fn with_msg(mut self, msg: &str) -> Self {
        self.msg = Some(msg.to_string());
        self
    }

    pub fn with_token(mut self, token: String) -> Self {
        self.session_token = Some(token);
        self
    }

    pub fn with_auth(mut self, auth: bool) -> Self {
        self.auth = Some(auth);
        self
    }

    pub fn with_email(mut self, email: Option<String>) -> Self {
        self.email = email;
        self
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| "{}".to_string())
    }

    // sessions info
    pub fn with_sessions(mut self, sessions: Vec<SessionSummary>) -> Self {
        self.sessions = Some(sessions);
        self
    }

    // connections info
    pub fn with_connections(mut self, list: Vec<ConnectionSummary>) -> Self {
        self.connections = Some(list);
        self
    }

    // total connection count
    pub fn with_total(mut self, total: u64) -> Self {
        self.total = Some(total);
        self
    }

    // ping info
    pub fn with_server_time(mut self, t: String) -> Self {
        self.server_time = Some(t);
        self
    }

    pub fn with_uptime(mut self, secs: u64) -> Self {
        self.uptime_seconds = Some(secs);
        self
    }

    // MB_CREATE data
    pub fn with_data(mut self, data: serde_json::Value) -> Self {
        self.data = Some(data);
        self
    }

}
