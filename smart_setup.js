const { execSync } = require('child_process');
const fs = require('fs');
const crypto = require('crypto');
const path = require('path');

const certsDir = path.join(__dirname, 'certs');
const certPath = path.join(certsDir, 'cert.pem');
const keyPath = path.join(certsDir, 'key.pem');
const confPath = path.join(certsDir, 'localhost.conf');
const uiJsPath = path.join(__dirname, 'client', 'js', 'ui.js');

console.log('🔄 Starting Smart Certificate Setup...');

try {
    // 1. Generate ECDSA Private Key (Prime256v1)
    console.log('🔑 Generating ECDSA private key...');
    execSync(`openssl ecparam -name prime256v1 -genkey -noout -out "${keyPath}"`);

    // 2. Generate Self-Signed Certificate with SANs using Config
    console.log('📜 Generating Certificate (Valid 10 days, with SANs)...');
    // Note: -days 10 is critical for WebTransport self-signed hash verification
    execSync(`openssl req -new -x509 -key "${keyPath}" -out "${certPath}" -days 10 -config "${confPath}"`);

    // 3. Calculate SHA-256 Hash
    console.log('ZOmq Calculating Certificate Hash...');
    const pem = fs.readFileSync(certPath, 'utf8');
    const lines = pem.split('\n');
    let body = '';
    let inCert = false;
    for (const line of lines) {
        if (line.includes('-----BEGIN CERTIFICATE-----')) {
            inCert = true;
            continue;
        }
        if (line.includes('-----END CERTIFICATE-----')) {
            break;
        }
        if (inCert) {
            body += line.trim();
        }
    }
    const der = Buffer.from(body, 'base64');
    const hash = crypto.createHash('sha256').update(der).digest('base64');

    console.log(`✨ New Hash: ${hash}`);

    // 4. Update Client Code
    console.log('📝 Updating client/js/ui.js...');
    if (fs.existsSync(uiJsPath)) {
        let content = fs.readFileSync(uiJsPath, 'utf8');
        const regex = /wmtpTransport\.setCertificateHash\(['"][^'"]+['"]\);/;
        if (regex.test(content)) {
            const newContent = content.replace(regex, `wmtpTransport.setCertificateHash('${hash}');`);
            fs.writeFileSync(uiJsPath, newContent, 'utf8');
            console.log('✅ Client updated successfully.');
        } else {
            console.warn('⚠️ Could not find setCertificateHash in ui.js');
        }
    } else {
        console.error('❌ client/js/ui.js not found!');
    }

    console.log('\n🎉 Setup Complete!');
    console.log('👉 IMPORTANT: Restart your Rust server now to apply changes.');

} catch (error) {
    console.error('❌ Error during setup:', error.message);
    process.exit(1);
}
