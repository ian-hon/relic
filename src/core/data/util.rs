use sha2::{Digest, Sha256};

pub fn empty_oid() -> [u8; 32] {
    [
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0,
    ]
}

pub fn oid_to_string(oid: [u8; 32]) -> String {
    oid.iter()
        .map(|byte| format!("{:02x}", byte))
        .collect::<String>()
}

pub fn string_to_oid(content: &str) -> [u8; 32] {
    content.as_bytes().try_into().unwrap()
}

pub fn oid_digest(content: &str) -> [u8; 32] {
    Sha256::digest(content).as_slice().try_into().unwrap()
}

pub fn oid_digest_data(content: &Vec<u8>) -> [u8; 32] {
    Sha256::digest(content).as_slice().try_into().unwrap()
}
