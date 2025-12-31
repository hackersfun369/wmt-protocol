/**
 * WMTP Transport Layer
 * Handles WebTransport connection management
 */

class WMTPTransport {
    constructor() {
        this.transport = null;
        this.writer = null;
        this.reader = null;
        this.connected = false;
        this.onMessage = null;
        this.onConnect = null;
        this.onDisconnect = null;
        this.onError = null;

        // Certificate hash for self-signed certs
        this.certHash = null;

        // For measuring response time of a single in-flight request
        this._pendingResolve = null;
    }

    /**
     * Set certificate hash for self-signed certificates
     * @param {string} hash - Base64 encoded SHA-256 hash
     */
    setCertificateHash(hash) {
        this.certHash = hash;
    }

    /**
     * Connect to WMTP server
     * @param {string} url - Server URL (e.g., "https://localhost:4433")
     */
    async connect(url) {
        try {
            // Check WebTransport support
            if (typeof WebTransport === 'undefined') {
                throw new Error('WebTransport is not supported. Use Chrome 97+ or Edge 97+');
            }

            console.log(`[Transport] Connecting to ${url}...`);

            // Build connection options
            let options = {};

            // For self-signed certificates, use certificate hash
            if (this.certHash) {
                options.serverCertificateHashes = [
                    {
                        algorithm: 'sha-256',
                        value: this._base64ToArrayBuffer(this.certHash),
                    },
                ];
                console.log('[Transport] Using certificate hash for self-signed cert');
            }

            // Create WebTransport connection
            this.transport = new WebTransport(url, options);

            // Wait for connection to be ready
            await this.transport.ready;
            console.log('[Transport] Connected!');

            // Open bidirectional stream for control
            const stream = await this.transport.createBidirectionalStream();
            this.writer = stream.writable.getWriter();
            this.reader = stream.readable.getReader();

            this.connected = true;

            // Start reading incoming messages
            this._startReading();

            // Handle connection close
            this.transport.closed
                .then(() => {
                    console.log('[Transport] Connection closed');
                    this._handleDisconnect();
                })
                .catch((err) => {
                    console.error('[Transport] Connection error:', err);
                    this._handleDisconnect();
                });

            // Trigger connect callback
            if (this.onConnect) {
                this.onConnect();
            }

            return true;
        } catch (error) {
            console.error('[Transport] Connection failed:', error);
            if (this.onError) {
                this.onError(error);
            }
            throw error;
        }
    }

    /**
     * Disconnect from server
     */
    async disconnect() {
        if (this.transport) {
            try {
                await this.transport.close();
            } catch (e) {
                // Ignore close errors
            }
        }
        this._handleDisconnect();
    }

    /**
     * Send data to server
     * @param {string|object} data - Data to send
     */
    async send(data) {
        if (!this.connected || !this.writer) {
            throw new Error('Not connected');
        }

        const message = typeof data === 'string' ? data : JSON.stringify(data);
        const encoded = new TextEncoder().encode(message);

        await this.writer.write(encoded);
        console.log('[Transport] Sent:', message);
    }

    /**
     * Send one JSON command and measure response time.
     * Resolves with { response, responseTimeMs }.
     * @param {string|object} data
     */
    async sendWithTiming(data) {
        if (!this.connected || !this.writer) {
            throw new Error('Not connected');
        }

        const originalOnMessage = this.onMessage;

        const responsePromise = new Promise((resolve) => {
            this._pendingResolve = (msg, rt) => {
                this._pendingResolve = null;
                resolve({ response: msg, responseTimeMs: rt });
            };
        });

        const start = performance.now();
        const message = typeof data === 'string' ? data : JSON.stringify(data);
        const encoded = new TextEncoder().encode(message);
        await this.writer.write(encoded);
        console.log('[Transport] Sent (timed):', message);

        // Temporarily intercept onMessage
        this.onMessage = (msg) => {
            const end = performance.now();
            const rt = end - start;
            console.log(`[Transport] Response time: ${rt.toFixed(2)} ms`);

            if (this._pendingResolve) {
                this._pendingResolve(msg, rt);
            }

            // Also forward to original handler if it exists
            if (originalOnMessage) {
                originalOnMessage(msg);
            }

            // Restore original handler
            this.onMessage = originalOnMessage;
        };

        return responsePromise;
    }

    /**
     * Convert base64 string to ArrayBuffer
     * @param {string} base64 - Base64 encoded string
     * @returns {ArrayBuffer}
     */
    _base64ToArrayBuffer(base64) {
        const binaryString = atob(base64);
        const bytes = new Uint8Array(binaryString.length);
        for (let i = 0; i < binaryString.length; i++) {
            bytes[i] = binaryString.charCodeAt(i);
        }
        return bytes.buffer;
    }

    /**
     * Start reading from stream
     */
    async _startReading() {
        const decoder = new TextDecoder();

        try {
            while (this.connected) {
                const { value, done } = await this.reader.read();

                if (done) {
                    console.log('[Transport] Stream ended');
                    break;
                }

                if (value) {
                    const message = decoder.decode(value);
                    console.log('[Transport] Received:', message);

                    const messages = this._parseMessages(message);
                    messages.forEach((msg) => {
                        if (this.onMessage) {
                            this.onMessage(msg);
                        }
                    });
                }
            }
        } catch (error) {
            if (this.connected) {
                console.error('[Transport] Read error:', error);
            }
        }
    }

    /**
     * Parse potentially multiple JSON messages
     * @param {string} text - Raw text that may contain multiple JSON objects
     * @returns {Array} Array of parsed objects
     */
    _parseMessages(text) {
        const messages = [];
        const parts = text.trim().split(/\}\s*\{/);

        parts.forEach((part, index) => {
            let json = part;
            if (index > 0) json = '{' + json;
            if (index < parts.length - 1) json = json + '}';

            try {
                messages.push(JSON.parse(json));
            } catch (e) {
                console.warn('[Transport] Parse error:', e, json);
            }
        });

        return messages;
    }

    /**
     * Handle disconnect cleanup
     */
    _handleDisconnect() {
        this.connected = false;
        this.transport = null;
        this.writer = null;
        this.reader = null;

        if (this.onDisconnect) {
            this.onDisconnect();
        }
    }
    // attachments
    async openAttachmentStream() {
    if (!this.transport) {
        throw new Error('Not connected');
    }
    const stream = await this.transport.createBidirectionalStream();
    return {
        send: stream.writable.getWriter(),
        recv: stream.readable.getReader(),
    };
}


    /**
     * Check if connected
     * @returns {boolean}
     */
    isConnected() {
        return this.connected;
    }
}

// Global instance
const wmtpTransport = new WMTPTransport();
