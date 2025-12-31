use crate::comm::{Request, Response};
use serde_json::json;

pub async fn handle_capabilities(_req: &Request) -> String {
    Response::ok("CAPABILITIES")
        .with_data(json!({
            "commands": [
                "INIT", "AUTH", "RESUME", "LOGOUT", "SESSION_INFO", "SESSION_LIST", "SESSION_KILL",
                "PING", "PONG", "HB", "UPTIME", "LATENCY_PING", "CONNECTION_INFO", "DEBUG_ECHO",
                "INFO", "STATUS", "CAPABILITIES", "TIME", "VERSION_CHECK", "CONFIG_PUBLIC_GET",
                "MB_LIST", "MB_CREATE", "MB_RENAME", "MB_DELETE", "MB_INFO", "MB_PURGE_TRASH",
                "MSG_SEND", "MSG_SEND_DRAFT", "MSG_LIST", "MSG_GET", "MSG_HEADERS", "MSG_THREAD",
                "MSG_MOVE", "MSG_COPY", "MSG_DELETE", "MSG_EXPUNGE", "MSG_UNDELETE", "MSG_FLAG_SET",
                "MSG_FLAG_CLEAR", "MSG_BULK_ACTION", "SEARCH", "SEARCH_GLOBAL", "SEARCH_ADV",
                "PROFILE_GET", "PROFILE_SET", "ATTACH_UPLOAD_INIT", "ATTACH_GET"
            ],
            "auth_methods": ["token", "oauth_future"],
            "max_attachment_size": 104857600
        }))
        .to_json()
}
