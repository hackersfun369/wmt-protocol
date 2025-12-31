/**
 * WMTP UI Controller
 * Handles user interface interactions
 */

document.addEventListener('DOMContentLoaded', () => {
    // DOM Elements
    const elements = {
        // Connection
        serverUrl: document.getElementById('serverUrl'),
        connectBtn: document.getElementById('connectBtn'),
        disconnectBtn: document.getElementById('disconnectBtn'),
        connectionStatus: document.getElementById('connectionStatus'),

        // Auth
        emailInput: document.getElementById('emailInput'),
        authBtn: document.getElementById('authBtn'),
        userInfo: document.getElementById('userInfo'),

        // Commands
        initBtn: document.getElementById('initBtn'),
        pingBtn: document.getElementById('pingBtn'),
        statusBtn: document.getElementById('statusBtn'),
        infoBtn: document.getElementById('infoBtn'),
        logoutBtn: document.getElementById('logoutBtn'),

        // Custom
        customCmd: document.getElementById('customCmd'),
        sendBtn: document.getElementById('sendBtn'),

        // Console
        console: document.getElementById('console'),
        clearBtn: document.getElementById('clearBtn'),

        // Navigation
        navItems: document.querySelectorAll('.nav-item'),
        views: document.querySelectorAll('.view'),

        // Attachments
        attachFileInput: document.getElementById('attachFileInput'),
        attachBtn: document.getElementById('attachBtn'),
        attachStatus: document.getElementById('attachStatus'),
    };

    // ========== Console Functions ==========

    function log(message, type = 'info') {
        const line = document.createElement('div');
        line.className = `console-line ${type}`;

        const timestamp = new Date().toLocaleTimeString();

        if (typeof message === 'object') {
            line.innerHTML = `<span class="timestamp">[${timestamp}]</span> <pre>${JSON.stringify(
                message,
                null,
                2
            )}</pre>`;
        } else {
            line.innerHTML = `<span class="timestamp">[${timestamp}]</span> ${message}`;
        }

        elements.console.appendChild(line);
        elements.console.scrollTop = elements.console.scrollHeight;
    }

    function clearConsole() {
        elements.console.innerHTML = '';
        log('Console cleared', 'info');
    }

    // ========== UI State Functions ==========

    function updateConnectionStatus(connected) {
        const dot = elements.connectionStatus.querySelector('.status-dot');
        const text = elements.connectionStatus.querySelector('.status-text');

        if (connected) {
            dot.className = 'status-dot connected';
            text.textContent = 'Connected';
        } else {
            dot.className = 'status-dot disconnected';
            text.textContent = 'Disconnected';
        }
    }

    function updateUserInfo(session) {
        if (session.authenticated && session.email) {
            elements.userInfo.innerHTML = `
                <span class="user-email">${session.email}</span>
                <span class="user-status">✓ Authenticated</span>
            `;
        } else if (session.token) {
            elements.userInfo.innerHTML = `
                <span class="user-status">Session active</span>
            `;
        } else {
            elements.userInfo.innerHTML = `
                <span>Not authenticated</span>
            `;
        }
    }

    function setButtonsEnabled(connected) {
        elements.disconnectBtn.disabled = !connected;
        elements.connectBtn.disabled = connected;
        elements.authBtn.disabled = !connected;
        elements.initBtn.disabled = !connected;
        elements.pingBtn.disabled = !connected;
        elements.statusBtn.disabled = !connected;
        elements.infoBtn.disabled = !connected;
        elements.logoutBtn.disabled = !connected;
        elements.sendBtn.disabled = !connected;

        if (elements.attachBtn) {
            // Require both connection and authenticated session
            elements.attachBtn.disabled = !connected || !wmtpProtocol.isAuthenticated();
        }
    }

    // ========== Transport Event Handlers ==========

    wmtpTransport.onConnect = () => {
        log('✅ Connected to server', 'success');
        updateConnectionStatus(true);
        setButtonsEnabled(true);
    };

    wmtpTransport.onDisconnect = () => {
        log('❌ Disconnected from server', 'error');
        updateConnectionStatus(false);
        setButtonsEnabled(false);
        updateUserInfo({});
    };

    wmtpTransport.onError = (error) => {
        log(`⚠️ Error: ${error.message}`, 'error');
    };

    wmtpTransport.onMessage = (message) => {
        wmtpProtocol.handleMessage(message);
    };

    // ========== Protocol Event Handlers ==========

    wmtpProtocol.onSessionInit = (msg) => {
        log('📋 Session initialized', 'success');
        log(msg, 'response');
        updateUserInfo(wmtpProtocol.getSession());
        // session exists but not authenticated yet
        setButtonsEnabled(wmtpTransport.isConnected());
    };

    wmtpProtocol.onAuthSuccess = (msg) => {
        log('🔐 Authentication successful', 'success');
        log(msg, 'response');
        updateUserInfo(wmtpProtocol.getSession());
        wmtpProtocol.saveSession();
        // now authenticated → re-enable buttons including attach
        setButtonsEnabled(wmtpTransport.isConnected());
    };

    wmtpProtocol.onHeartbeat = (_msg) => {
        // Silent heartbeat
    };

    wmtpProtocol.onResponse = (msg) => {
        if (msg.cmd !== 'HB') {
            log(msg, msg.status === 'OK' ? 'response' : 'error');
        }

        if (msg.cmd === 'ATTACH_GET' && msg.status === 'OK') {
            const d = msg.data;
            openBase64File(d.content_b64, d.mime_type, d.filename);
        }
    };


    wmtpProtocol.onError = (msg) => {
        log(`❌ Error: ${msg.msg}`, 'error');
    };

    // ========== Button Event Handlers ==========

    elements.connectBtn.addEventListener('click', async () => {
        const url = elements.serverUrl.value.trim();
        if (!url) {
            log('Please enter a server URL', 'error');
            return;
        }

        log(`Connecting to ${url}...`, 'info');

        try {
            if (url.includes('localhost') || url.includes('127.0.0.1')) {
                wmtpTransport.setCertificateHash('9kDUV0kAxsCBObFdiULtY3w5b0xcp8l6A+uF7Ds9yFc=');
                log('Using self-signed certificate hash', 'info');
            }

            await wmtpTransport.connect(url);
        } catch (error) {
            log(`Connection failed: ${error.message}`, 'error');
        }
    });

    elements.disconnectBtn.addEventListener('click', async () => {
        log('Disconnecting...', 'info');
        await wmtpTransport.disconnect();
    });

    elements.authBtn.addEventListener('click', async () => {
        const email = elements.emailInput.value.trim();
        if (!email) {
            log('Please enter an email', 'error');
            return;
        }

        log(`Authenticating as ${email}...`, 'info');
        await wmtpProtocol.auth(email);
    });

    elements.initBtn.addEventListener('click', async () => {
        log('Initializing session...', 'info');
        await wmtpProtocol.init();
    });

    // PING with latency measurement
    elements.pingBtn.addEventListener('click', async () => {
        if (!wmtpTransport.isConnected()) {
            log('Not connected', 'error');
            return;
        }

        log('Sending PING...', 'info');

        try {
            const { response, responseTimeMs } = await wmtpTransport.sendWithTiming({
                cmd: 'PING',
                data: {},
            });

            log(`PING latency: ${responseTimeMs.toFixed(2)} ms`, 'info');
            log(response, response.status === 'OK' ? 'response' : 'error');
        } catch (e) {
            log(`PING failed: ${e.message}`, 'error');
        }
    });

    elements.statusBtn.addEventListener('click', async () => {
        log('Requesting status...', 'info');
        await wmtpProtocol.status();
    });

    elements.infoBtn.addEventListener('click', async () => {
        log('Requesting info...', 'info');
        await wmtpProtocol.info();
    });

    elements.logoutBtn.addEventListener('click', async () => {
        log('Logging out...', 'info');
        await wmtpProtocol.logout();
        wmtpProtocol.clearSession();
        updateUserInfo({});
        setButtonsEnabled(wmtpTransport.isConnected());
    });

    elements.sendBtn.addEventListener('click', async () => {
        const cmdText = elements.customCmd.value.trim();
        if (!cmdText) {
            log('Please enter a command', 'error');
            return;
        }

        try {
            const cmd = JSON.parse(cmdText);
            log(`Sending: ${cmdText}`, 'info');

            if (!wmtpTransport.isConnected()) {
                log('Not connected', 'error');
                return;
            }

            const { response, responseTimeMs } = await wmtpTransport.sendWithTiming(cmd);

            const latencyEl = document.getElementById('customLatency');
            if (latencyEl) {
                latencyEl.textContent = `${responseTimeMs.toFixed(2)} ms`;
            }

            log(response, response.status === 'OK' ? 'response' : 'error');
        } catch (e) {
            log('Invalid JSON', 'error');
        }
    });

    elements.clearBtn.addEventListener('click', clearConsole);

    // Enter key handlers
    elements.customCmd.addEventListener('keypress', (e) => {
        if (e.key === 'Enter') {
            elements.sendBtn.click();
        }
    });

    elements.emailInput.addEventListener('keypress', (e) => {
        if (e.key === 'Enter') {
            elements.authBtn.click();
        }
    });

    elements.serverUrl.addEventListener('keypress', (e) => {
        if (e.key === 'Enter') {
            elements.connectBtn.click();
        }
    });

    // ========== Navigation ==========

    elements.navItems.forEach((item) => {
        item.addEventListener('click', (e) => {
            e.preventDefault();

            const viewId = item.dataset.view + 'View';

            elements.navItems.forEach((nav) => nav.classList.remove('active'));
            item.classList.add('active');

            elements.views.forEach((view) => {
                view.classList.remove('active');
                if (view.id === viewId) {
                    view.classList.add('active');
                }
            });
        });
    });

    // ========== Attachments ==========

    if (elements.attachBtn && elements.attachFileInput) {
        elements.attachBtn.addEventListener('click', async () => {
            if (!wmtpTransport.isConnected() || !wmtpProtocol.isAuthenticated()) {
                log('You must be connected and authenticated to upload attachments', 'error');
                return;
            }

            const files = elements.attachFileInput.files;
            if (!files || files.length === 0) {
                log('Please choose one or more files first', 'error');
                return;
            }

            for (const file of files) {
                try {
                    // 1) INIT on control stream
                    log(`Initializing attachment upload: ${file.name} (${file.size} bytes)`, 'info');
                    if (elements.attachStatus) {
                        elements.attachStatus.textContent = `Initializing ${file.name}...`;
                    }

                    const { response } = await wmtpTransport.sendWithTiming({
                        cmd: 'ATTACH_UPLOAD_INIT',
                        data: {
                            session_token: wmtpProtocol.sessionToken,
                            filename: file.name,
                            mime_type: file.type || 'application/octet-stream',
                            size_bytes: file.size,
                        },
                    });

                    if (response.status !== 'OK') {
                        log(response, 'error');
                        if (elements.attachStatus) {
                            elements.attachStatus.textContent = `Init failed for ${file.name}`;
                        }
                        continue;
                    }

                    const uploadId = response.data?.upload?.upload_id;
                    if (!uploadId) {
                        log('No upload_id in ATTACH_UPLOAD_INIT response', 'error');
                        continue;
                    }

                    log(`upload_id for ${file.name}: ${uploadId}`, 'info');

                    // >>> ADD THIS LINE TO PREVIEW LOCALLY <<<
                    previewLocalFile(file);

                    // 2) open attachment stream
                    const { send, recv } = await wmtpTransport.openAttachmentStream();

                    // 3) send header (JSON + \n) on attachment stream
                    const header = {
                        upload_id: uploadId,
                        filename: file.name,
                        mime_type: file.type || 'application/octet-stream',
                        size_bytes: file.size,
                    };
                    const headerStr = JSON.stringify(header) + '\n';
                    await send.write(new TextEncoder().encode(headerStr));

                    // 4) stream raw bytes
                    const reader = file.stream().getReader();
                    while (true) {
                        const { done, value } = await reader.read();
                        if (done) break;
                        await send.write(value);
                    }

                    await send.close();
                    log(`Attachment bytes sent for ${file.name}`, 'info');
                    if (elements.attachStatus) {
                        elements.attachStatus.textContent = `Uploaded ${file.name}`;
                    }
                } catch (err) {
                    log(`Attachment upload failed for ${file.name}: ${err.message}`, 'error');
                    if (elements.attachStatus) {
                        elements.attachStatus.textContent = `Error uploading ${file.name}`;
                    }
                }
            }
        });
    }


    // display attachment on frontend

    function openBase64File(contentB64, mimeType, filename) {
        if (!contentB64) {
            alert('Attachment has no content.');
            return;
        }

        const byteChars = atob(contentB64);
        const byteNumbers = new Array(byteChars.length);
        for (let i = 0; i < byteChars.length; i++) {
            byteNumbers[i] = byteChars.charCodeAt(i);
        }
        const byteArray = new Uint8Array(byteNumbers);
        const blob = new Blob([byteArray], { type: mimeType || 'application/octet-stream' });
        const url = URL.createObjectURL(blob);
        window.open(url, '_blank');
    }



    function previewLocalFile(file) {
        const url = URL.createObjectURL(file);
        window.open(url, '_blank'); // opens PDF/image/text in new tab
    }


    // ========== Initialize ==========

    log('🚀 WMTP Client Ready', 'info');
    log('Connect to a server to begin', 'info');

    const savedSession = wmtpProtocol.loadSession();
    if (savedSession) {
        log(`Found saved session for ${savedSession.email}`, 'info');
    }
});
