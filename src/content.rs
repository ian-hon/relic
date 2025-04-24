use std::fs;

use serde::{Deserialize, Serialize};

use crate::error::BonesError;

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

    pub fn create(name: String, path: String) -> Result<File, BonesError> {
        match fs::read_to_string(path) {
            Ok(content) => {
                Ok(File {
                    name: name,
                    content: content
                })
            },
            Err(_) => {
                // println!("Error creating file : {e}");
                Err(BonesError::FileCantOpen)
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
}
