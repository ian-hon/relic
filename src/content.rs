use std::fs;

use serde::{Deserialize, Serialize};

use crate::error::RelicError;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Content {
    Directory(Directory),
    File(File),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct File {
    pub name: String,
    pub content: String,
}

impl File {
    pub fn new() -> File {
        File {
            name: "".to_string(),
            content: "".to_string()
        }
    }

    pub fn create(name: String, path: String) -> Result<File, RelicError> {
        match fs::read_to_string(path) {
            Ok(content) => {
                Ok(File {
                    name: name,
                    content: content
                })
            },
            Err(_) => {
                // println!("Error creating file : {e}");
                Err(RelicError::FileCantOpen)
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Directory {
    pub name: String,
    pub content: Vec<Content>
}

impl Directory {
    pub fn new() -> Directory {
        Directory {
            name: "".to_string(),
            content: vec![]
        }
    }

    pub fn deserialise(s: String) -> Option<Directory> {
        match serde_json::from_str(&s) {
            Ok(d) => Some(d),
            _ => None
        }
    }

    pub fn serialise(&self) -> String {
        serde_json::to_string(&self).unwrap()
    }
}
