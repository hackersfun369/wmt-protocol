
const CERT_HASH = "gnTZFqw83Zw0DaWn8LmkosVoM9pVD4+uF7Ds9yFc=";
const SERVER_URL = "https://localhost:4433";
const USER_EMAIL = "test@auto.com";

const TEST_SEQUENCE = [
    // --- SESSION ---
    { cmd: "INIT", data: {} },
    { cmd: "AUTH", data: { email: USER_EMAIL } },
    { cmd: "RESUME", data: { token: "$TOKEN" } },
    { cmd: "SESSION_INFO", data: { session_token: "$TOKEN" } },
    { cmd: "SESSION_LIST", data: { session_token: "$TOKEN" } },
    { cmd: "SESSION_KILL", data: { session_token: "$TOKEN", target_token: "invalid_token_test" } },
    { cmd: "SESSION_SUSPEND", data: { session_token: "$TOKEN" } },
    { cmd: "SESSION_RESUME_SUSPENDED", data: { session_token: "$TOKEN", token: "$TOKEN_SUSPEND_TEST" } },

    // --- HEALTH ---
    { cmd: "PING", data: {} },
    { cmd: "PONG", data: { session_token: "$TOKEN" } },
    { cmd: "HB", data: { session_token: "$TOKEN" } },
    { cmd: "UPTIME", data: {} },
    { cmd: "LATENCY_PING", data: {} },
    { cmd: "CONNECTION_INFO", data: {} },
    { cmd: "RATE_LIMIT_INFO", data: { session_token: "$TOKEN" } },
    { cmd: "DEBUG_ECHO", data: { session_token: "$TOKEN", foo: "bar" } },

    // --- INFO & ADMIN ---
    { cmd: "INFO", data: {} },
    { cmd: "STATUS", data: {} },
    { cmd: "CAPABILITIES", data: {} },
    { cmd: "TIME", data: {} },
    { cmd: "VERSION_CHECK", data: { client_version: "1.0.0" } },
    { cmd: "CONFIG_PUBLIC_GET", data: {} },
    { cmd: "CONNECTION_LIST", data: { session_token: "$TOKEN" } },

    // --- MAILBOX ---
    { cmd: "MB_LIST", data: { session_token: "$TOKEN" } },
    { cmd: "MAIL_LIST", data: { session_token: "$TOKEN", folder_code: "INBOX", limit: 5 } },
    { cmd: "MB_CREATE", data: { session_token: "$TOKEN", name: "AutoTestBox", code: "AUTOTESTBOX" } },
    { cmd: "MB_RENAME", data: { session_token: "$TOKEN", folder: "AUTOTESTBOX", new_name: "RenamedBox" } },
    { cmd: "MB_DELETE", data: { session_token: "$TOKEN", folder: "AUTOTESTBOX" } },
    { cmd: "MB_INFO", data: { session_token: "$TOKEN", folder_code: "INBOX" } },
    { cmd: "MB_PURGE_TRASH", data: { session_token: "$TOKEN" } },
    { cmd: "MB_SUBSCRIBE", data: { session_token: "$TOKEN", folder: "INBOX" } },
    { cmd: "MB_UNSUBSCRIBE", data: { session_token: "$TOKEN", folder: "INBOX" } },

    // --- ATTACHMENTS (PART 1) ---
    {
        cmd: "ATTACH_UPLOAD_INIT",
        data: {
            session_token: "$TOKEN",
            filename: "WMTP_Spec.pdf",
            mime_type: "application/pdf",
            size_bytes: 395069
        }
    },

    // --- MESSAGES (SEND) ---
    {
        cmd: "MSG_SEND",
        data: {
            session_token: "$TOKEN",
            to: [USER_EMAIL],
            subject: "Attachment Test",
            body: "Sent via automated tester with PDF attachment",
            attachment_ids: ["$UPLOAD_ID"]
        }
    },
    {
        cmd: "MSG_SEND_DRAFT",
        data: {
            session_token: "$TOKEN",
            to: ["draft@test.com"],
            subject: "Draft Test",
            body: "This is a draft"
        }
    },

    // --- MESSAGES (OPERATIONS) ---
    { cmd: "MSG_LIST", data: { session_token: "$TOKEN", folder_code: "SENT", limit: 5 } },
    { cmd: "MSG_GET", data: { session_token: "$TOKEN", id: "$MSG_ID" } },
    { cmd: "MSG_HEADERS", data: { session_token: "$TOKEN", id: "$MSG_ID" } },
    { cmd: "MSG_THREAD", data: { session_token: "$TOKEN", msg_id: "$MSG_ID" } },
    { cmd: "MSG_FLAG_SET", data: { session_token: "$TOKEN", id: "$MSG_ID", read: true, starred: true } },
    { cmd: "MSG_FLAG_CLEAR", data: { session_token: "$TOKEN", id: "$MSG_ID", starred: true } },
    { cmd: "MSG_COPY", data: { session_token: "$TOKEN", id: "$MSG_ID", target_folder: "DRAFTS" } },
    { cmd: "MSG_MOVE", data: { session_token: "$TOKEN", id: "$MSG_ID", target_folder: "BIN" } },
    { cmd: "MSG_UNDELETE", data: { session_token: "$TOKEN", id: "$MSG_ID", target_folder: "INBOX" } },
    { cmd: "MSG_DELETE", data: { session_token: "$TOKEN", id: "$MSG_ID" } },
    { cmd: "MSG_EXPUNGE", data: { session_token: "$TOKEN", id: "$MSG_ID" } },
    { cmd: "MSG_BULK_ACTION", data: { session_token: "$TOKEN", ids: ["$MSG_ID"], action: "DELETE" } },

    // --- SEARCH ---
    { cmd: "SEARCH_SUGGEST", data: { session_token: "$TOKEN", query: "Attach" } },
    { cmd: "SEARCH", data: { session_token: "$TOKEN", q: "Attachment", folder_code: "SENT" } },
    { cmd: "SEARCH_GLOBAL", data: { session_token: "$TOKEN", q: "test" } },
    { cmd: "SEARCH_ADV", data: { session_token: "$TOKEN", q: "Attachment", folder_code: "SENT" } },

    // --- PROFILE ---
    { cmd: "PROFILE_GET", data: { session_token: "$TOKEN" } },
    { cmd: "PROFILE_SET", data: { session_token: "$TOKEN", name: "Auto Tester", signature: "Automated via WebTransport" } },
    { cmd: "PREF_GET", data: { session_token: "$TOKEN" } },
    { cmd: "PREF_SET", data: { session_token: "$TOKEN", theme: "dark" } },

    // --- ATTACHMENTS (PART 2) ---
    { cmd: "ATTACH_GET", data: { session_token: "$TOKEN", attachment_id: "$ATTACH_ID" } },

    // --- CLEANUP ---
    { cmd: "LOGOUT", data: { session_token: "$TOKEN" } },
];

let results = [];
let sessionToken = null;
let lastMsgId = null;
let lastUploadId = null;
let lastAttachId = null;

async function runTests() {
    const runBtn = document.getElementById('runBtn');
    const status = document.getElementById('status');
    const progress = document.getElementById('progress');
    const resultsArea = document.getElementById('results');

    runBtn.disabled = true;
    status.textContent = "Running...";

    try {
        wmtpTransport.setCertificateHash(CERT_HASH);
        await wmtpTransport.connect(SERVER_URL);

        for (let i = 0; i < TEST_SEQUENCE.length; i++) {
            let step = TEST_SEQUENCE[i];
            let cmdName = step.cmd;
            let payload = JSON.parse(JSON.stringify(step.data));

            progress.textContent = `Executing ${i + 1}/${TEST_SEQUENCE.length}: ${cmdName}`;

            // Dynamic Replacements
            if (payload.session_token === "$TOKEN") payload.session_token = sessionToken;
            if (payload.token === "$TOKEN") payload.token = sessionToken;
            if (payload.id === "$MSG_ID") payload.id = lastMsgId;
            if (payload.msg_id === "$MSG_ID") payload.msg_id = lastMsgId;
            if (payload.ids && payload.ids[0] === "$MSG_ID") payload.ids = [lastMsgId];
            if (payload.attachment_ids && payload.attachment_ids[0] === "$UPLOAD_ID") payload.attachment_ids = [lastUploadId];
            if (payload.attachment_id === "$ATTACH_ID") payload.attachment_id = lastAttachId;

            // Handle special session for suspend test
            if (payload.token === "$TOKEN_SUSPEND_TEST") {
                // Create a temporary session just for this test
                console.log("Creating temporary session for suspend test...");
                const tempTransport = new WMTPTransport();
                tempTransport.setCertificateHash(CERT_HASH);
                await tempTransport.connect(SERVER_URL);
                const initRes = await tempTransport.sendWithTiming({ cmd: "INIT" });
                payload.token = initRes.response.session_token;
                await tempTransport.disconnect();
            }

            let responseObj = null;
            let responseTimeMs = 0;

            console.log(`[AutoTester] Testing ${cmdName}`, payload);

            // Execute Command
            const result = await wmtpTransport.sendWithTiming({ cmd: cmdName, data: payload });
            responseObj = result.response;
            responseTimeMs = result.responseTimeMs;

            results.push({
                command: cmdName,
                request: { cmd: cmdName, data: payload },
                response: responseObj,
                latency_ms: responseTimeMs
            });

            // Post-Processing
            if (cmdName === "INIT" && responseObj.session_token) sessionToken = responseObj.session_token;
            if (cmdName === "AUTH" && responseObj.session_token) sessionToken = responseObj.session_token;

            if (cmdName === "ATTACH_UPLOAD_INIT" && responseObj.status === "OK") {
                lastUploadId = responseObj.data.upload.upload_id;
                console.log(`[AutoTester] Got upload_id: ${lastUploadId}, starting stream...`);

                // REAL ATTACHMENT FLOW
                try {
                    const stream = await wmtpTransport.openAttachmentStream();
                    const writer = stream.send;

                    // 1. Send header
                    const header = JSON.stringify({
                        upload_id: lastUploadId,
                        filename: "WMTP_Spec.pdf",
                        mime_type: "application/pdf",
                        size_bytes: 395069
                    }) + "\n";
                    await writer.write(new TextEncoder().encode(header));

                    // 2. Send dummy binary data (approx size of the requested PDF)
                    const dummyData = new Uint8Array(395069).fill(0x20); // Just fill with spaces or something
                    await writer.write(dummyData);

                    await writer.close();
                    console.log("[AutoTester] Attachment stream finished.");
                } catch (streamErr) {
                    console.error("[AutoTester] Attachment stream failed:", streamErr);
                    results.push({ command: "ATTACH_STREAM_ERROR", error: streamErr.message });
                }
            }

            // Capture IDs
            if (cmdName === "MSG_LIST" || cmdName === "MAIL_LIST") {
                if (responseObj.messages && responseObj.messages.length > 0) {
                    lastMsgId = responseObj.messages[0].id;
                }
            }
            if (cmdName === "MSG_SEND" && responseObj.data && responseObj.data.id) {
                lastMsgId = responseObj.data.id;
                if (responseObj.data.attachments && responseObj.data.attachments.length > 0) {
                    lastAttachId = responseObj.data.attachments[0].id;
                }
            }
            if (cmdName === "MSG_GET" && responseObj.data && responseObj.data.message && responseObj.data.message.attachments && responseObj.data.message.attachments.length > 0) {
                lastAttachId = responseObj.data.message.attachments[0].id;
            }

            await new Promise(r => setTimeout(r, 200)); // Slightly longer wait
        }

        status.textContent = "Tests Completed";
        runBtn.textContent = "Run Again";
        runBtn.disabled = false;
        resultsArea.value = JSON.stringify(results, null, 2);

    } catch (err) {
        console.error(err);
        status.textContent = "Error: " + err.message;
        runBtn.disabled = false;
        resultsArea.value = JSON.stringify(results, null, 2) + "\n\nError: " + err.toString();
    }
}

document.getElementById('runBtn').addEventListener('click', runTests);
