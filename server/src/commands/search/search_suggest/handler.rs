use crate::comm::{Request, Response};
use serde_json::json;

pub async fn handle_search_suggest(req: &Request) -> String {
    let query = req.data.get("query").and_then(|v| v.as_str()).unwrap_or("");
    
    // Simple mock suggestions
    let suggestions = vec![
        format!("from:{}", query),
        format!("subject:{}", query),
        format!("has:attachment {}", query),
    ];

    Response::ok("SEARCH_SUGGEST_OK")
        .with_data(json!({
            "query": query,
            "suggestions": suggestions
        }))
        .to_json()
}
