const { execSync } = require('child_process');
const fs = require('fs');
const crypto = require('crypto');
const path = require('path');

const certsDir = path.join(__dirname, 'certs');
const certPath = path.join(certsDir, 'cert.pem');
const keyPath = path.join(certsDir, 'key.pem');
const confPath = path.join(certsDir, 'localhost.conf');

const FILES_TO_UPDATE = [
    {
        path: path.join(__dirname, 'client', 'js', 'transport.js'),
        regex: /this\.certHash = ('[^']+'|"[^"]+"|null);/g,
        template: (hash) => `this.certHash = '${hash}';`
    },
    {
        path: path.join(__dirname, 'client', 'js', 'ui.js'),
        regex: /wmtpTransport\.setCertificateHash\(['"][^'"]+['"]\);/g,
        template: (hash) => `wmtpTransport.setCertificateHash('${hash}');`
    }
];

console.log('🔄 Starting Smart Certificate Setup...');

try {
    // 1. Generate ECDSA Private Key (Prime256v1)
    console.log('🔑 Generating ECDSA private key...');
    execSync(`openssl ecparam -name prime256v1 -genkey -noout -out "${keyPath}"`);

    // 2. Generate Self-Signed Certificate with SANs using Config
    console.log('📜 Generating Certificate (Valid 10 days, with SANs)...');
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

    // 4. Update Client Files
    FILES_TO_UPDATE.forEach(file => {
        const relativePath = path.relative(__dirname, file.path);
        console.log(`📝 Updating ${relativePath}...`);

        if (fs.existsSync(file.path)) {
            let content = fs.readFileSync(file.path, 'utf8');
            if (file.regex.test(content)) {
                const newContent = content.replace(file.regex, file.template(hash));
                fs.writeFileSync(file.path, newContent, 'utf8');
                console.log(`✅ ${relativePath} updated successfully.`);
            } else {
                console.warn(`⚠️ Could not find pattern in ${relativePath}`);
            }
        } else {
            console.warn(`⚠️ File not found: ${relativePath}`);
        }
    });

    console.log('\n🎉 Setup Complete!');
    console.log('👉 IMPORTANT: Restart your Rust server now to apply changes.');

} catch (error) {
    console.error('❌ Error during setup:', error.message);
    process.exit(1);
}
