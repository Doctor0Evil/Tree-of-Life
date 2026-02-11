use sha2::{Digest, Sha256};

pub fn compute_sha256_hash(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let bytes = hasher.finalize();
    format!("{:x}", bytes)
}
