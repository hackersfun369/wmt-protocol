
const CERT_HASH = "9kDUV0kAxsCBObFdiULtY3w5b0xcp8l6A+uF7Ds9yFc=";
const SERVER_URL = "https://localhost:4433";
const USER_EMAIL = "test@auto.com";

const TEST_SEQUENCE = [
    { cmd: "INIT", data: {} },
    { cmd: "AUTH", data: { email: USER_EMAIL } },
    {
        cmd: "ATTACH_UPLOAD_INIT",
        data: {
            session_token: "$TOKEN",
            filename: "Attachment_Test.pdf",
            mime_type: "application/pdf",
            size_bytes: 1024
        }
    },
    {
        cmd: "MSG_SEND",
        data: {
            session_token: "$TOKEN",
            to: [USER_EMAIL],
            subject: "Attachment Verification",
            body: "Testing ATTACH_GET after MSG_SEND",
            attachment_ids: ["$UPLOAD_ID"]
        }
    },
    { cmd: "MSG_GET", data: { session_token: "$TOKEN", id: "$MSG_ID" } },
    { cmd: "ATTACH_GET", data: { session_token: "$TOKEN", attachment_id: "$ATTACH_ID" } },
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
    status.textContent = "Running Attachment Tests...";

    try {
        wmtpTransport.setCertificateHash(CERT_HASH);
        await wmtpTransport.connect(SERVER_URL);

        for (let i = 0; i < TEST_SEQUENCE.length; i++) {
            let step = TEST_SEQUENCE[i];
            let cmdName = step.cmd;
            let payload = JSON.parse(JSON.stringify(step.data));

            progress.textContent = `Executing ${i + 1}/${TEST_SEQUENCE.length}: ${cmdName}`;

            if (payload.session_token === "$TOKEN") payload.session_token = sessionToken;
            if (payload.id === "$MSG_ID") payload.id = lastMsgId;
            if (payload.attachment_ids && payload.attachment_ids[0] === "$UPLOAD_ID") payload.attachment_ids = [lastUploadId];
            if (payload.attachment_id === "$ATTACH_ID") payload.attachment_id = lastAttachId;

            const result = await wmtpTransport.sendWithTiming({ cmd: cmdName, data: payload });
            let responseObj = result.response;

            results.push({
                command: cmdName,
                request: { cmd: cmdName, data: payload },
                response: responseObj,
                latency_ms: result.responseTimeMs
            });

            if (cmdName === "INIT" || cmdName === "AUTH") {
                if (responseObj.session_token) sessionToken = responseObj.session_token;
            }

            if (cmdName === "ATTACH_UPLOAD_INIT" && responseObj.status === "OK") {
                lastUploadId = responseObj.data.upload.upload_id;
                try {
                    const stream = await wmtpTransport.openAttachmentStream();
                    const writer = stream.send;
                    const header = JSON.stringify({
                        upload_id: lastUploadId,
                        filename: "Attachment_Test.pdf",
                        mime_type: "application/pdf",
                        size_bytes: 1024
                    }) + "\n";
                    await writer.write(new TextEncoder().encode(header));
                    await writer.write(new Uint8Array(1024).fill(0x20));
                    await writer.close();
                    console.log("[AutoTester] Stream upload finished");
                } catch (e) {
                    results.push({ command: "STREAM_ERROR", error: e.message });
                }
            }

            if (cmdName === "MSG_SEND" && responseObj.data && responseObj.data.id) {
                lastMsgId = responseObj.data.id;
            }
            if (cmdName === "MSG_GET" && responseObj.data && responseObj.data.message.attachments.length > 0) {
                lastAttachId = responseObj.data.message.attachments[0].id;
            }

            await new Promise(r => setTimeout(r, 500));
        }

        status.textContent = "Tests Completed";
        runBtn.disabled = false;
        resultsArea.value = JSON.stringify(results, null, 2);

    } catch (err) {
        status.textContent = "Error: " + err.message;
        runBtn.disabled = false;
        resultsArea.value = JSON.stringify(results, null, 2) + "\n\nError: " + err.toString();
    }
}

document.getElementById('runBtn').addEventListener('click', runTests);
