use std::time::{Duration, SystemTime, UNIX_EPOCH};

use chrono::{DateTime, Utc};
use sha2::{Digest, Sha256};
use urlencoding::{decode, encode};

// #region OID
// TODO: separate these out somewhere else
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
    // will this haunt me later?

    // let content = content.to_string();
    let content = format!("{:0>64}", content);
    (0..content.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&content[i..i + 2], 16).unwrap())
        .collect::<Vec<u8>>()
        .try_into()
        .unwrap()
}

pub fn oid_digest(content: &str) -> [u8; 32] {
    Sha256::digest(content).as_slice().try_into().unwrap()
}

pub fn oid_digest_data(content: &Vec<u8>) -> [u8; 32] {
    Sha256::digest(content).as_slice().try_into().unwrap()
}
// #endregion

// #region url encoding
pub fn url_encode(s: &str) -> String {
    encode(s).into_owned()
}

pub fn url_decode(s: &str) -> String {
    decode(s).unwrap().into_owned()
}
// #endregion

pub fn get_time() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time went backwards (???)")
        .as_millis() as u64
}

pub fn into_human_readable(t: u64) -> String {
    // accepts unix time, but only in milliseconds format
    DateTime::<Utc>::from(UNIX_EPOCH + Duration::from_millis(t as u64))
        .format("%Y-%m-%d %H:%M:%S")
        .to_string()
}
