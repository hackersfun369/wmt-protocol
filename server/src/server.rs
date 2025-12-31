// src/server.rs
use anyhow::Result;
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use tokio::time::interval;
use tracing::{error, info, warn};

use chrono::{DateTime, Utc};
use wtransport::{Endpoint, Identity, ServerConfig};
use wtransport::endpoint::IncomingSession;
use wtransport::stream::{RecvStream, SendStream};

// attachments: streaming into GridFS
use futures_util::io::AsyncWriteExt as FuturesAsyncWriteExt;
use mongodb::bson::doc;
use mongodb::gridfs::GridFsBucket;

// MongoDB
use mongodb::{Client, options::ClientOptions, Collection, Database};
use serde_json::Value;

use crate::commands::mailbox::db::{MailboxRepository, Message};
use crate::comm::{cmd, Request, Response};
use crate::commands::connections::list::handler as connection_list_handler;

// session imports
use crate::session::{create_session_store, SessionStore};
use crate::commands::session::init::handler as init_handler;
use crate::commands::session::auth::handler as auth_handler;
use crate::commands::session::resume::handler as resume_handler;
use crate::commands::session::logout::handler as logout_handler;
use crate::commands::session::session_info::handler as session_info_handler;
use crate::commands::session::session_list::handler as session_list_handler;
use crate::commands::session::session_kill::handler as session_kill_handler;
use crate::commands::session::session_suspend::handler as session_suspend_handler;
use crate::commands::session::session_resume_suspended::handler as session_resume_suspended_handler;
use crate::commands::mailbox::mb_list::handler as mb_list_handler;
use crate::commands::session::auth::handler::UserDoc;
use crate::commands::mailbox::mail_list::handler as mail_list_handler;
use crate::commands::mailbox::mb_create::handler as mb_create_handler;
use crate::commands::mailbox::mb_rename::handler as mb_rename_handler;
use crate::commands::mailbox::mb_delete::handler as mb_delete_handler;
use crate::commands::mailbox::mb_subscribe::handler as mb_subscribe_handler;
use crate::commands::mailbox::mb_unsubscribe::handler as mb_unsubscribe_handler;
use crate::commands::mailbox::mb_info::handler as mb_info_handler;
use crate::commands::mailbox::mb_purge_trash::handler as mb_purge_trash_handler;
use crate::commands::message::msg_send::handler as msg_send_handler;
use crate::commands::message::msg_send_draft::handler as msg_send_draft_handler;
use crate::commands::message::msg_list::handler as msg_list_handler;
use crate::commands::message::msg_get::handler as msg_get_handler;
use crate::commands::message::msg_headers::handler as msg_headers_handler;
use crate::commands::message::msg_thread::handler as msg_thread_handler;
use crate::commands::message::msg_move::handler as msg_move_handler;
use crate::commands::message::msg_copy::handler as msg_copy_handler;
use crate::commands::message::msg_delete::handler as msg_delete_handler;
use crate::commands::message::msg_expunge::handler as msg_expunge_handler;
use crate::commands::message::msg_undelete::handler as msg_undelete_handler;
use crate::commands::message::msg_flag_set::handler as msg_flag_set_handler;
use crate::commands::message::msg_flag_clear::handler as msg_flag_clear_handler;
use crate::commands::message::msg_bulk_action::handler as msg_bulk_action_handler;
use crate::commands::search::search_simple::handler as search_handler;
use crate::commands::search::search_global::handler as search_global_handler;
use crate::commands::search::search_adv::handler as search_adv_handler;
use crate::commands::search::search_suggest::handler as search_suggest_handler;
use crate::commands::profile::profile_get::handler as profile_get_handler;
use crate::commands::profile::profile_set::handler as profile_set_handler;
use crate::commands::health::pong::handler as pong_handler;
use crate::commands::health::hb::handler as hb_handler;
use crate::commands::health::connection_info::handler as connection_info_handler_v2;
use crate::commands::health::debug_echo::handler as debug_echo_handler;
use crate::commands::admin::capabilities::handler as capabilities_handler;
use crate::commands::admin::time::handler as time_handler;
use crate::commands::admin::version_check::handler as version_check_handler;
use crate::commands::admin::config_public_get::handler as config_public_get_handler;
use crate::commands::user::pref_get::handler::{handle_pref_get, PreferencesDoc};
use crate::commands::user::pref_set::handler::handle_pref_set;

// attachment commands
use crate::commands::attachment::attach_upload_init::handler as attach_upload_init_handler;
use crate::commands::attachment::attach_upload_init::handler::PendingUpload;
use crate::commands::attachment::attach_get::handler as attach_get_handler;

use crate::connection::{ConnectionStore, create_connection_store, make_connection_info};

pub async fn run_server() -> Result<()> {
    let start_time = SystemTime::now();
    let port = 4433;
    let cert_path = "C:/Drive_D/webdev/WMTP/certs/cert.pem";
    let key_path = "C:/Drive_D/webdev/WMTP/certs/key.pem";

    // Session & connection stores
    let sessions: SessionStore = create_session_store();
    let connections: ConnectionStore = create_connection_store();
    let mut next_conn_id: u64 = 1;

    // MongoDB client and mailbox repo
    let mongo_client = Client::with_options(
        ClientOptions::parse("mongodb://localhost:27017").await?
    )?;
    let db = mongo_client.database("wmtp");
    let db_arc = Arc::new(db.clone());

    let users_coll: Collection<UserDoc> = db_arc.collection::<UserDoc>("users");
    let mailbox_repo = Arc::new(MailboxRepository::new(&db));
    let users_coll = Arc::new(users_coll);
    let messages_coll: Collection<Message> = db_arc.collection::<Message>("messages");
    let preferences_coll: Collection<PreferencesDoc> = db_arc.collection::<PreferencesDoc>("preferences");
    let preferences_coll = Arc::new(preferences_coll);
    // attachments uploads collection
    let uploads_coll: Collection<PendingUpload> = db.collection::<PendingUpload>("uploads");

    // TLS identity and WebTransport endpoint
    let identity = Identity::load_pemfiles(cert_path, key_path).await?;
    let server_config = ServerConfig::builder()
        .with_bind_default(port)
        .with_identity(&identity)
        .max_idle_timeout(Some(Duration::from_secs(60)))?
        .keep_alive_interval(Some(Duration::from_secs(10)))
        .build();

    let endpoint = Endpoint::server(server_config)?;
    info!("WMTP server running on https://localhost:{port}");

    let heartbeat_interval: u64 = 5; // seconds

    loop {
        let incoming: IncomingSession = endpoint.accept().await;
        let sessions = sessions.clone();
        let connections = connections.clone();
        let conn_id = next_conn_id;
        next_conn_id += 1;

        let start_time_clone = start_time;
        let mailbox_repo = mailbox_repo.clone();
        let users_coll_cloned = users_coll.clone();
        let uploads_coll_cloned = uploads_coll.clone();
        let messages_coll_cloned = messages_coll.clone();
        let preferences_coll_cloned = preferences_coll.clone();
        let db_cloned = db_arc.clone();

        tokio::spawn(async move {
            if let Err(e) = handle_connection(
                incoming,
                sessions,
                connections,
                conn_id,
                heartbeat_interval,
                start_time_clone,
                mailbox_repo,
                users_coll_cloned,
                uploads_coll_cloned,
                messages_coll_cloned,
                preferences_coll_cloned,
                db_cloned,
            )
            .await
            {
                error!("Connection error: {:?}", e);
            }
        });
    }
}

// PING
fn make_ping_response(start_time: SystemTime) -> String {
    let now = SystemTime::now();
    let uptime = now
        .duration_since(start_time)
        .unwrap_or(Duration::from_secs(0))
        .as_secs();

    let now_utc: DateTime<Utc> = now.into();
    let ts = now_utc.to_rfc3339();

    Response::ok("PONG")
        .with_msg("PONG")
        .with_server_time(ts)
        .with_uptime(uptime)
        .to_json()
}

// LATENCY_PING
fn make_latency_response(start_time: SystemTime) -> String {
    let now = SystemTime::now();
    let uptime = now
        .duration_since(start_time)
        .unwrap_or(Duration::from_secs(0))
        .as_secs();

    let now_utc: DateTime<Utc> = now.into();
    let ts = now_utc.to_rfc3339();

    Response::ok("LATENCY_PONG")
        .with_msg("Latency ping")
        .with_server_time(ts)
        .with_uptime(uptime)
        .to_json()
}

// Heartbeat
fn make_hb_response(start_time: SystemTime) -> String {
    let now = SystemTime::now();
    let uptime = now
        .duration_since(start_time)
        .unwrap_or(Duration::from_secs(0))
        .as_secs();

    let now_utc: DateTime<Utc> = now.into();
    let ts = now_utc.to_rfc3339();

    Response::ok("HB")
        .with_msg("Heartbeat")
        .with_server_time(ts)
        .with_uptime(uptime)
        .to_json()
}

async fn handle_connection(
    incoming: IncomingSession,
    sessions: SessionStore,
    connections: ConnectionStore,
    conn_id: u64,
    hb_interval: u64,
    start_time: SystemTime,
    mailbox_repo: Arc<MailboxRepository>,
    users_coll: Arc<Collection<UserDoc>>,
    uploads_coll: Collection<PendingUpload>,
    messages_coll: Collection<Message>,
    preferences_coll: Arc<Collection<PreferencesDoc>>,
    db: Arc<Database>,
) -> Result<()> {
    let session_request = incoming.await?;
    let remote = session_request.remote_address();
    let connection = Arc::new(session_request.accept().await?);

    {
        let mut store = connections.lock().unwrap();
        store.insert(conn_id, make_connection_info(conn_id, Some(remote)));
    }

    // 1) control stream
    let (control_send, control_recv) = connection.accept_bi().await?;

    let sessions_clone = sessions.clone();
    let connections_clone = connections.clone();
    let mailbox_repo_clone = mailbox_repo.clone();
    let users_coll_clone = users_coll.clone();
    let uploads_coll_clone = uploads_coll.clone();
    let messages_coll_clone = messages_coll.clone();
    let preferences_coll_clone = preferences_coll.clone();
    let db_clone = db.clone();
    let remote_str = remote.to_string();

    tokio::spawn(async move {
        if let Err(e) = handle_control_stream(
            control_send,
            control_recv,
            sessions_clone,
            connections_clone,
            conn_id,
            hb_interval,
            start_time,
            mailbox_repo_clone,
            users_coll_clone,
            uploads_coll_clone,
            messages_coll_clone,
            preferences_coll_clone,
            db_clone,
            remote_str,
        )
        .await
        {
            warn!("Control stream ended: {:?}", e);
        }
    });

    // 2) extra streams = attachment streams
    loop {
        match connection.accept_bi().await {
            Ok((send, recv)) => {
                let sessions_clone = sessions.clone();
                let uploads_clone = uploads_coll.clone();

                tokio::spawn(async move {
                    if let Err(e) = handle_attachment_stream(send, recv, sessions_clone, uploads_clone).await {
                        warn!("Attachment stream error: {:?}", e);
                    }
                });
            }
            Err(e) => {
                warn!("No more streams or error: {:?}", e);
                break;
            }
        }
    }

    {
        let mut store = connections.lock().unwrap();
        store.remove(&conn_id);
    }

    Ok(())
}

// handle attachment connection stream (upload bytes into GridFS)
async fn handle_attachment_stream(
    mut _send: SendStream,
    mut recv: RecvStream,
    _sessions: SessionStore,
    uploads_coll: Collection<PendingUpload>,
) -> Result<()> {
    let mut buf = [0u8; 8192];
    let mut text_buf = String::new();

    // 1) read one header line as JSON
    let header_json = loop {
        let n = match recv.read(&mut buf).await {
            Ok(Some(n)) if n > 0 => n,
            Ok(_) => return Ok(()),
            Err(e) => {
                warn!("Attachment header read error: {:?}", e);
                return Ok(());
            }
        };

        let chunk = match std::str::from_utf8(&buf[..n]) {
            Ok(t) => t,
            Err(_) => {
                warn!("Attachment header non-UTF8");
                continue;
            }
        };

        text_buf.push_str(chunk);

        if let Some(pos) = text_buf.find('\n') {
            let line = text_buf[..pos].trim().to_string();
            text_buf.drain(..=pos);
            break line;
        }
    };

    #[derive(serde::Deserialize)]
    struct AttachHeader {
        upload_id: String,
        filename: String,
        mime_type: String,
        size_bytes: u64,
    }

    let header: AttachHeader = match serde_json::from_str(&header_json) {
        Ok(h) => h,
        Err(e) => {
            warn!(
                "Invalid attachment header JSON: {:?}, {:?}",
                e, header_json
            );
            return Ok(());
        }
    };

    // 2) find PendingUpload
    let pending = match uploads_coll
        .find_one(doc! { "upload_id": &header.upload_id })
        .await
    {
        Ok(Some(p)) => p,
        Ok(None) => {
            warn!("No PendingUpload for upload_id {}", header.upload_id);
            return Ok(());
        }
        Err(e) => {
            warn!("DB error loading PendingUpload: {:?}", e);
            return Ok(());
        }
    };

    // 3) get Database and GridFS bucket
    // 3) get Database and GridFS bucket
let ns = uploads_coll.namespace();
let db = uploads_coll.client().database(&ns.db);
let bucket: GridFsBucket = db.gridfs_bucket(None);

// Use upload_id as GridFS filename key: "attach:<upload_id>"
let gridfs_name = format!("attach:{}", header.upload_id);

// 4) open upload stream
let mut upload_stream = bucket
    .open_upload_stream(gridfs_name.clone())
    .await?;


    // any leftover bytes after header newline
    if !text_buf.is_empty() {
        upload_stream
            .write_all(text_buf.as_bytes())
            .await
            .map_err(|e| {
                warn!("GridFS write_all (header leftovers) error: {:?}", e);
                e
            })?;
        text_buf.clear();
    }

    // 5) stream rest of file
    loop {
        let n = match recv.read(&mut buf).await {
            Ok(Some(n)) if n > 0 => n,
            Ok(_) => break,
            Err(e) => {
                warn!("Attachment body read error: {:?}", e);
                break;
            }
        };

        upload_stream
            .write_all(&buf[..n])
            .await
            .map_err(|e| {
                warn!("GridFS write_all error: {:?}", e);
                e
            })?;
    }

    // 6) close upload stream (ignore return value, driver returns ())
    upload_stream.close().await.map_err(|e| {
        warn!("GridFS close error: {:?}", e);
        e
    })?;

    // 7) mark upload completed (no file_id stored)
    uploads_coll
        .update_one(
            doc! { "_id": &pending.id },
            doc! { "$set": { "completed": true } },
        )
        .await
        .map_err(|e| {
            warn!("DB update PendingUpload error: {:?}", e);
            e
        })?;

    Ok(())
}


// handle control stream
async fn handle_control_stream(
    mut send: SendStream,
    mut recv: RecvStream,
    sessions: SessionStore,
    connections: ConnectionStore,
    conn_id: u64,
    hb_interval: u64,
    start_time: SystemTime,
    mailbox_repo: Arc<MailboxRepository>,
    users_coll: Arc<Collection<UserDoc>>,
    uploads_coll: Collection<PendingUpload>,
    messages_coll: Collection<Message>,
    preferences_coll: Arc<Collection<PreferencesDoc>>,
    db: Arc<Database>,
    remote_addr: String,
) -> Result<()> {
    let mut heartbeat = interval(Duration::from_secs(hb_interval));
    let mut buf = [0u8; 8192];

    loop {
        tokio::select! {
            _ = heartbeat.tick() => {
                let hb = make_hb_response(start_time);
                if let Err(e) = send.write_all(hb.as_bytes()).await {
                    warn!("Failed to send heartbeat: {:?}", e);
                    break;
                }
            }

            result = recv.read(&mut buf) => {
                match result {
                    Ok(Some(n)) if n > 0 => {
                        let text = match std::str::from_utf8(&buf[..n]) {
                            Ok(t) => t.trim(),
                            Err(_) => {
                                warn!("Non-UTF8 data");
                                continue;
                            }
                        };

                        let response = process_command(
                            text,
                            &sessions,
                            &connections,
                            start_time,
                            &mailbox_repo,
                            &users_coll,
                            &uploads_coll,
                            &messages_coll,
                            &preferences_coll,
                            &db,
                            &remote_addr,
                        ).await;

                        if send.write_all(response.as_bytes()).await.is_err() {
                            break;
                        }
                    }
                    Ok(Some(_)) => break,
                    Ok(None) => break,
                    Err(e) => {
                        warn!("Read error: {:?}", e);
                        break;
                    }
                }
            }
        }
    }

    Ok(())
}

async fn process_command(
    text: &str,
    sessions: &SessionStore,
    connections: &ConnectionStore,
    start_time: SystemTime,
    mailbox_repo: &MailboxRepository,
    users_coll: &Collection<UserDoc>,
    uploads_coll: &Collection<PendingUpload>,
    messages_coll: &Collection<Message>,
    preferences_coll: &Collection<PreferencesDoc>,
    db: &Database,
    remote_addr: &str,
) -> String {
     eprintln!("[PROCESS_COMMAND] raw text: {}", text);

    let req = match Request::from_json(text) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("[PROCESS_COMMAND] PARSE ERROR: {:?}", e);
            return Response::err("PARSE", &format!("Invalid JSON: {}", e)).to_json();
        }
    };

    let command = req.cmd.to_uppercase();
    eprintln!("[PROCESS_COMMAND] cmd: {}", command);

    let token = req.data.get("session_token")
        .or_else(|| req.data.get("token"))
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();

    match command.as_str() {
        // Sessions
        cmd::INIT => init_handler::handle_init(&req, sessions).await,
        cmd::AUTH => auth_handler::handle_auth(&req, sessions, mailbox_repo, users_coll).await,
        cmd::RESUME => resume_handler::handle_resume(&req, sessions).await,
        cmd::LOGOUT => logout_handler::handle_logout(&req, sessions).await,
        cmd::SESSION_INFO => session_info_handler::handle_session_info(&token, sessions, &req).await,
        cmd::SESSION_LIST => session_list_handler::handle_session_list(&req, sessions).await,
        cmd::SESSION_KILL => session_kill_handler::handle_session_kill(&req, sessions).await,
        cmd::SESSION_SUSPEND => session_suspend_handler::handle_session_suspend(&token, sessions, &req).await,
        cmd::SESSION_RESUME_SUSPENDED => session_resume_suspended_handler::handle_session_resume_suspended(&token, sessions, &req).await,
        
        // Health
        cmd::PING => make_ping_response(start_time),
        cmd::PONG => pong_handler::handle_pong(&req).await,
        cmd::HB => hb_handler::handle_hb(&req).await,
        cmd::LATENCY_PING => make_latency_response(start_time),
        cmd::CONNECTION_INFO => connection_info_handler_v2::handle_connection_info(&req, remote_addr).await,
        cmd::RATE_LIMIT_INFO => Response::ok("RATE_LIMIT_INFO").with_msg("No active limits").to_json(),
        cmd::DEBUG_ECHO => debug_echo_handler::handle_debug_echo(&req).await,
        cmd::UPTIME => Response::ok("UPTIME").with_uptime(SystemTime::now().duration_since(start_time).unwrap_or(Duration::from_secs(0)).as_secs()).to_json(),
        
        // Diagnostics
        cmd::INFO => Response::ok("INFO").with_msg("WMTP Server v1.0.0").to_json(),
        cmd::STATUS => Response::ok("STATUS").with_msg("Running").to_json(),
        cmd::CAPABILITIES => capabilities_handler::handle_capabilities(&req).await,
        cmd::TIME => time_handler::handle_time(&req).await,
        cmd::VERSION_CHECK => version_check_handler::handle_version_check(&req).await,
        cmd::CONFIG_PUBLIC_GET => config_public_get_handler::handle_config_public_get(&req).await,
        
        // Mailbox
        cmd::MB_LIST => mb_list_handler::handle_mb_list(&token, sessions, mailbox_repo).await,
        cmd::MAIL_LIST => mail_list_handler::handle_mail_list(&token, sessions, mailbox_repo, &req).await,
        cmd::MB_CREATE => mb_create_handler::handle_mb_create(&token, sessions.clone(), mailbox_repo, &req).await,
        cmd::MB_RENAME => mb_rename_handler::handle_mb_rename(&token, sessions, mailbox_repo, &req).await,
        cmd::MB_DELETE => mb_delete_handler::handle_mb_delete(&token, sessions, mailbox_repo, &req).await,
        cmd::MB_INFO => mb_info_handler::handle_mb_info(&token, sessions, mailbox_repo, &req).await,
        cmd::MB_PURGE_TRASH => mb_purge_trash_handler::handle_mb_purge_trash(&req, sessions, mailbox_repo).await,
        cmd::MB_SUBSCRIBE => mb_subscribe_handler::handle_mb_subscribe(&req).await,
        cmd::MB_UNSUBSCRIBE => mb_unsubscribe_handler::handle_mb_unsubscribe(&req).await,
        
        // Messages
        cmd::MSG_SEND => msg_send_handler::handle_msg_send(&token, sessions, mailbox_repo, uploads_coll, &req).await,
        cmd::MSG_SEND_DRAFT => msg_send_draft_handler::handle_msg_send_draft(&token, sessions, mailbox_repo, uploads_coll, &req).await,
        cmd::MSG_LIST => msg_list_handler::handle_msg_list(&token, sessions, mailbox_repo, &req).await,
        cmd::MSG_GET => msg_get_handler::handle_msg_get(&token, sessions, mailbox_repo, &req).await,
        cmd::MSG_HEADERS => msg_headers_handler::handle_msg_headers(&token, sessions, mailbox_repo, &req).await,
        cmd::MSG_THREAD => msg_thread_handler::handle_msg_thread(&token, sessions, mailbox_repo, &req).await,
        cmd::MSG_MOVE => msg_move_handler::handle_msg_move(&token, sessions, mailbox_repo, &req).await,
        cmd::MSG_COPY => msg_copy_handler::handle_msg_copy(&token, sessions, mailbox_repo, &req).await,
        cmd::MSG_DELETE => msg_delete_handler::handle_msg_delete(&token, sessions, mailbox_repo, &req).await,
        cmd::MSG_EXPUNGE => msg_expunge_handler::handle_msg_expunge(&token, sessions, mailbox_repo, &req).await,
        cmd::MSG_UNDELETE => msg_undelete_handler::handle_msg_undelete(&token, sessions, mailbox_repo, &req).await,
        cmd::MSG_FLAG_SET => msg_flag_set_handler::handle_msg_flag_set(&token, sessions, mailbox_repo, &req).await,
        cmd::MSG_FLAG_CLEAR => msg_flag_clear_handler::handle_msg_flag_clear(&token, sessions, mailbox_repo, &req).await,
        cmd::MSG_BULK_ACTION => msg_bulk_action_handler::handle_msg_bulk_action(&token, sessions, mailbox_repo, &req).await,
        
        // Searches
        cmd::SEARCH => search_handler::handle_search_simple(&token, sessions, mailbox_repo, &req).await,
        cmd::SEARCH_GLOBAL => search_global_handler::handle_search_global(&token, sessions, mailbox_repo, &req).await,
        cmd::SEARCH_ADV => search_adv_handler::handle_search_adv(&token, sessions, mailbox_repo, &req).await,
        cmd::SEARCH_SUGGEST => search_suggest_handler::handle_search_suggest(&req).await,
        
        // Profile & User
        cmd::PROFILE_GET => profile_get_handler::handle_profile_get(&token, sessions, users_coll, &req).await,
        cmd::PROFILE_SET => profile_set_handler::handle_profile_set(&token, sessions, users_coll, &req).await,
        cmd::PREF_GET => handle_pref_get(&token, sessions, preferences_coll, &req).await,
        cmd::PREF_SET => handle_pref_set(&token, sessions, preferences_coll, &req).await,
        
        // Attachments
        cmd::ATTACH_UPLOAD_INIT => attach_upload_init_handler::handle_attach_upload_init(&req, sessions, uploads_coll).await,
        cmd::ATTACH_GET => attach_get_handler::handle_attach_get(&req, sessions, uploads_coll, db).await,
        
        // Admin
        cmd::CONNECTION_LIST => connection_list_handler::handle_connection_list(&req, connections).await,
        
        _ => Response::err("UNKNOWN", &format!("Unknown command: {}", command)).to_json(),
    }
}
