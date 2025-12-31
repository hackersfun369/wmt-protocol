use futures_util::stream::TryStreamExt;
use mongodb::bson::{self, doc, oid::ObjectId, DateTime as BsonDateTime};
use mongodb::{Collection, Database};
use serde::{Deserialize, Serialize};
use mongodb::error::Error as MongoError;
use mongodb::bson::Regex;


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Folder {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    #[serde(rename = "userId", skip_serializing_if = "Option::is_none")]
    pub user_id: Option<ObjectId>,         // None for global (INBOX/SENT), Some for user-specific
    pub code: String,                      // "INBOX", "SENT", "DRAFTS", "BIN", "SPAM", or custom
    pub name: String,                      // "Inbox", ...
    #[serde(rename = "createdAt", skip_serializing_if = "Option::is_none")]
    pub created_at: Option<BsonDateTime>,  // only set for custom folders
}


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Message {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    #[serde(rename = "userId")]
    pub user_id: ObjectId,
    #[serde(rename = "folderCode")]
    pub folder_code: String,
    pub from: String,
    pub to: Vec<String>,
    pub subject: String,
    pub snippet: Option<String>,
    pub body: Option<String>,
    #[serde(rename = "receivedAt")]
    pub received_at: BsonDateTime,
    pub unread: bool,
    pub starred: bool,
    pub important: bool,
    #[serde(default)]
    pub attachments: Vec<AttachmentMeta>,   // new
}

// attachment metadata
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AttachmentMeta {
    pub id: String,         // your logical attachment id (string)
    pub filename: String,
    pub mime_type: String,
    pub size_bytes: u64,
    pub gridfs_id: String,  // GridFS file _id as hex string (ObjectId) or plain string
}

// folder info for MB_FOLDER_INFO
#[derive(Debug, Serialize)]
pub struct MbFolderInfoDto {
    pub code: String,
    pub name: String,
    pub unread: u64,
    pub total: u64,
    pub latest_subject: Option<String>,
    pub latest_from: Option<String>,
    pub latest_received_at: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct MbListFolderDto {
    pub code: String,
    pub name: String,
    pub unread: u64,
    pub total: u64,
}

pub struct MailboxRepository {
    folders: Collection<Folder>,
    messages: Collection<Message>,
}

impl MailboxRepository {
    pub fn new(db: &Database) -> Self {
        Self {
            folders: db.collection::<Folder>("folders"),
            messages: db.collection::<Message>("messages"),
        }
    }

    /// Ensure global fixed folders exist once (no userId).
   pub async fn ensure_default_folders_global(&self) -> mongodb::error::Result<()> {
    let codes = [
        ("INBOX", "Inbox"),
        ("SENT", "Sent"),
        ("DRAFTS", "Drafts"),
        ("BIN", "Bin"),
        ("SPAM", "Spam"),
    ];

    for (code, name) in codes.iter() {
        let existing = self
            .folders
            .find_one(doc! { "code": code })
            .await?;
        if existing.is_none() {
            let folder = Folder {
                id: ObjectId::new(),
                user_id: None,
                code: code.to_string(),
                name: name.to_string(),
                created_at: None,
            };
            self.folders.insert_one(folder).await?;
        }
    }
    Ok(())
}


    /// List messages for a user in one folderCode with paging (using aggregate).
    pub async fn list_messages(
        &self,
        user_id: &ObjectId,
        folder_code: &str,
        offset: u64,
        limit: u64,
    ) -> mongodb::error::Result<Vec<Message>> {
        let pipeline = vec![
            doc! {
                "$match": {
                    "userId": user_id,
                    "folderCode": folder_code,
                }
            },
            doc! { "$sort": { "receivedAt": -1 } },
            doc! { "$skip": i64::try_from(offset).unwrap_or(0) },
            doc! { "$limit": i64::try_from(limit).unwrap_or(20) },
        ];

        let mut cursor = self.messages.aggregate(pipeline).await?;
        let mut msgs = Vec::new();
        while let Some(doc) = cursor.try_next().await? {
            let m: Message = bson::from_document(doc)?;
            msgs.push(m);
        }
        Ok(msgs)
    }

    /// Return MB_LIST data for one user (count by userId + folderCode).
    pub async fn get_mb_list_for_user(
        &self,
        user_id: &ObjectId,
    ) -> mongodb::error::Result<Vec<MbListFolderDto>> {
        // Load all global folders
        let mut cursor = self.folders.find(doc! {}).await?;
        let mut result = Vec::new();

        while let Some(folder) = cursor.try_next().await? {
            let code = &folder.code;

            let total = self
                .messages
                .count_documents(
                    doc! {
                        "userId": user_id,
                        "folderCode": code,
                    }
                )
                .await?;

            let unread = self
                .messages
                .count_documents(
                    doc! {
                        "userId": user_id,
                        "folderCode": code,
                        "unread": true,
                    }
                )
                .await?;

            result.push(MbListFolderDto {
                code: folder.code,
                name: folder.name,
                unread,
                total,
            });
        }

        Ok(result)
    }

    /// Insert new incoming mail; always into INBOX for that user.
    pub async fn insert_incoming_into_inbox(
        &self,
        user_id: &ObjectId,
        from: &str,
        to: Vec<String>,
        subject: &str,
        body: &str,
    ) -> mongodb::error::Result<ObjectId> {
        let snippet = body.chars().take(100).collect::<String>();

        let message = Message {
            id: ObjectId::new(),
            user_id: user_id.clone(),
            folder_code: "INBOX".to_string(),
            from: from.to_string(),
            to,
            subject: subject.to_string(),
            snippet: Some(snippet),
            body: Some(body.to_string()),
            received_at: BsonDateTime::now(),
            unread: true,
            starred: false,
            important: false,
            attachments: Vec::new(),

        };

        let id = message.id;
        self.messages.insert_one(message).await?;
        Ok(id)
    }

    /// Move a message to another folder code (e.g. BIN, SPAM).
    pub async fn move_message_to_folder_code(
        &self,
        user_id: &ObjectId,
        msg_id: &ObjectId,
        target_code: &str,
    ) -> mongodb::error::Result<()> {
        self.messages
            .update_one(
                doc! { "_id": msg_id, "userId": user_id },
                doc! { "$set": { "folderCode": target_code } },
            )
            .await?;

        Ok(())
    }

    /// Create custom folder for a user (beyond fixed INBOX/SENT/etc).
    /// Create custom folder for a user (beyond fixed INBOX/SENT/etc).
pub async fn create_user_folder(
    &self,
    user_id: ObjectId,
    code: &str,
    name: &str,
) -> Result<ObjectId, MongoError> {
    // Check if folder with same code already exists for this user
    if self
        .folders
        .find_one(doc! { "userId": &user_id, "code": code })
        .await?
        .is_some()
    {
        return Err(MongoError::custom("Folder exists"));
    }

    let folder = Folder {
        id: ObjectId::new(),
        user_id: Some(user_id.clone()),
        code: code.to_string(),
        name: name.to_string(),
        created_at: Some(BsonDateTime::now()),
    };

    self.folders.insert_one(&folder).await?;
    Ok(folder.id)
}

pub async fn rename_user_folder(
    &self,
    user_id: &ObjectId,
    folder_code: &str,
    new_name: &str,
) -> Result<u64, MongoError> {
    let res = self.folders.update_one(
        doc! { "userId": user_id, "code": folder_code },
        doc! { "$set": { "name": new_name } },
    ).await?;
    Ok(res.modified_count)
}

pub async fn delete_user_folder(
    &self,
    user_id: &ObjectId,
    folder_code: &str,
) -> Result<u64, MongoError> {
    // 1) Delete messages in this folder first? Or just the folder?
    // Usually recommended to move messages to BIN or just delete.
    // For now, let's just delete the folder metadata.
    let res = self.folders.delete_one(
        doc! { "userId": user_id, "code": folder_code },
    ).await?;
    Ok(res.deleted_count)
}

// Folder info for particular user
pub async fn get_folder_info_for_user(
        &self,
        user_id: &ObjectId,
        folder_code: &str,
    ) -> mongodb::error::Result<MbFolderInfoDto> {
        // Load folder meta (global or user-specific)
        let folder = self
            .folders
            .find_one(doc! { "code": folder_code })
            .await?
            .ok_or_else(|| mongodb::error::Error::custom("Folder not found"))?;

        // Counts
        let total = self
            .messages
            .count_documents(
                doc! {
                    "userId": user_id,
                    "folderCode": folder_code,
                }
            )
            .await?;

        let unread = self
            .messages
            .count_documents(
                doc! {
                    "userId": user_id,
                    "folderCode": folder_code,
                    "unread": true,
                }
            )
            .await?;

        // Latest message (subject, from, timestamp)
        let mut cursor = self
            .messages
            .aggregate(vec![
                doc! {
                    "$match": {
                        "userId": user_id,
                        "folderCode": folder_code,
                    }
                },
                doc! { "$sort": { "receivedAt": -1 } },
                doc! { "$limit": 1 },
            ])
            .await?;

        let (latest_subject, latest_from, latest_received_at) =
    if let Some(doc) = cursor.try_next().await? {
        let m: Message = bson::from_document(doc)?;
        let ts = m.received_at.to_system_time();
        let dt: chrono::DateTime<chrono::Utc> = ts.into();
        (
            Some(m.subject),
            Some(m.from),
            Some(dt.to_rfc3339()),
        )
    } else {
        (None, None, None)
    };


        Ok(MbFolderInfoDto {
            code: folder.code,
            name: folder.name,
            unread,
            total,
            latest_subject,
            latest_from,
            latest_received_at,
        })
    }

    // clear the BIN folder for a user
    pub async fn purge_trash_for_user(
    &self,
    user_id: &ObjectId,
) -> mongodb::error::Result<u64> {
    let result = self
        .messages
        .delete_many(
            doc! {
                "userId": user_id,
                "folderCode": "BIN",
            }
        )
        .await?;
    Ok(result.deleted_count)
}

// messages
pub async fn insert_sent_message(
    &self,
    user_id: &ObjectId,
    from: &str,
    to: Vec<String>,
    subject: &str,
    body: &str,
    attachments: Vec<AttachmentMeta>,
) -> mongodb::error::Result<ObjectId> {
    let snippet = body.chars().take(100).collect::<String>();

    let message = Message {
        id: ObjectId::new(),
        user_id: user_id.clone(),
        folder_code: "SENT".to_string(),
        from: from.to_string(),
        to,
        subject: subject.to_string(),
        snippet: Some(snippet),
        body: Some(body.to_string()),
        received_at: BsonDateTime::now(), // acts as sent time
        unread: false,
        starred: false,
        important: false,
        attachments,
    };

    let id = message.id;
    self.messages.insert_one(message).await?;
    Ok(id)
}

// save the draft message
    pub async fn insert_draft_message(
        &self,
        user_id: &ObjectId,
        from: &str,
        to: Vec<String>,
        subject: &str,
        body: &str,
        attachments: Vec<AttachmentMeta>,
    ) -> mongodb::error::Result<ObjectId> {
        let snippet = body.chars().take(100).collect::<String>();
    
        let message = Message {
            id: ObjectId::new(),
            user_id: user_id.clone(),
            folder_code: "DRAFTS".to_string(),
            from: from.to_string(),
            to,
            subject: subject.to_string(),
            snippet: Some(snippet),
            body: Some(body.to_string()),
            received_at: BsonDateTime::now(),
            unread: false, // drafts are usually not "unread"
            starred: false,
            important: false,
            attachments,
        };

    let id = message.id;
    self.messages.insert_one(message).await?;
    Ok(id)
}

// get message by id
pub async fn get_message_for_user(
        &self,
        user_id: &ObjectId,
        msg_id: &ObjectId,
    ) -> mongodb::error::Result<Option<Message>> {
        let msg = self
            .messages
            .find_one(
                doc! {
                    "_id": msg_id,
                    "userId": user_id,
                }
            )
            .await?;
        Ok(msg)
    }

    // msg_headers
    pub async fn get_message_headers_for_user(
    &self,
    user_id: &ObjectId,
    msg_id: &ObjectId,
) -> mongodb::error::Result<Option<Message>> {
    // Using aggregation with $match + $project to drop body field
    let pipeline = vec![
        doc! {
            "$match": {
                "_id": msg_id,
                "userId": user_id,
            }
        },
        doc! {
            "$project": {
                "body": 0 // exclude body to keep it light
            }
        },
    ];

    let mut cursor = self.messages.aggregate(pipeline).await?;
    if let Some(doc) = cursor.try_next().await? {
        let m: Message = bson::from_document(doc)?;
        Ok(Some(m))
    } else {
        Ok(None)
    }
}

// move message to custom folder
pub async fn move_message_for_user(
    &self,
    user_id: &ObjectId,
    msg_id: &ObjectId,
    target_folder: &str,
) -> mongodb::error::Result<Option<String>> {
    // find current folderCode
    let existing = self
        .messages
        .find_one(
            doc! {
                "_id": msg_id,
                "userId": user_id,
            }
        )
        .await?;

    let from_folder = match existing {
        Some(m) => m.folder_code,
        None => return Ok(None),
    };

    let res = self
        .messages
        .update_one(
            doc! {
                "_id": msg_id,
                "userId": user_id,
            },
            doc! {
                "$set": { "folderCode": target_folder }
            },
        )
        .await?;

    if res.matched_count == 0 {
        return Ok(None);
    }

    Ok(Some(from_folder))
}

// copy message to another folder
pub async fn copy_message_for_user(
    &self,
    user_id: &ObjectId,
    msg_id: &ObjectId,
    target_folder: &str,
) -> mongodb::error::Result<Option<(String, ObjectId)>> {
    // 1) load source message (owned by this user)
    let src = match self
        .messages
        .find_one(
            doc! {
                "_id": msg_id,
                "userId": user_id,
            }
        )
        .await?
    {
        Some(m) => m,
        None => return Ok(None),
    };

    let source_folder = src.folder_code.clone();

    // 2) build new Message with new _id and target folder
    let new_id = ObjectId::new();

    let copy_msg = Message {
        id: new_id,
        user_id: src.user_id,          // same owner
        folder_code: target_folder.to_string(),
        from: src.from,
        to: src.to,
        subject: src.subject,
        snippet: src.snippet,
        body: src.body,
        received_at: src.received_at,  // same timestamp
        unread: src.unread,
        starred: src.starred,
        important: src.important,
        attachments: src.attachments.clone(),
    };

    self.messages.insert_one(copy_msg).await?;  // insert Message, not Document
    Ok(Some((source_folder, new_id)))
}

// delete the message for a user
pub async fn soft_delete_message_for_user(
    &self,
    user_id: &ObjectId,
    msg_id: &ObjectId,
) -> mongodb::error::Result<Option<String>> {
    // load to get current folder
    let existing = self
        .messages
        .find_one(
            doc! {
                "_id": msg_id,
                "userId": user_id,
            }
        )
        .await?;

    let from_folder = match existing {
        Some(m) => m.folder_code,
        None => return Ok(None),
    };

    let res = self
        .messages
        .update_one(
            doc! {
                "_id": msg_id,
                "userId": user_id,
            },
            doc! {
                "$set": { "folderCode": "BIN" }
            },
        )
        .await?;

    if res.matched_count == 0 {
        return Ok(None);
    }

    Ok(Some(from_folder))
}

// delete messages permanently
pub async fn hard_delete_message_for_user(
    &self,
    user_id: &ObjectId,
    msg_id: &ObjectId,
) -> mongodb::error::Result<u64> {
    let res = self
        .messages
        .delete_one(
            doc! {
                "_id": msg_id,
                "userId": user_id,
            }
        )
        .await?;
    Ok(res.deleted_count)
}

// message undelete
pub async fn undelete_message_for_user(
    &self,
    user_id: &ObjectId,
    msg_id: &ObjectId,
    target_folder: &str,
) -> mongodb::error::Result<Option<String>> {
    // Only restore if currently in BIN
    let existing = self
        .messages
        .find_one(
            doc! {
                "_id": msg_id,
                "userId": user_id,
                "folderCode": "BIN",
            }
        )
        .await?;

    let from_folder = match existing {
        Some(m) => m.folder_code,
        None => return Ok(None), // not found or not in BIN
    };

    let res = self
        .messages
        .update_one(
            doc! {
                "_id": msg_id,
                "userId": user_id,
                "folderCode": "BIN",
            },
            doc! {
                "$set": { "folderCode": target_folder }
            },
        )
        .await?;

    if res.matched_count == 0 {
        return Ok(None);
    }

    Ok(Some(from_folder))
}

// set flags for a message
pub async fn set_flags_for_message(
    &self,
    user_id: &ObjectId,
    msg_id: &ObjectId,
    read: Option<bool>,
    starred: Option<bool>,
    important: Option<bool>,
) -> mongodb::error::Result<u64> {
    let mut set_doc = bson::Document::new();

    if let Some(read_val) = read {
        // store as unread internally
        set_doc.insert("unread", !read_val);
    }
    if let Some(starred_val) = starred {
        set_doc.insert("starred", starred_val);
    }
    if let Some(important_val) = important {
        set_doc.insert("important", important_val);
    }

    if set_doc.is_empty() {
        return Ok(0);
    }

    let res = self
        .messages
        .update_one(
            doc! {
                "_id": msg_id,
                "userId": user_id,
            },
            doc! {
                "$set": set_doc
            },
        )
        .await?;

    Ok(res.modified_count)
}

pub async fn bulk_move_for_user(
    &self,
    user_id: &ObjectId,
    ids: &[ObjectId],
    target_folder: &str,
) -> mongodb::error::Result<u64> {
    let res = self
        .messages
        .update_many(
            doc! {
                "userId": user_id,
                "_id": { "$in": ids },
            },
            doc! {
                "$set": { "folderCode": target_folder }
            },
        )
        .await?;
    Ok(res.modified_count)
}

pub async fn bulk_soft_delete_for_user(
    &self,
    user_id: &ObjectId,
    ids: &[ObjectId],
) -> mongodb::error::Result<u64> {
    self.bulk_move_for_user(user_id, ids, "BIN").await
}

pub async fn bulk_expunge_for_user(
    &self,
    user_id: &ObjectId,
    ids: &[ObjectId],
) -> mongodb::error::Result<u64> {
    let res = self
        .messages
        .delete_many(
            doc! {
                "userId": user_id,
                "_id": { "$in": ids },
            }
        )
        .await?;
    Ok(res.deleted_count)
}

pub async fn bulk_set_flags_for_user(
    &self,
    user_id: &ObjectId,
    ids: &[ObjectId],
    read: Option<bool>,
    starred: Option<bool>,
    important: Option<bool>,
) -> mongodb::error::Result<u64> {
    let mut set_doc = bson::Document::new();

    if let Some(read_val) = read {
        set_doc.insert("unread", !read_val);
    }
    if let Some(starred_val) = starred {
        set_doc.insert("starred", starred_val);
    }
    if let Some(important_val) = important {
        set_doc.insert("important", important_val);
    }

    if set_doc.is_empty() {
        return Ok(0);
    }

    let res = self
        .messages
        .update_many(
            doc! {
                "userId": user_id,
                "_id": { "$in": ids },
            },
            doc! {
                "$set": set_doc
            },
        )
        .await?;
    Ok(res.modified_count)
}

// search
pub async fn search_messages_simple(
    &self,
    user_id: &ObjectId,
    folder_code: &str,
    q: Option<&str>,
    from_filter: Option<&str>,
    offset: u64,
    limit: u64,
) -> mongodb::error::Result<Vec<Message>> {
    let mut filter = doc! {
        "userId": user_id,
        "folderCode": folder_code,
    };

    if let Some(from_val) = from_filter {
        if !from_val.is_empty() {
            filter.insert("from", from_val);
        }
    }

    if let Some(query) = q {
        if !query.is_empty() {
            let regex = Regex {
                pattern: query.to_string(),
                options: "i".to_string(), // case-insensitive
            };
            filter.insert(
                "$or",
                vec![
                    doc! { "subject": { "$regex": &regex } },
                    doc! { "snippet": { "$regex": &regex } },
                ],
            );
        }
    }

    let mut cursor = self
        .messages
        .find(filter)
        .sort(doc! { "receivedAt": -1 })
        .skip(u64::try_from(offset).unwrap_or(0))
        .limit(i64::try_from(limit).unwrap_or(50))
        .await?;

    let mut results = Vec::new();
    while let Some(m) = cursor.try_next().await? {
        results.push(m);
    }

    Ok(results)
}

// global search
pub async fn search_messages_global_simple(
    &self,
    user_id: &ObjectId,
    q: Option<&str>,
    from_filter: Option<&str>,
    offset: u64,
    limit: u64,
) -> mongodb::error::Result<Vec<Message>> {
    let mut filter = doc! {
        "userId": user_id,
    };

    if let Some(from_val) = from_filter {
        if !from_val.is_empty() {
            filter.insert("from", from_val);
        }
    }

    if let Some(query) = q {
        if !query.is_empty() {
            let regex = Regex {
                pattern: query.to_string(),
                options: "i".to_string(),
            };
            filter.insert(
                "$or",
                vec![
                    doc! { "subject": { "$regex": &regex } },
                    doc! { "snippet": { "$regex": &regex } },
                ],
            );
        }
    }

    let mut cursor = self
        .messages
        .find(filter)
        .sort(doc! { "receivedAt": -1 })
        .skip(u64::try_from(offset).unwrap_or(0))
        .limit(i64::try_from(limit).unwrap_or(50))
        .await?;

    let mut results = Vec::new();
    while let Some(m) = cursor.try_next().await? {
        results.push(m);
    }

    Ok(results)
}

// search advanced
pub async fn search_messages_advanced(
    &self,
    user_id: &ObjectId,
    folder_code: Option<&str>,
    q: Option<&str>,
    from_filter: Option<&str>,
    to_filter: Option<&str>,
    unread: Option<bool>,
    starred: Option<bool>,
    important: Option<bool>,
    date_from: Option<BsonDateTime>,
    date_to: Option<BsonDateTime>,
    offset: u64,
    limit: u64,
) -> mongodb::error::Result<Vec<Message>> {
    let mut filter = doc! {
        "userId": user_id,
    };

    if let Some(fc) = folder_code {
        if !fc.is_empty() {
            filter.insert("folderCode", fc);
        }
    }

    if let Some(from_val) = from_filter {
        if !from_val.is_empty() {
            filter.insert("from", from_val);
        }
    }

    if let Some(to_val) = to_filter {
        if !to_val.is_empty() {
            let regex = Regex {
                pattern: to_val.to_string(),
                options: "i".to_string(),
            };
            filter.insert("to", doc! { "$regex": &regex });
        }
    }

    if let Some(unread_val) = unread {
        filter.insert("unread", unread_val);
    }
    if let Some(starred_val) = starred {
        filter.insert("starred", starred_val);
    }
    if let Some(important_val) = important {
        filter.insert("important", important_val);
    }

    // date range on receivedAt
    if date_from.is_some() || date_to.is_some() {
        let mut date_cond = doc! {};
        if let Some(df) = date_from {
            date_cond.insert("$gte", df);
        }
        if let Some(dt) = date_to {
            date_cond.insert("$lte", dt);
        }
        filter.insert("receivedAt", date_cond);
    }

    // text query on subject + snippet
    if let Some(query) = q {
        if !query.is_empty() {
            let regex = Regex {
                pattern: query.to_string(),
                options: "i".to_string(),
            };
            filter.insert(
                "$or",
                vec![
                    doc! { "subject": { "$regex": &regex } },
                    doc! { "snippet": { "$regex": &regex } },
                ],
            );
        }
    }

    let mut cursor = self
        .messages
        .find(filter)
        .sort(doc! { "receivedAt": -1 })
        .skip(u64::try_from(offset).unwrap_or(0))
        .limit(i64::try_from(limit).unwrap_or(50))
        .await?;

    let mut results = Vec::new();
    while let Some(m) = cursor.try_next().await? {
        results.push(m);
    }

    Ok(results)
}

}
