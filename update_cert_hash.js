const fs = require('fs');
const crypto = require('crypto');
const path = require('path');

const certPath = path.join(__dirname, 'certs', 'cert.pem');
const uiJsPath = path.join(__dirname, 'client', 'js', 'ui.js');

try {
    // 1. Read Certificate
    if (!fs.existsSync(certPath)) {
        console.error(`Certificate not found at: ${certPath}`);
        process.exit(1);
    }
    const pem = fs.readFileSync(certPath, 'utf8');

    // Extract the base64 body
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

    if (!body) {
        console.error('Could not find certificate body in pem file');
        process.exit(1);
    }

    // 2. Calculate Hash
    const der = Buffer.from(body, 'base64');
    const hash = crypto.createHash('sha256').update(der).digest('base64');

    console.log(`Calculated new Certificate Hash: ${hash}`);

    // 3. Update client/js/ui.js
    if (!fs.existsSync(uiJsPath)) {
        console.error(`UI file not found at: ${uiJsPath}`);
        process.exit(1);
    }

    let uiJsContent = fs.readFileSync(uiJsPath, 'utf8');

    // Regex to find the setCertificateHash call
    // Matches: wmtpTransport.setCertificateHash('...');
    const regex = /wmtpTransport\.setCertificateHash\(['"][^'"]+['"]\);/;

    if (regex.test(uiJsContent)) {
        const newContent = uiJsContent.replace(regex, `wmtpTransport.setCertificateHash('${hash}');`);
        fs.writeFileSync(uiJsPath, newContent, 'utf8');
        console.log(`✅ Successfully updated client/js/ui.js with new hash.`);
    } else {
        console.warn(`⚠️ Could not find wmtpTransport.setCertificateHash(...) in ui.js to replace.`);
    }

} catch (err) {
    console.error('Error:', err);
    process.exit(1);
}
