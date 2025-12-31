use rand::{distributions::Alphanumeric, Rng};
use sha2::{Digest, Sha256};

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
    format!("{:x}", digest) // 64-char hex token [web:204][web:210]
}
