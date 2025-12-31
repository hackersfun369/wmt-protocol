# WMTP Command Reference

This document provides the detailed JSON structure for all WMTP commands and their responses.

## 1. Session Management

### `INIT`
Initialize a new ephemeral session.

**Request:**
```json
{
  "cmd": "INIT"
}
```

**Response:**
```json
{
  "cmd": "SESSION_INIT",
  "status": "OK",
  "msg": "Session initialized",
  "session_token": "ephemeral-token-123",
  "auth": false
}
```

### `AUTH`
Authenticate an existing session with an email address.

**Request:**
```json
{
  "cmd": "AUTH",
  "data": {
    "email": "user@example.com"
  }
}
```

**Response:**
```json
{
  "cmd": "AUTH_OK",
  "status": "OK",
  "msg": "Authentication successful",
  "session_token": "stable-token-hash",
  "auth": true
}
```

### `RESUME`
Resume a session using a previously issued token.

**Request:**
```json
{
  "cmd": "RESUME",
  "data": {
    "token": "stable-token-hash"
  }
}
```

**Response:**
```json
{
  "cmd": "SESSION_RESUMED",
  "status": "OK",
  "msg": "Session resumed",
  "session_token": "stable-token-hash",
  "auth": true
}
```

### `LOGOUT`
Terminate the current session.

**Request:**
```json
{
  "cmd": "LOGOUT",
  "data": {
    "token": "session-token"
  }
}
```

**Response:**
```json
{
  "cmd": "LOGOUT_OK",
  "status": "OK",
  "msg": "Logged out"
}
```

## 2. Mailbox Operations

### `MB_LIST`
List all mailboxes (folders) for the authenticated user.

**Request:**
```json
{
  "cmd": "MB_LIST",
  "data": {
    "session_token": "session-token"
  }
}
```

**Response:**
```json
{
  "cmd": "MB_LIST",
  "status": "OK",
  "msg": "Mailbox folders",
  "folders": [
    {
      "code": "INBOX",
      "name": "Inbox",
      "total": 5,
      "unread": 2
    },
    {
      "code": "SENT",
      "name": "Sent Items",
      "total": 10,
      "unread": 0
    }
  ]
}
```

### `MB_CREATE`
Create a new custom folder.

**Request:**
```json
{
  "cmd": "MB_CREATE",
  "data": {
    "session_token": "session-token",
    "code": "WORK",
    "name": "Work Projects"
  }
}
```

**Response:**
```json
{
  "cmd": "MB_CREATE",
  "status": "OK",
  "msg": "Folder created successfully",
  "data": {
    "code": "WORK",
    "name": "Work Projects",
    "id": "folder-object-id"
  }
}
```

## 3. Message Operations

### `MSG_SEND`
Send a new message.

**Request:**
```json
{
  "cmd": "MSG_SEND",
  "data": {
    "session_token": "session-token",
    "to": ["recipient@example.com"],
    "subject": "Hello World",
    "body": "This is the message body."
  }
}
```

**Response:**
```json
{
  "cmd": "MSG_SEND",
  "status": "OK",
  "msg": "Message stored in Sent",
  "data": {
    "id": "message-object-id",
    "folder_code": "SENT",
    "from": "user@example.com",
    "to": ["recipient@example.com"],
    "subject": "Hello World"
  }
}
```

### `MSG_LIST`
List messages in a specific folder with pagination.

**Request:**
```json
{
  "cmd": "MSG_LIST",
  "data": {
    "session_token": "session-token",
    "folder_code": "INBOX",
    "offset": 0,
    "limit": 20
  }
}
```

**Response:**
```json
{
  "cmd": "MSG_LIST",
  "status": "OK",
  "msg": "Messages",
  "folder_code": "INBOX",
  "offset": 0,
  "limit": 20,
  "messages": [
    {
      "id": "message-id",
      "from": "sender@example.com",
      "to": ["user@example.com"],
      "subject": "Meeting",
      "snippet": "Hi, are we still on for...",
      "received_at": "2023-10-27T10:00:00Z",
      "unread": true
    }
  ]
}
```

### `MSG_GET`
Retrieve the full content of a specific message.

**Request:**
```json
{
  "cmd": "MSG_GET",
  "data": {
    "session_token": "session-token",
    "id": "message-id"
  }
}
```

**Response:**
```json
{
  "cmd": "MSG_GET",
  "status": "OK",
  "msg": "Message",
  "message": {
    "id": "message-id",
    "folder_code": "INBOX",
    "from": "sender@example.com",
    "to": ["user@example.com"],
    "subject": "Meeting",
    "body": "Full body content...",
    "received_at": "2023-10-27T10:00:00Z",
    "unread": false
  }
}
```

### `MSG_MOVE`
Move a message to a different folder.

**Request:**
```json
{
  "cmd": "MSG_MOVE",
  "data": {
    "session_token": "session-token",
    "id": "message-id",
    "target_folder": "ARCHIVE"
  }
}
```

**Response:**
```json
{
  "cmd": "MSG_MOVE",
  "status": "OK",
  "msg": "Message moved",
  "data": {
    "id": "message-id",
    "from_folder": "INBOX",
    "to_folder": "ARCHIVE"
  }
}
```

## 4. Search

### `SEARCH`
Search for messages within a specific folder.

**Request:**
```json
{
  "cmd": "SEARCH",
  "data": {
    "session_token": "session-token",
    "folder_code": "INBOX",
    "q": "meeting",
    "from": "boss@example.com",
    "offset": 0,
    "limit": 50
  }
}
```

**Response:**
```json
{
  "cmd": "SEARCH",
  "status": "OK",
  "msg": "Search results",
  "folder_code": "INBOX",
  "offset": 0,
  "limit": 50,
  "messages": [
    {
      "id": "message-id",
      "subject": "Important Meeting",
      "..." : "..."
    }
  ]
}
```

## 5. Profile

### `PROFILE_GET`
Get the authenticated user's profile.

**Request:**
```json
{
  "cmd": "PROFILE_GET",
  "data": {
    "session_token": "session-token"
  }
}
```

**Response:**
```json
{
  "cmd": "PROFILE_GET",
  "status": "OK",
  "msg": "User profile",
  "profile": {
    "id": "user-id",
    "email": "user@example.com",
    "name": "John Doe",
    "created_at": "2023-01-01T00:00:00Z"
  }
}
```

## 6. Attachments

### `ATTACH_UPLOAD_INIT`
Initialize an attachment upload.

**Request:**
```json
{
  "cmd": "ATTACH_UPLOAD_INIT",
  "data": {
    "session_token": "session-token",
    "filename": "document.pdf",
    "mime_type": "application/pdf",
    "size_bytes": 102400
  }
}
```

**Response:**
```json
{
  "cmd": "ATTACH_UPLOAD_INIT",
  "status": "OK",
  "msg": "Attachment upload initialized",
  "data": {
    "upload": {
      "upload_id": "upl_AbCdEfGhIjKlMnOp",
      "filename": "document.pdf",
      "mime_type": "application/pdf",
      "size_bytes": 102400
    }
  }
}
```
*Note: After receiving this response, the client must open a new bidirectional stream, send the JSON header followed by a newline, and then stream the binary data.*

## 7. Connections

### `CONNECTION_LIST`
List active WebTransport connections (Admin/Debug).

**Request:**
```json
{
  "cmd": "CONNECTION_LIST"
}
```

**Response:**
```json
{
  "cmd": "CONNECTION_LIST",
  "status": "OK",
  "msg": "Active connections",
  "total": 1,
  "connections": [
    {
      "id": 1,
      "remote_addr": "127.0.0.1:54321",
      "session_token": "token...",
      "authenticated": true
    }
  ]
}
```
