use sha2::{Digest, Sha256};

pub fn empty_oid() -> [u8; 32] {
    [
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0,
    ]
}

pub fn oid_to_string(oid: [u8; 32]) -> String {
    // format!("{:x}", oid)
    // TODO: test
    String::from_utf8(oid.to_vec()).unwrap()
}

pub fn oid_digest(content: &str) -> [u8; 32] {
    Sha256::digest(content).as_slice().try_into().unwrap()
}
