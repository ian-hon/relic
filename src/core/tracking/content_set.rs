use std::{collections::HashSet, fs, path::Path};

use crate::core::error::{IOError, RelicError};

pub struct ContentSet(HashSet<String>);
impl ContentSet {
    pub fn new() -> ContentSet {
        ContentSet(HashSet::new())
    }

    pub fn construct(path: &Path) -> Result<ContentSet, RelicError> {
        if !path.exists() {
            // create new
            let c = ContentSet::new();
            if let Ok(_) = fs::write(path, c.serialise()) {
                return Ok(c);
            }
            return Err(RelicError::IOError(IOError::FileCantCreate));
        }

        if let Ok(s) = fs::read_to_string(path) {
            return Ok(ContentSet::deserialise(s));
        }

        Err(RelicError::IOError(IOError::FileCantOpen))
    }

    pub fn serialise(&self) -> String {
        "".to_string()
    }

    pub fn deserialise(payload: String) -> ContentSet {
        let mut result = HashSet::new();

        for line in payload.lines() {
            if line.starts_with("--") {
                continue;
            }
            result.insert(line.to_string());
        }

        ContentSet(result)
    }
}
