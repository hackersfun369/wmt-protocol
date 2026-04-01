let currentCmd = 'INIT';
let sessionToken = null;
let selectedUploadFile = null; // Store file for upload

// DOM Elements
const sidebar = document.getElementById('sidebar');
const sidebarCmds = document.getElementById('sidebar-cmds');
const resizer = document.getElementById('resizer');
const formPanel = document.getElementById('form-panel');
const activeCmdName = document.getElementById('active-cmd-name');
const statusDot = document.getElementById('status-dot');
const statusText = document.getElementById('status-text');
const sessionEmail = document.getElementById('session-email');

// Groups based on API reference
const groups = {
    "Session": ["INIT", "AUTH", "RESUME", "LOGOUT", "SESSION_INFO", "SESSION_LIST", "SESSION_KILL", "SESSION_SUSPEND", "SESSION_RESUME_SUSPENDED"],
    "Health & Admin": ["PING", "PONG", "HB", "UPTIME", "LATENCY_PING", "CONNECTION_INFO", "RATE_LIMIT_INFO", "DEBUG_ECHO", "INFO", "STATUS", "CAPABILITIES", "TIME", "VERSION_CHECK", "CONFIG_PUBLIC_GET", "CONNECTION_LIST"],
    "Mailbox": ["MB_LIST", "MAIL_LIST", "MB_CREATE", "MB_RENAME", "MB_DELETE", "MB_INFO", "MB_PURGE_TRASH", "MB_SUBSCRIBE", "MB_UNSUBSCRIBE"],
    "Messages": ["MSG_SEND", "MSG_SEND_DRAFT", "MSG_LIST", "MSG_GET", "MSG_HEADERS", "MSG_THREAD", "MSG_MOVE", "MSG_COPY", "MSG_DELETE", "MSG_EXPUNGE", "MSG_UNDELETE", "MSG_FLAG_SET", "MSG_FLAG_CLEAR", "MSG_BULK_ACTION"],
    "Search": ["SEARCH", "SEARCH_GLOBAL", "SEARCH_ADV", "SEARCH_SUGGEST"],
    "User Profile": ["PROFILE_GET", "PROFILE_SET", "PREF_GET", "PREF_SET"],
    "Attachments": ["ATTACH_UPLOAD_INIT", "ATTACH_GET"]
};

// Resizer logic
function setupResizer() {
    let isDragging = false;
    resizer.addEventListener('mousedown', () => {
        isDragging = true;
        resizer.classList.add('dragging');
        document.body.style.cursor = 'col-resize';
    });
    document.addEventListener('mousemove', (e) => {
        if (!isDragging) return;
        const width = e.clientX;
        if (width >= 220 && width <= 450) {
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

// Syntax highlighting for JSON
function syntaxHighlight(json) {
    if (typeof json != 'string') {
        json = JSON.stringify(json, undefined, 2);
    }
    json = json.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
    return json.replace(/("(\\u[a-zA-Z0-9]{4}|\\[^u]|[^\\"])*"(\s*:)?|\b(true|false|null)\b|-?\d+(?:\.\d*)?(?:[eE][+\-]?\d+)?)/g, function (match) {
        let cls = 'token-number';
        if (/^"/.test(match)) {
            if (/:$/.test(match)) {
                cls = 'token-key';
            } else {
                cls = 'token-string';
            }
        } else if (/true|false/.test(match)) {
            cls = 'token-boolean';
        } else if (/null/.test(match)) {
            cls = 'token-null';
        }
        return '<span class="' + cls + '">' + match + '</span>';
    });
}

// Result Area Handling
function updateResultArea(content, type = 'res') {
    const resultArea = document.getElementById('result-area');
    const resultContent = document.getElementById('result-content');

    resultArea.classList.add('visible');

    let displayStr = '';

    // For outgoing request visually formatted strings
    if (typeof content === 'string' && content.startsWith('Sending')) {
        displayStr = `<span style="color:var(--primary)">${content}</span>`;
    } else {
        // Truncate giant base64 strings before stringifying to prevent browser lockup for the log view
        let displayContent = content;
        if (typeof content === 'object' && content !== null && content.cmd === 'ATTACH_GET' && content.data && content.data.content_b64) {
            displayContent = JSON.parse(JSON.stringify(content)); // deep clone
            if (displayContent.data.content_b64.length > 100) {
                displayContent.data.content_b64 = displayContent.data.content_b64.substring(0, 100) + '... (truncated, ' + displayContent.data.content_b64.length + ' bytes total)';
            }
        }

        // Format object correctly with syntax highlighting
        let jsonStr = typeof displayContent === 'string' ? displayContent : JSON.stringify(displayContent, null, 2);
        displayStr = syntaxHighlight(jsonStr);
    }

    // Append instead of overwrite if it's a response and we already have a request showing
    if (type === 'res' || type === 'err') {
        const prefix = type === 'res' ? '\n\n<span style="color:var(--success)">// --- Server Response --- //</span>\n' : '\n\n<span style="color:var(--error)">// --- Error Response --- //</span>\n';
        resultContent.innerHTML += prefix + displayStr;

        // --- Special Preview Rendering for ATTACH_GET ---
        if (type === 'res' && typeof content === 'object' && content.cmd === 'ATTACH_GET' && content.data) {
            const attach = content.data;
            if (!attach.content_b64) {
                // Binary stream trigger
                handleAttachmentDownload(attach);
            }
        }

        // auto-scroll to bottom //
        const viewport = document.querySelector('.log-viewport');
        if (viewport) viewport.scrollTop = viewport.scrollHeight;
    } else {
        // This handles "req" or append-style custom strings
        if (typeof content === 'string' && content.includes('<span')) {
            resultContent.innerHTML += content;
            const viewport = document.querySelector('.log-viewport');
            if (viewport) viewport.scrollTop = viewport.scrollHeight;
        } else {
            resultContent.innerHTML = displayStr; // Reset for new request
        }
    }
}

// Ensure WebTransport handles incoming data
wmtpProtocol.onResponse = (msg) => {
    // Determine type by status error vs ok
    const type = msg.status === 'ERR' ? 'err' : 'res';
    updateResultArea(msg, type);

    // Automatically capture session token state if successful
    if (msg.cmd === 'AUTH_OK' || msg.cmd === 'SESSION_RESUMED' || msg.cmd === 'SESSION_INIT' || msg.session_token) {
        if (msg.session_token) {
            sessionToken = msg.session_token;
            localStorage.setItem('wmtp_session', sessionToken);
            sessionEmail.innerText = msg.email ? msg.email : "Authenticated";
        }
    }

    if (msg.cmd === 'LOGOUT_OK') {
        sessionToken = null;
        localStorage.removeItem('wmtp_session');
        sessionEmail.innerText = "No Session";
    }

    // Auto-trigger upload if we get ATTACH_UPLOAD_INIT response and have a file pending
    if (msg.cmd === 'ATTACH_UPLOAD_INIT' && msg.data && msg.data.upload && selectedUploadFile) {
        handleAttachmentUpload(msg.data.upload, selectedUploadFile);
        selectedUploadFile = null;
    }
};

async function handleAttachmentUpload(uploadInfo, file) {
    if (!WT_CONNECTED) return;
    updateResultArea(`\n<span style="color:var(--primary)">// --- Starting binary stream upload for ${file.name} --- //</span>\n`, 'req');

    try {
        const streamInfo = await wmtpTransport.openAttachmentStream();
        const writer = streamInfo.send;

        // Prepare metadata JSON
        const metaObj = {
            upload_id: uploadInfo.upload_id,
            filename: file.name,
            mime_type: file.type || 'application/octet-stream',
            size_bytes: file.size
        };
        const metaString = JSON.stringify(metaObj);
        const encoder = new TextEncoder();
        const metaBytes = encoder.encode(metaString);

        // Prepare header: 12 bytes
        const headerBuf = new ArrayBuffer(12);
        const view = new DataView(headerBuf);
        view.setBigUint64(0, BigInt(file.size), true); // true = little endian
        view.setUint32(8, metaBytes.byteLength, true); // true = little endian

        // Write header & metadata
        await writer.write(new Uint8Array(headerBuf));
        await writer.write(metaBytes);

        // Stream file chunks
        const reader = file.stream().getReader();
        let uploaded = 0;
        while (true) {
            const { done, value } = await reader.read();
            if (done) break;
            await writer.write(value);
            uploaded += value.byteLength;
        }

        await writer.close();
        updateResultArea(`\n<span style="color:var(--success)">// --- Binary upload completed successfully (${uploaded} bytes) --- //</span>\n`, 'res');

    } catch (e) {
        updateResultArea(`\n<span style="color:var(--error)">// --- Upload failed: ${e.message} --- //</span>\n`, 'err');
    }
}

async function handleAttachmentDownload(attach) {
    if (!WT_CONNECTED) return;
    updateResultArea(`\n<span style="color:var(--primary)">// --- Starting binary stream download for ${attach.filename} --- //</span>\n`, 'req');

    try {
        const streamInfo = await wmtpTransport.openAttachmentStream();
        const writer = streamInfo.send;
        const reader = streamInfo.recv;

        // Prepare metadata JSON header
        const metaObj = {
            action: 'download',
            upload_id: attach.attachment_id,
        };
        const metaString = JSON.stringify(metaObj);
        const encoder = new TextEncoder();
        const metaBytes = encoder.encode(metaString);

        // Prepare header: 12 bytes. FileSize doesn't matter for download init, but we must send the 12 byte format
        const headerBuf = new ArrayBuffer(12);
        const view = new DataView(headerBuf);
        view.setBigUint64(0, BigInt(0), true);
        view.setUint32(8, metaBytes.byteLength, true);

        // Write header & metadata
        await writer.write(new Uint8Array(headerBuf));
        await writer.write(metaBytes);
        await writer.close(); // Important: signal we are done writing so server can start piping

        // Stream file chunks back
        let downloaded = 0;
        const chunks = [];

        while (true) {
            const { done, value } = await reader.read();
            if (done) break;
            chunks.push(value);
            downloaded += value.byteLength;
        }

        // Combine chunks into a single Blob
        const blob = new Blob(chunks, { type: attach.mime_type || 'application/octet-stream' });
        const dataUrl = URL.createObjectURL(blob);

        updateResultArea(`\n<span style="color:var(--success)">// --- Binary download completed successfully (${downloaded} bytes) --- //</span>\n`, 'res');

        const resultContent = document.getElementById('result-content');
        let previewHtml = `<div style="margin-top:20px; padding-top:20px; border-top:1px solid var(--border)">
                <h4 style="color:var(--text); margin-bottom:10px;">Attachment Preview: ${attach.filename}</h4>`;

        const mime = blob.type;
        if (mime.startsWith('image/')) {
            previewHtml += `<div style="text-align:center; padding: 10px; background: var(--surface); border-radius: 4px;"><img src="${dataUrl}" style="max-width:100%; border-radius:4px; max-height:400px; object-fit:contain;"></div>`;
        } else if (mime.startsWith('text/')) {
            const textContent = await blob.text();
            previewHtml += `<pre style="background:var(--surface); padding:10px; border-radius:4px; max-height:300px; overflow:auto; white-space:pre-wrap; word-wrap:break-word;">${textContent.replace(/</g, '&lt;')}</pre>`;
        } else {
            previewHtml += `<a href="${dataUrl}" download="${attach.filename}" class="btn btn-primary" style="display:inline-block; margin-top:10px;">Download ${attach.filename}</a>`;
        }
        previewHtml += `</div>`;
        resultContent.innerHTML += previewHtml;

        const viewport = document.querySelector('.log-viewport');
        if (viewport) viewport.scrollTop = viewport.scrollHeight;

    } catch (e) {
        updateResultArea(`\n<span style="color:var(--error)">// --- Download failed: ${e.message} --- //</span>\n`, 'err');
    }
}

wmtpProtocol.onError = (msg) => {
    updateResultArea(msg, 'err');
};

// Connection Tracking
let WT_CONNECTED = false;

async function trackConnection() {
    statusDot.className = 'dot';
    statusText.innerText = 'Connecting...';
    try {
        await wmtpTransport.connect('https://127.0.0.1:4434/wmtp');
        WT_CONNECTED = true;
        statusDot.classList.add('connected');
        statusText.innerText = 'Secure Transport Connected';

        const saved = localStorage.getItem('wmtp_session');
        if (saved) {
            // Auto-resume if token exists
            await wmtpProtocol.send({ cmd: 'RESUME', data: { token: saved } });
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
    selectedUploadFile = null; // Reset on form switch
    // Note: COMMAND_SCHEMAS comes from schema.js (extracted exactly from docs)
    const schema = COMMAND_SCHEMAS[currentCmd] || {};

    let html = '';

    // Special custom UI for file upload
    if (currentCmd === 'ATTACH_UPLOAD_INIT') {
        html += `
            <div class="form-group">
                <label class="form-label">Select File to Upload <span class="req-star">*</span></label>
                <input type="file" id="f_upload_picker" class="form-control" style="padding: 8px;">
            </div>
        `;
    }
    const fields = Object.entries(schema);

    if (fields.length === 0) {
        html += '<p style="color:var(--text-muted); font-size: 13px; margin-bottom: 24px;">This command requires no parameters.</p>';
    }

    fields.forEach(([key, conf]) => {
        // Special case: we auto-inject session_token, but let user override
        if (key === 'session_token') {
            html += `
                <div class="form-group">
                    <label class="form-label">${key} <span class="type-hint">(Auto-Injected)</span></label>
                    <input type="text" class="form-control" id="f_${key}" placeholder="Leave blank to use active session [${sessionToken ? sessionToken.substring(0, 8) + '...' : 'None'}]">
                </div>
            `;
            return;
        }

        const isReq = conf.required;
        const typeRaw = conf.type ? conf.type.toLowerCase() : 'string';
        const hint = conf.hint || '';

        let inputHtml = '';
        if (typeRaw.includes('bool')) {
            inputHtml = `<select id="f_${key}" class="form-control">
                <option value="">-- Match Type --</option>
                <option value="true">True</option>
                <option value="false">False</option>
            </select>`;
        } else if (typeRaw.includes('array')) {
            inputHtml = `<textarea id="f_${key}" class="form-control" style="min-height: 60px;" placeholder="Comma separated values... (${hint})"></textarea>`;
        } else if (typeRaw === 'object' || typeRaw === 'any') {
            inputHtml = `<textarea id="f_${key}" class="form-control" style="min-height: 80px;" placeholder="Valid JSON... (${hint})"></textarea>`;
        } else if (key === 'body' || key === 'signature') {
            inputHtml = `<textarea id="f_${key}" class="form-control" style="min-height: 120px;" placeholder="${hint}"></textarea>`;
        } else {
            inputHtml = `<input type="text" id="f_${key}" class="form-control" placeholder="${hint}">`;
        }

        html += `
            <div class="form-group">
                <label class="form-label">${key} ${isReq ? '<span class="req-star">*</span>' : ''} <span class="type-hint">(${typeRaw})</span></label>
                ${inputHtml}
            </div>
        `;
    });

    html += `<button class="btn btn-primary" onclick="executeCommand()">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M22 2L11 13M22 2l-7 20-4-9-9-4 20-7z"/></svg>
        Send ${currentCmd}
    </button>`;

    formPanel.innerHTML = html;

    // Attach file picker listener if applicable
    if (currentCmd === 'ATTACH_UPLOAD_INIT') {
        const picker = document.getElementById('f_upload_picker');
        if (picker) {
            picker.addEventListener('change', (e) => {
                const file = e.target.files[0];
                if (file) {
                    selectedUploadFile = file;
                    document.getElementById('f_filename').value = file.name;
                    document.getElementById('f_mime_type').value = file.type || 'application/octet-stream';
                    document.getElementById('f_size_bytes').value = file.size;
                }
            });
        }
        // Make auto-filled fields readonly
        setTimeout(() => {
            const f1 = document.getElementById('f_filename');
            const f2 = document.getElementById('f_mime_type');
            const f3 = document.getElementById('f_size_bytes');
            if (f1) f1.readOnly = true;
            if (f2) f2.readOnly = true;
            if (f3) f3.readOnly = true;
        }, 50);
    }
}

// Execution
async function executeCommand() {
    if (!WT_CONNECTED) {
        alert('Cannot send: Not connected to transport');
        return;
    }

    const schema = COMMAND_SCHEMAS[currentCmd] || {};
    let data = {};

    for (const [key, conf] of Object.entries(schema)) {
        const input = document.getElementById(`f_${key}`);
        if (!input) continue;

        const val = input.value.trim();

        if (key === 'session_token' || key === 'token') {
            // For RESUME, `token` is the permanent token — don't auto-inject session token
            if (key === 'token' && currentCmd === 'RESUME') {
                // fall through to normal input handling
            } else {
                if (val !== '') {
                    data[key] = val;
                } else if (sessionToken && currentCmd !== 'INIT' && currentCmd !== 'RESUME') {
                    // Auto inject active session token
                    data[key] = sessionToken;
                }
                continue;
            }
        }

        if (val === '' && conf.required) {
            alert(`Missing required field: ${key}`);
            return;
        }

        if (val !== '') {
            const typeRaw = conf.type ? conf.type.toLowerCase() : 'string';

            if (typeRaw.includes('array')) {
                data[key] = val.split(',').map(s => s.trim()).filter(s => s.length > 0);
            } else if (typeRaw.includes('num') || typeRaw.includes('int')) {
                data[key] = Number(val);
            } else if (typeRaw.includes('bool')) {
                data[key] = val === 'true';
            } else if (typeRaw === 'object' || typeRaw === 'any') {
                try {
                    data[key] = JSON.parse(val);
                } catch (e) {
                    alert(`Invalid JSON for field ${key}`);
                    return;
                }
            } else {
                data[key] = val;
            }
        }
    }

    let payload = { cmd: currentCmd };
    if (Object.keys(data).length > 0) {
        payload.data = data;
    }

    // Capture visual representation
    const visualPayload = JSON.stringify(payload, null, 2);
    updateResultArea(`Sending ${currentCmd}...\n\n${visualPayload}`, 'req');

    await wmtpProtocol.send(payload);
}

// Init
window.onload = () => {
    setupResizer();
    renderSidebar();

    // Check if we have session in memory at init
    const saved = localStorage.getItem('wmtp_session');
    if (saved) {
        sessionToken = saved;
        sessionEmail.innerText = "Restoring...";
    }

    selectCommand('INIT'); // Force render initial state
    trackConnection();
};
