use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Blob {
    // creation/deletion of lines in files
    Create(
        String, // parent directory
        String, // file name
        usize,  // line
        String, // text
    ),
    Delete(
        String, // parent directory
        String, // file name
        usize,  // line
        String, // text
    ),
}

impl Blob {
    pub fn extract_path(&self) -> (String, String) {
        match self {
            Blob::Create(path, name, _, _) | Blob::Delete(path, name, _, _) => {
                (path.clone(), name.clone())
            }
        }
    }

    pub fn extract_change(&self) -> String {
        format!(
            "{} {}",
            match self {
                Blob::Create(_, _, _, _) => "+",
                Blob::Delete(_, _, _, _) => "-",
            },
            match self {
                Blob::Create(_, _, line, content) | Blob::Delete(_, _, line, content) => {
                    format!("{line} {content:?}")
                }
            }
        )
    }
}
