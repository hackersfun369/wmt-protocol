use crate::comm::{Request, Response, ConnectionSummary};
use crate::connection::ConnectionStore;

pub async fn handle_connection_list(
    _req: &Request,
    connections: &ConnectionStore,
) -> String {
    let list: Vec<ConnectionSummary> = {
        let store = connections.lock().unwrap();
        store
            .values()
            .map(|c| ConnectionSummary {
                id: c.id,
                remote_addr: c.remote_addr.clone(),
                session_token: c.session_token.clone(),
                authenticated: c.authenticated,
            })
            .collect()
    };

    let total = list.len() as u64;

    Response::ok("CONNECTION_LIST")
        .with_msg("Active connections")
        .with_connections(list)
        .with_total(total)
        .to_json()
}
