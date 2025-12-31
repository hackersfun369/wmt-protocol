# WMTP - WebTransport Mail Transfer Protocol

![Version](https://img.shields.io/badge/version-0.1.0-blue)
![License](https://img.shields.io/badge/license-MIT-green)
![Transport](https://img.shields.io/badge/transport-WebTransport-purple)

A modern, secure, low-latency email transfer protocol built on QUIC/WebTransport.

🌐 **Website:** [https://wmtp.online](https://wmtp.online)

## ✨ Features

- 🚀 **Built on QUIC** - Faster connections, multiplexed streams
- 🔒 **Always Encrypted** - TLS 1.3 mandatory
- ⚡ **Low Latency** - Real-time message delivery
- 🌐 **Browser Native** - Works via WebTransport API
- 📱 **Mobile Friendly** - Connection migration support
- 🔧 **Simple Protocol** - JSON-based, easy to implement

## 🏗️ Architecture

┌──────────┐ WebTransport/QUIC ┌──────────┐
│ Client │◄─────────────────────────►│ Server │
│ (Browser)│ │ (Rust) │
└──────────┘ └──────────┘
text

## 🚀 Quick Start

### Prerequisites
- Rust 1.70+
- OpenSSL 3.x
- Modern browser (Chrome 97+, Edge 97+, Firefox 114+)

### 1. Clone
```bash
git clone https://github.com/yourusername/wmtp.git
cd wmtp
2. Generate Certificates
bash
cd certs
openssl genpkey -algorithm RSA -out key.pem
openssl req -new -x509 -key key.pem -out cert.pem -days 10 -subj "/CN=localhost"  /// browsers can only validate certificates only for a limited time
3. Run Server
bash
cd server
cargo run
4. Open Client
Open client/app.html in your browser.
📁 Project Structure
text
wmtp/
├── server/          # Rust WebTransport server
├── client/          # Browser client (HTML/JS)
├── certs/           # TLS certificates
├── docs/            # Documentation
└── scripts/         # Deployment scripts
📖 Protocol
Commands
Command	Description
INIT	Initialize session
AUTH	Authenticate with email
RESUME	Resume session
LOGOUT	End session
PING	Test connectivity
STATUS	Server status
INFO	Server info
Example
javascript
// Connect
const transport = new WebTransport("https://localhost:4433");
await transport.ready;

// Authenticate
send({ cmd: "AUTH", data: { email: "user@example.com" } });
See docs/PROTOCOL.md for full specification.
🌍 Deployment
Server (VPS)
bash
./scripts/setup-server.sh
Client (Cloudflare Pages)
Push client/ to GitHub, connect to Cloudflare Pages.
🛠️ Development
Server
bash
cd server
cargo run          # Development
cargo build --release  # Production
cargo test         # Run tests
Client
Simply open client/app.html in browser.
📄 License
MIT License - see LICENSE
🙏 Acknowledgments
wtransport - Rust WebTransport
tokio - Async runtime
Made with ❤️ for the future of email
