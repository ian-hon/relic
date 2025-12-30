use std::{fs, path::PathBuf};

use serde::{Deserialize, Serialize};

use crate::core::{error::RelicError, modifications};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Blob {
    pub name: String,
    pub content: String,
}

impl Blob {
    pub fn new() -> Blob {
        Blob {
            name: "".to_string(),
            content: "".to_string(),
        }
    }

    pub fn create(name: String, path: PathBuf) -> Result<Blob, RelicError> {
        match fs::read_to_string(path) {
            Ok(content) => Ok(Blob {
                name: name,
                content: content,
            }),
            Err(_) => Err(RelicError::FileCantOpen),
        }
    }

    pub fn apply_changes(&mut self, modifications: &Vec<modifications::Blob>) {
        // TODO : investigate whether an additional newline is added to eof
        // BUG : when the file has only one line, diffs start to break
        //
        // content : ""
        // Create(,, 0, "something")
        // result : "something\nsomething"
        //
        // content : "something\nsomething"
        // Delete(,, 0)
        // result : ""

        // CHANGES ARE BEING APPLIED TO THE WRONG FILE
        // APPLY CHANGES TO UPSTREAM, NOT CURRENT

        // TODO : revise modification order

        // deletions first then creations?
        //      sorted largest to smallest
        // creations sorted smallest to largest?
        let mut lines = self
            .content
            .split("\n")
            .map(|x| x.to_string())
            .collect::<Vec<String>>();

        let mut modifications = modifications.clone();
        modifications.sort_by_key(|m| match m {
            modifications::Blob::Create(_, _, l, _) => *l as i128,
            modifications::Blob::Delete(_, _, l, _) => -(*l as i128),
        });

        for m in &modifications {
            match m {
                modifications::Blob::Create(_, _, line, content) => {
                    // insert at that line
                    lines.insert(*line, content.clone());
                }
                modifications::Blob::Delete(_, _, line, _) => {
                    // delete that line
                    lines.remove(*line);
                }
            }
        }

        self.content = lines.join("\n");
    }
}
