use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum File {
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

impl File {
    pub fn extract_path(&self) -> (String, String) {
        match self {
            File::Create(path, name, _, _) | File::Delete(path, name, _, _) => {
                (path.clone(), name.clone())
            }
        }
    }

    pub fn extract_change(&self) -> String {
        format!(
            "{} {}",
            match self {
                File::Create(_, _, _, _) => "+",
                File::Delete(_, _, _, _) => "-",
            },
            match self {
                File::Create(_, _, line, content) | File::Delete(_, _, line, content) => {
                    format!("{line} {content:?}")
                }
            }
        )
    }
}
