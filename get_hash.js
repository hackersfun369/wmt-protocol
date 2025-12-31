const fs = require('fs');
const crypto = require('crypto');
const path = require('path');

const certPath = path.join(__dirname, 'certs', 'cert.pem');

try {
    const pem = fs.readFileSync(certPath, 'utf8');
    const lines = pem.split('\n');
    let body = '';
    let inCert = false;
    for (const line of lines) {
        if (line.includes('-----BEGIN CERTIFICATE-----')) { inCert = true; continue; }
        if (line.includes('-----END CERTIFICATE-----')) { break; }
        if (inCert) { body += line.trim(); }
    }

    if (!body) {
        console.error('No cert body found');
        process.exit(1);
    }

    const der = Buffer.from(body, 'base64');
    const hash = crypto.createHash('sha256').update(der).digest('base64');
    console.log(hash);

} catch (e) {
    console.error(e);
}
