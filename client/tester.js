const COMMAND_SCHEMAS = {
    // Session
    INIT: {},
    AUTH: { email: { type: 'string', required: true } },
    RESUME: { token: { type: 'string', required: true } },
    LOGOUT: { token: { type: 'string', required: false } },
    SESSION_INFO: {},
    SESSION_LIST: {},
    SESSION_KILL: { token: { type: 'string', required: true } },
    SESSION_SUSPEND: {},
    SESSION_RESUME_SUSPENDED: { token: { type: 'string', required: true } },

    // Health & Admin
    HB: {},
    PING: {},
    PONG: {},
    TIME: {},
    VERSION_CHECK: { client_version: { type: 'string', required: true, default: '1.0.0' } },
    CAPABILITIES: {},
    CONFIG_PUBLIC_GET: {},
    CONNECTION_INFO: {},
    LIST: {},
    DEBUG_ECHO: { message: { type: 'string', required: true } },

    // Mailbox
    MB_LIST: {},
    MB_CREATE: { name: { type: 'string', required: true }, code: { type: 'string', required: false } },
    MB_RENAME: { old_name: { type: 'string', required: true }, new_name: { type: 'string', required: true } },
    MB_DELETE: { name: { type: 'string', required: true } },
    MB_INFO: { name: { type: 'string', required: true } },
    MB_PURGE_TRASH: {},
    MB_SUBSCRIBE: { folder_code: { type: 'string', required: true } },
    MB_UNSUBSCRIBE: { folder_code: { type: 'string', required: true } },

    // Message
    MSG_SEND: {
        to: { type: 'array', required: true, hint: 'Comma separated emails' },
        cc: { type: 'array', required: false },
        bcc: { type: 'array', required: false },
        subject: { type: 'string', required: true },
        body: { type: 'text', required: true },
        in_reply_to: { type: 'string', required: false }
    },
    MSG_SEND_DRAFT: {
        to: { type: 'array', required: false },
        subject: { type: 'string', required: false },
        body: { type: 'text', required: true }
    },
    MSG_LIST: {
        folder_code: { type: 'string', required: true, default: 'INBOX' },
        offset: { type: 'number', required: false, default: 0 },
        limit: { type: 'number', required: false, default: 50 },
        sort: { type: 'string', required: false, default: 'desc' }
    },
    MSG_GET: { id: { type: 'string', required: true } },
    MSG_HEADERS: { id: { type: 'string', required: true } },
    MSG_THREAD: { thread_id: { type: 'string', required: true } },
    MSG_MOVE: { id: { type: 'string', required: true }, target_folder: { type: 'string', required: true } },
    MSG_COPY: { id: { type: 'string', required: true }, target_folder: { type: 'string', required: true } },
    MSG_DELETE: { id: { type: 'string', required: true } },
    MSG_UNDELETE: { id: { type: 'string', required: true } },
    MSG_EXPUNGE: { id: { type: 'string', required: true } },
    MSG_FLAG_SET: { id: { type: 'string', required: true }, flag: { type: 'string', required: true, default: 'UNREAD' } },
    MSG_FLAG_CLEAR: { id: { type: 'string', required: true }, flag: { type: 'string', required: true, default: 'UNREAD' } },
    MSG_BULK_ACTION: {
        ids: { type: 'array', required: true, hint: 'Comma separated IDs' },
        action: { type: 'string', required: true, hint: 'MOVE, DELETE, etc.' },
        target_folder: { type: 'string', required: false }
    },

    // Search
    SEARCH_GLOBAL: { query: { type: 'string', required: true } },
    SEARCH_SIMPLE: { query: { type: 'string', required: true }, folder_code: { type: 'string', required: false } },
    SEARCH_ADV: {
        query: { type: 'string', required: false },
        from: { type: 'string', required: false },
        to: { type: 'string', required: false },
        subject: { type: 'string', required: false },
        has_attachment: { type: 'boolean', required: false }
    },
    SEARCH_SUGGEST: { query: { type: 'string', required: true } },

    // Profile
    PROFILE_GET: {},
    PROFILE_SET: {
        name: { type: 'string', required: false },
        signature: { type: 'text', required: false },
        timezone: { type: 'string', required: false },
        avatar_url: { type: 'string', required: false }
    },

    // Preferences
    PREF_GET: { key: { type: 'string', required: false, hint: 'Leave empty for all' } },
    PREF_SET: { key: { type: 'string', required: true }, value: { type: 'string', required: true } },

    // Attachments
    ATTACH_UPLOAD_INIT: {
        filename: { type: 'string', required: true },
        mime_type: { type: 'string', required: true, default: 'application/octet-stream' },
        size_bytes: { type: 'number', required: true }
    },
    ATTACH_GET: { attachment_id: { type: 'string', required: true } },
};

let currentCmd = 'INIT';
let sessionToken = null;

// DOM Elements
const sidebar = document.getElementById('sidebar');
const sidebarCmds = document.getElementById('sidebar-cmds');
const resizer = document.getElementById('resizer');
const formPanel = document.getElementById('form-panel');
const activeCmdName = document.getElementById('active-cmd-name');
const statusDot = document.getElementById('status-dot');
const statusText = document.getElementById('status-text');
const sessionEmail = document.getElementById('session-email');

// Groups
const groups = {
    "Session": ["INIT", "AUTH", "RESUME", "LOGOUT", "SESSION_INFO", "SESSION_LIST", "SESSION_KILL", "SESSION_SUSPEND", "SESSION_RESUME_SUSPENDED"],
    "Mailbox": ["MB_LIST", "MB_CREATE", "MB_RENAME", "MB_DELETE", "MB_INFO", "MB_PURGE_TRASH", "MB_SUBSCRIBE", "MB_UNSUBSCRIBE"],
    "Message": ["MSG_SEND", "MSG_SEND_DRAFT", "MSG_LIST", "MSG_GET", "MSG_HEADERS", "MSG_THREAD", "MSG_MOVE", "MSG_COPY", "MSG_DELETE", "MSG_UNDELETE", "MSG_EXPUNGE", "MSG_FLAG_SET", "MSG_FLAG_CLEAR", "MSG_BULK_ACTION"],
    "Search": ["SEARCH_GLOBAL", "SEARCH_SIMPLE", "SEARCH_ADV", "SEARCH_SUGGEST"],
    "Profile Data": ["PROFILE_GET", "PROFILE_SET", "PREF_GET", "PREF_SET"],
    "System / Network": ["HB", "PING", "PONG", "TIME", "VERSION_CHECK", "CAPABILITIES", "CONFIG_PUBLIC_GET", "CONNECTION_INFO", "LIST", "DEBUG_ECHO", "ATTACH_UPLOAD_INIT", "ATTACH_GET"],
};

// Resizer logic
function setupResizer() {
    let isDragging = false;

    resizer.addEventListener('mousedown', (e) => {
        isDragging = true;
        resizer.classList.add('dragging');
        document.body.style.cursor = 'col-resize';
    });

    document.addEventListener('mousemove', (e) => {
        if (!isDragging) return;
        const width = e.clientX;
        if (width >= 200 && width <= 500) {
            sidebar.style.width = width + 'px';
        }
    });

    document.addEventListener('mouseup', () => {
        isDragging = false;
        resizer.classList.remove('dragging');
        document.body.style.cursor = 'default';
    });
}

// Render Sidebar
function renderSidebar() {
    sidebarCmds.innerHTML = '';
    for (const [groupName, cmdList] of Object.entries(groups)) {
        const cat = document.createElement('div');
        cat.className = 'cmd-category';
        cat.innerText = groupName;
        sidebarCmds.appendChild(cat);

        cmdList.forEach(cmd => {
            const item = document.createElement('div');
            item.className = 'cmd-item' + (cmd === currentCmd ? ' active' : '');
            item.innerText = cmd;
            item.onclick = () => selectCommand(cmd);
            sidebarCmds.appendChild(item);
        });
    }
}

function selectCommand(cmd) {
    currentCmd = cmd;
    activeCmdName.innerText = cmd;
    renderSidebar();
    renderForm();
}

// Result Area Handling
function updateResultArea(content, type = 'res') {
    const resultArea = document.getElementById('result-area');
    const resultContent = document.getElementById('result-content');
    if (!resultArea || !resultContent) return;

    resultArea.classList.add('visible');
    resultContent.innerText = typeof content === 'string' ? content : JSON.stringify(content, null, 2);
    resultContent.style.color = type === 'err' ? 'var(--error)' : type === 'req' ? 'var(--accent)' : '#10b981';
}

// Ensure WebTransport handles incoming data
wmtpProtocol.onResponse = (msg) => {
    updateResultArea(msg, 'res');

    // Automatically capture session token state if successful
    if (msg.cmd === 'AUTH_OK' || msg.cmd === 'SESSION_RESUMED' || msg.cmd === 'SESSION_INIT') {
        if (msg.session_token) {
            sessionToken = msg.session_token;
            localStorage.setItem('wmtp_session', sessionToken);
            sessionEmail.innerText = msg.email ? msg.email : "Anonymous Token";
        }
    }

    if (msg.cmd === 'LOGOUT_OK') {
        sessionToken = null;
        localStorage.removeItem('wmtp_session');
        sessionEmail.innerText = "No Session";
    }
};

wmtpProtocol.onError = (msg) => {
    updateResultArea(msg, 'err');
};

// Connection Tracking
let WT_CONNECTED = false;

async function trackConnection() {
    statusDot.className = 'status-dot';
    statusText.innerText = 'Connecting...';
    try {
        await wmtpTransport.connect('https://127.0.0.1:4434/wmtp');
        WT_CONNECTED = true;
        statusDot.classList.add('connected');
        statusText.innerText = 'Secure Transport Connected';

        // Auto resume if available
        const saved = localStorage.getItem('wmtp_session');
        if (saved) {
            await wmtpProtocol.resume(saved);
        }
    } catch (e) {
        WT_CONNECTED = false;
        statusDot.classList.add('error');
        statusText.innerText = 'Offline';
        setTimeout(trackConnection, 5000); // retry
    }
}

// Form Builder
function renderForm() {
    const schema = COMMAND_SCHEMAS[currentCmd];
    if (!schema) {
        formPanel.innerHTML = '<div style="color:var(--error)">Unknown Command</div>';
        return;
    }

    let html = '';
    const fields = Object.entries(schema);

    if (fields.length === 0) {
        html += '<p style="color:var(--text-muted); font-size: 13px;">This command requires no additional parameters.</p>';
    }

    // Session token field
    html += `
        <div class="form-group" style="padding-bottom: 20px; border-bottom: 1px solid var(--border)">
            <label>session_token (Auto-Injected)</label>
            <input type="text" class="form-control" id="f_session_token" placeholder="Leave blank to use current active session" style="opacity: 0.6">
        </div>
    `;

    fields.forEach(([key, conf]) => {
        let inputHtml = '';
        if (conf.type === 'text') {
            inputHtml = `<textarea id="f_${key}" class="form-control" placeholder="${conf.hint || ''}">${conf.default || ''}</textarea>`;
        } else if (conf.type === 'boolean') {
            inputHtml = `<select id="f_${key}" class="form-control"><option value="false">False</option><option value="true">True</option></select>`;
        } else {
            inputHtml = `<input type="text" id="f_${key}" class="form-control" placeholder="${conf.hint || ''}" value="${conf.default || ''}">`;
        }

        html += `
            <div class="form-group">
                <label>${key} ${conf.required ? '<span style="color:var(--error)">*</span>' : ''} <span style="font-weight:normal;color:#666">(${conf.type})</span></label>
                ${inputHtml}
            </div>
        `;
    });

    html += `<button class="btn" onclick="executeCommand()">Send ${currentCmd}</button>`;

    // Add result area
    html += `
        <div id="result-area" class="result-area">
            <div class="result-header">
                <h3>Server Response</h3>
                <button class="btn" style="width: auto; padding: 4px 8px; font-size: 11px;" onclick="document.getElementById('result-area').classList.remove('visible')">Clear</button>
            </div>
            <pre id="result-content" class="result-content"></pre>
        </div>
    `;

    formPanel.innerHTML = html;
}

// Execution
async function executeCommand() {
    if (!WT_CONNECTED) {
        alert('Cannot send: Not connected to transport');
        return;
    }

    const schema = COMMAND_SCHEMAS[currentCmd];
    let data = {};

    const manualToken = document.getElementById('f_session_token').value;
    if (manualToken) {
        data.session_token = manualToken;
    } else if (sessionToken && currentCmd !== 'INIT' && currentCmd !== 'RESUME') {
        data.session_token = sessionToken;
        if (currentCmd === 'AUTH' && !data.session_token) delete data.session_token;
    }

    for (const [key, conf] of Object.entries(schema)) {
        const input = document.getElementById(`f_${key}`);
        if (!input) continue;
        const val = input.value;
        if (val === '' && conf.required) {
            alert(`Missing required field: ${key}`);
            return;
        }
        if (val !== '') {
            if (conf.type === 'array') {
                data[key] = val.split(',').map(s => s.trim());
            } else if (conf.type === 'number') {
                data[key] = Number(val);
            } else if (conf.type === 'boolean') {
                data[key] = val === 'true';
            } else {
                data[key] = val;
            }
        }
    }

    let payload = { cmd: currentCmd };
    if (Object.keys(data).length > 0) {
        payload.data = data;
    }

    // Show request state
    updateResultArea(`Sending ${currentCmd}...\n` + JSON.stringify(payload, null, 2), 'req');

    await wmtpProtocol.send(payload);
}

// Init
window.onload = () => {
    setupResizer();
    renderSidebar();
    renderForm();
    trackConnection();
};
