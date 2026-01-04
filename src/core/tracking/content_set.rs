use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};

use crate::core::error::{IOError, RelicError};

pub struct ContentSet {
    pub files: HashSet<String>,
    pub directories: HashSet<String>,
}
impl ContentSet {
    pub fn new() -> ContentSet {
        ContentSet {
            files: HashSet::new(),
            directories: HashSet::new(),
        }
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
        format!(
            "{}\n\n{}",
            self.directories
                .iter()
                .fold("".to_string(), |mut left, right| {
                    left.push_str(&right[2..]);
                    // left.push_str(right);
                    left.push_str("/\n");
                    left
                })
                .trim_end()
                .to_string(),
            self.files
                .iter()
                .fold("".to_string(), |mut left, right| {
                    left.push_str(&right[2..]);
                    // left.push_str(right);
                    left.push_str("\n");
                    left
                })
                .trim_end()
                .to_string()
        )
    }

    pub fn deserialise(payload: String) -> ContentSet {
        let mut files = HashSet::new();
        let mut directories = HashSet::new();

        for line in payload.lines() {
            if line.starts_with("--") || line.is_empty() {
                continue;
            }

            if line.ends_with("/") {
                let l = line.len() - 1;
                directories.insert(format!("./{}", line[..l].to_owned()));
            } else {
                files.insert(format!("./{line}"));
            }
        }

        ContentSet { files, directories }
    }

    pub fn append(&mut self, paths: Vec<PathBuf>) {
        for i in paths {
            let r: String = i.to_string_lossy().into();

            if i.is_file() {
                self.files.insert(format!("./{r}"));
            } else {
                self.directories.insert(format!("./{r}"));
            }
        }
    }

    pub fn remove(&mut self, paths: Vec<PathBuf>) {
        for i in paths {
            let r: String = i.to_string_lossy().into();

            if i.is_file() {
                // self.files.insert(format!("./{r}"));
                self.files.remove(&format!("./{r}"));
            } else {
                self.directories.remove(&format!("./{r}"));
                // self.directories.insert(format!("./{r}"));
            }
        }
    }
}
