const COMMAND_SCHEMAS = {
  "INIT": {},
  "AUTH": {
    "email": {
      "type": "string",
      "required": true,
      "hint": "User email address"
    }
  },
  "RESUME": {
    "token": {
      "type": "string",
      "required": true,
      "hint": "Permanent session token"
    }
  },
  "LOGOUT": {
    "session_token": {
      "type": "string",
      "required": true,
      "hint": "Active session token"
    }
  },
  "SESSION_INFO": {
    "session_token": {
      "type": "string",
      "required": true,
      "hint": "Active session token"
    }
  },
  "SESSION_LIST": {
    "session_token": {
      "type": "string",
      "required": true,
      "hint": "Active session token"
    }
  },
  "SESSION_KILL": {
    "session_token": {
      "type": "string",
      "required": true,
      "hint": "Active session token"
    },
    "target_token": {
      "type": "string",
      "required": true,
      "hint": "The token to invalidate"
    }
  },
  "SESSION_SUSPEND": {
    "session_token": {
      "type": "string",
      "required": true,
      "hint": "Active session token"
    }
  },
  "SESSION_RESUME_SUSPENDED": {
    "session_token": {
      "type": "string",
      "required": true,
      "hint": "Current (or new) connection token"
    },
    "token": {
      "type": "string",
      "required": true,
      "hint": "The suspended token to resume"
    }
  },
  "PING": {},
  "PONG": {},
  "HB": {},
  "UPTIME": {},
  "LATENCY_PING": {},
  "CONNECTION_INFO": {},
  "RATE_LIMIT_INFO": {
    "session_token": {
      "type": "string",
      "required": true,
      "hint": "Active session token"
    }
  },
  "DEBUG_ECHO": {
    "session_token": {
      "type": "string",
      "required": false,
      "hint": "Optional token"
    },
    "...": {
      "type": "any",
      "required": false,
      "hint": "Arbitrary data to echo"
    }
  },
  "INFO": {},
  "STATUS": {},
  "CAPABILITIES": {},
  "TIME": {},
  "VERSION_CHECK": {},
  "CONFIG_PUBLIC_GET": {},
  "CONNECTION_LIST": {
    "session_token": {
      "type": "string",
      "required": true,
      "hint": "Admin session token"
    }
  },
  "MB_LIST": {
    "session_token": {
      "type": "string",
      "required": true,
      "hint": "Active session token"
    }
  },
  "MAIL_LIST": {
    "session_token": {
      "type": "string",
      "required": true,
      "hint": "Active session token"
    },
    "folder_code": {
      "type": "string",
      "required": true,
      "hint": "Folder code, e.g. INBOX, SENT, DRAFTS"
    },
    "limit": {
      "type": "number",
      "required": false,
      "hint": "Max messages to return (default: 20)"
    },
    "offset": {
      "type": "number",
      "required": false,
      "hint": "Skip count (default: 0)"
    }
  },
  "MB_CREATE": {
    "session_token": {
      "type": "string",
      "required": true,
      "hint": "Active session token"
    },
    "name": {
      "type": "string",
      "required": true,
      "hint": "Display name of the folder"
    },
    "code": {
      "type": "string",
      "required": false,
      "hint": "Folder code (auto-generated from name if omitted)"
    }
  },
  "MB_RENAME": {
    "session_token": {
      "type": "string",
      "required": true,
      "hint": "Active session token"
    },
    "folder": {
      "type": "string",
      "required": true,
      "hint": "Folder code to rename"
    },
    "new_name": {
      "type": "string",
      "required": true,
      "hint": "New display name"
    }
  },
  "MB_DELETE": {
    "session_token": {
      "type": "string",
      "required": true,
      "hint": "Active session token"
    },
    "folder": {
      "type": "string",
      "required": true,
      "hint": "Folder code to delete"
    }
  },
  "MB_INFO": {
    "session_token": {
      "type": "string",
      "required": true,
      "hint": "Active session token"
    },
    "folder_code": {
      "type": "string",
      "required": true,
      "hint": "Folder code to query"
    }
  },
  "MB_PURGE_TRASH": {
    "session_token": {
      "type": "string",
      "required": true,
      "hint": "Active session token"
    }
  },
  "MB_SUBSCRIBE": {
    "session_token": {
      "type": "string",
      "required": true,
      "hint": "Active session token"
    },
    "folder": {
      "type": "string",
      "required": true,
      "hint": "Folder code (e.g. INBOX)"
    }
  },
  "MB_UNSUBSCRIBE": {
    "session_token": {
      "type": "string",
      "required": true,
      "hint": "Active session token"
    },
    "folder": {
      "type": "string",
      "required": true,
      "hint": "Folder code (e.g. INBOX)"
    }
  },
  "MSG_SEND": {
    "session_token": {
      "type": "string",
      "required": true,
      "hint": "Active session token"
    },
    "to": {
      "type": "array[string]",
      "required": true,
      "hint": "List of recipient email addresses"
    },
    "subject": {
      "type": "string",
      "required": false,
      "hint": "Subject line"
    },
    "body": {
      "type": "string",
      "required": false,
      "hint": "Email body content"
    },
    "attachment_ids": {
      "type": "array[string]",
      "required": false,
      "hint": "Upload IDs for attachments"
    }
  },
  "MSG_SEND_DRAFT": {
    "session_token": {
      "type": "string",
      "required": true,
      "hint": "Active session token"
    },
    "to": {
      "type": "array[string]",
      "required": false,
      "hint": "Optional recipient list"
    },
    "subject": {
      "type": "string",
      "required": false,
      "hint": "Draft subject"
    },
    "body": {
      "type": "string",
      "required": false,
      "hint": "Draft body"
    }
  },
  "MSG_LIST": {
    "session_token": {
      "type": "string",
      "required": true,
      "hint": "Active session token"
    },
    "folder_code": {
      "type": "string",
      "required": true,
      "hint": "Folder code (e.g. INBOX, SENT, DRAFTS, BIN, SPAM)"
    },
    "limit": {
      "type": "number",
      "required": false,
      "hint": "Max messages to return (default: 20)"
    },
    "offset": {
      "type": "number",
      "required": false,
      "hint": "Skip count (default: 0)"
    }
  },
  "MSG_GET": {
    "session_token": {
      "type": "string",
      "required": true,
      "hint": "Active session token"
    },
    "id": {
      "type": "string",
      "required": true,
      "hint": "Message ObjectId (24-char hex)"
    }
  },
  "MSG_HEADERS": {
    "session_token": {
      "type": "string",
      "required": true,
      "hint": "Active session token"
    },
    "id": {
      "type": "string",
      "required": true,
      "hint": "Message unique ID"
    }
  },
  "MSG_THREAD": {
    "session_token": {
      "type": "string",
      "required": true,
      "hint": "Active session token"
    },
    "msg_id": {
      "type": "string",
      "required": true,
      "hint": "Message ID in the thread"
    }
  },
  "MSG_MOVE": {
    "session_token": {
      "type": "string",
      "required": true,
      "hint": "Active session token"
    },
    "id": {
      "type": "string",
      "required": true,
      "hint": "Message ID"
    },
    "target_folder": {
      "type": "string",
      "required": true,
      "hint": "Target folder code (e.g. TRASH)"
    }
  },
  "MSG_COPY": {
    "session_token": {
      "type": "string",
      "required": true,
      "hint": "Active session token"
    },
    "id": {
      "type": "string",
      "required": true,
      "hint": "Message ID"
    },
    "target_folder": {
      "type": "string",
      "required": true,
      "hint": "Target folder code"
    }
  },
  "MSG_DELETE": {
    "session_token": {
      "type": "string",
      "required": true,
      "hint": "Active session token"
    },
    "id": {
      "type": "string",
      "required": true,
      "hint": "Message ID"
    }
  },
  "MSG_EXPUNGE": {
    "session_token": {
      "type": "string",
      "required": true,
      "hint": "Active session token"
    },
    "id": {
      "type": "string",
      "required": true,
      "hint": "Message ID"
    }
  },
  "MSG_UNDELETE": {
    "session_token": {
      "type": "string",
      "required": true,
      "hint": "Active session token"
    },
    "id": {
      "type": "string",
      "required": true,
      "hint": "Message ID"
    },
    "target_folder": {
      "type": "string",
      "required": false,
      "hint": "Optional destination (defaults to INBOX)"
    }
  },
  "MSG_FLAG_SET": {
    "session_token": {
      "type": "string",
      "required": true,
      "hint": "Active session token"
    },
    "id": {
      "type": "string",
      "required": true,
      "hint": "Message ID"
    },
    "read": {
      "type": "bool",
      "required": false,
      "hint": "Set read status"
    },
    "starred": {
      "type": "bool",
      "required": false,
      "hint": "Set starred status"
    },
    "important": {
      "type": "bool",
      "required": false,
      "hint": "Set importance"
    }
  },
  "MSG_FLAG_CLEAR": {
    "session_token": {
      "type": "string",
      "required": true,
      "hint": "Active session token"
    },
    "id": {
      "type": "string",
      "required": true,
      "hint": "Message ID"
    },
    "read": {
      "type": "bool",
      "required": false,
      "hint": "Clear read status"
    },
    "starred": {
      "type": "bool",
      "required": false,
      "hint": "Clear starred status"
    },
    "important": {
      "type": "bool",
      "required": false,
      "hint": "Clear importance"
    }
  },
  "MSG_BULK_ACTION": {
    "session_token": {
      "type": "string",
      "required": true,
      "hint": "Active session token"
    },
    "ids": {
      "type": "array[string]",
      "required": true,
      "hint": "List of message IDs"
    },
    "action": {
      "type": "string",
      "required": true,
      "hint": "'MOVE', 'DELETE', 'EXPUNGE', 'FLAG_SET', 'FLAG_CLEAR'"
    },
    "target_folder": {
      "type": "string",
      "required": false,
      "hint": "Required for 'MOVE' action"
    },
    "flags": {
      "type": "object",
      "required": false,
      "hint": "Required for 'FLAG_SET'/'FLAG_CLEAR'"
    }
  },
  "SEARCH": {
    "session_token": {
      "type": "string",
      "required": true,
      "hint": "Active session token"
    },
    "q": {
      "type": "string",
      "required": false,
      "hint": "Search terms"
    },
    "folder_code": {
      "type": "string",
      "required": true,
      "hint": "Folder code to search"
    },
    "from": {
      "type": "string",
      "required": false,
      "hint": "Filter by sender email"
    }
  },
  "SEARCH_GLOBAL": {
    "session_token": {
      "type": "string",
      "required": true,
      "hint": "Active session token"
    },
    "q": {
      "type": "string",
      "required": false,
      "hint": "Search terms"
    },
    "from": {
      "type": "string",
      "required": false,
      "hint": "Filter by sender email"
    }
  },
  "SEARCH_ADV": {
    "session_token": {
      "type": "string",
      "required": true,
      "hint": "Active session token"
    },
    "folder_code": {
      "type": "string",
      "required": false,
      "hint": "Filter by folder"
    },
    "q": {
      "type": "string",
      "required": false,
      "hint": "Search terms"
    },
    "from": {
      "type": "string",
      "required": false,
      "hint": "Filter by sender"
    },
    "to": {
      "type": "string",
      "required": false,
      "hint": "Filter by recipient"
    },
    "unread": {
      "type": "boolean",
      "required": false,
      "hint": "Filter by read status"
    },
    "starred": {
      "type": "boolean",
      "required": false,
      "hint": "Filter by star status"
    },
    "important": {
      "type": "boolean",
      "required": false,
      "hint": "Filter by importance"
    },
    "date_from": {
      "type": "string",
      "required": false,
      "hint": "Start date (ISO 8601)"
    },
    "date_to": {
      "type": "string",
      "required": false,
      "hint": "End date (ISO 8601)"
    }
  },
  "SEARCH_SUGGEST": {
    "session_token": {
      "type": "string",
      "required": true,
      "hint": "Active session token"
    },
    "query": {
      "type": "string",
      "required": true,
      "hint": "Partial query string"
    }
  },
  "PROFILE_GET": {
    "session_token": {
      "type": "string",
      "required": true,
      "hint": "Active session token"
    }
  },
  "PROFILE_SET": {
    "session_token": {
      "type": "string",
      "required": true,
      "hint": "Active session token"
    },
    "name": {
      "type": "string",
      "required": false,
      "hint": "Display name (e.g. Sathya Gajula)"
    },
    "avatar_url": {
      "type": "string",
      "required": false,
      "hint": "Full URL to avatar image"
    },
    "timezone": {
      "type": "string",
      "required": false,
      "hint": "IANA timezone (e.g. Asia/Kolkata)"
    },
    "signature": {
      "type": "string",
      "required": false,
      "hint": "Email signature text"
    }
  },
  "PREF_GET": {
    "session_token": {
      "type": "string",
      "required": true,
      "hint": "Active session token"
    }
  },
  "PREF_SET": {
    "session_token": {
      "type": "string",
      "required": true,
      "hint": "Active session token"
    },
    "theme": {
      "type": "string",
      "required": false,
      "hint": "UI theme: 'light', 'dark', or 'system' (default: light)"
    },
    "notifications_enabled": {
      "type": "bool",
      "required": false,
      "hint": "Enable notifications: true or false (default: true)"
    },
    "language": {
      "type": "string",
      "required": false,
      "hint": "Language code (e.g. en, fr, de — default: en)"
    }
  },
  "ATTACH_UPLOAD_INIT": {
    "session_token": {
      "type": "string",
      "required": true,
      "hint": "Active session token"
    },
    "filename": {
      "type": "string",
      "required": true,
      "hint": "Original filename (e.g. document.pdf)"
    },
    "mime_type": {
      "type": "string",
      "required": false,
      "hint": "MIME type (default: application/octet-stream)"
    },
    "size_bytes": {
      "type": "integer",
      "required": false,
      "hint": "File size in bytes (e.g. 204800)"
    }
  },
  "ATTACH_GET": {
    "session_token": {
      "type": "string",
      "required": true,
      "hint": "Active session token"
    },
    "attachment_id": {
      "type": "string",
      "required": true,
      "hint": "The upload_id returned by ATTACH_UPLOAD_INIT (e.g. upl_abc123)"
    }
  }
};