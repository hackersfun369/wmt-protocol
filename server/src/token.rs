use rand::{distributions::Alphanumeric, Rng};
use sha2::{Digest, Sha256};
use hmac::{Hmac, Mac};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use serde_json::json;
use std::time::{SystemTime, UNIX_EPOCH};

type HmacSha256 = Hmac<Sha256>;

const DEFAULT_SECRET: &str = "wmtp-default-secret-change-in-production";
const JWT_EXPIRY_SECS: u64 = 7 * 24 * 3600; // 7 days

fn jwt_secret() -> String {
    std::env::var("WMTP_JWT_SECRET").unwrap_or_else(|_| DEFAULT_SECRET.to_string())
}

fn base64url(data: &[u8]) -> String {
    URL_SAFE_NO_PAD.encode(data)
}

fn base64url_decode(s: &str) -> Result<Vec<u8>, ()> {
    URL_SAFE_NO_PAD.decode(s).map_err(|_| ())
}

/// Issue a JWT containing { sub: email, tok: session_token, exp: unix+7d }
pub fn issue_jwt(email: &str, session_token: &str) -> String {
    let header = base64url(br#"{"alg":"HS256","typ":"JWT"}"#);
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
    let payload_json = json!({
        "sub": email,
        "tok": session_token,
        "iat": now,
        "exp": now + JWT_EXPIRY_SECS
    }).to_string();
    let payload = base64url(payload_json.as_bytes());
    let signing_input = format!("{}.{}", header, payload);
    let secret = jwt_secret();
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).expect("HMAC init");
    mac.update(signing_input.as_bytes());
    let sig = base64url(&mac.finalize().into_bytes());
    format!("{}.{}", signing_input, sig)
}

#[derive(Debug)]
pub struct JwtClaims {
    pub email: String,
    pub session_token: String,
}

/// Verify a JWT. Returns claims if valid and not expired.
pub fn verify_jwt(jwt: &str) -> Option<JwtClaims> {
    let parts: Vec<&str> = jwt.splitn(3, '.').collect();
    if parts.len() != 3 { return None; }
    let signing_input = format!("{}.{}", parts[0], parts[1]);
    let secret = jwt_secret();
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).ok()?;
    mac.update(signing_input.as_bytes());
    let expected_sig = base64url(&mac.finalize().into_bytes());
    if expected_sig != parts[2] { return None; }

    let payload_bytes = base64url_decode(parts[1]).ok()?;
    let payload: serde_json::Value = serde_json::from_slice(&payload_bytes).ok()?;

    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
    let exp = payload["exp"].as_u64()?;
    if now > exp { return None; } // expired

    let email = payload["sub"].as_str()?.to_string();
    let session_token = payload["tok"].as_str()?.to_string();
    Some(JwtClaims { email, session_token })
}

pub fn generate_ephemeral_token() -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(32)
        .map(char::from)
        .collect()
}

pub fn token_from_email(email: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(email.trim().to_lowercase().as_bytes());
    let digest = hasher.finalize();
    format!("{:x}", digest)
}
