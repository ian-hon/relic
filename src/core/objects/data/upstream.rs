// upstream might be stored in differing formats
// this is here to ensure backwards compatibility with outdated standards and etc

use std::{fs, path::PathBuf};

use serde::{Deserialize, Serialize};

use crate::{
    core::{Content, Tree},
    error::RelicError,
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Upstream {
    pub convention: String, // used for backwards compatibility
    pub content: Vec<Content>,
}
impl Upstream {
    pub fn empty() -> Upstream {
        Upstream {
            convention: "0.0.1".to_string(),
            content: vec![],
        }
    }

    pub fn tree(self) -> Tree {
        Tree {
            path: PathBuf::from("."),
            name: "".to_string(),
            content: self.content,
        }
    }

    pub fn from_tree(tree: &Tree) -> Upstream {
        Upstream {
            convention: "0.0.1".to_string(),
            content: tree.content.clone(),
        }
    }

    pub fn serialise(&self) -> String {
        serde_json::to_string(&self).unwrap()
    }

    pub fn deserialise(path: &str) -> Result<Upstream, RelicError> {
        match fs::read_to_string(path) {
            Ok(data) => match serde_json::from_str::<Upstream>(&data) {
                Ok(u) => Ok(u),
                // TODO: implement backwards compatibility
                Err(_) => Err(RelicError::ConfigurationIncorrect),
            },
            Err(_) => Err(RelicError::FileCantOpen),
        }
    }
}
