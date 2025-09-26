use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Container {
    // denote that parent doesnt exist?

    // creation/deletion of files & folders
    CreateDirectory(
        String, // parent directory
        String, // name
    ),
    DeleteDirectory(
        String, // parent directory
        String, // name
    ),

    CreateFile(
        String, // parent directory
        String, // name
    ),
    DeleteFile(
        String, // parent directory
        String, // name
    ),
}
impl Container {
    pub fn serialise(&self) -> String {
        format!(
            "{} {}",
            match self {
                Container::CreateDirectory(_, _) => {
                    "+ D"
                }
                Container::DeleteDirectory(_, _) => {
                    "- D"
                }
                Container::CreateFile(_, _) => {
                    "+ F"
                }
                Container::DeleteFile(_, _) => {
                    "- F"
                }
            },
            match self {
                Container::CreateDirectory(p, n)
                | Container::DeleteDirectory(p, n)
                | Container::CreateFile(p, n)
                | Container::DeleteFile(p, n) => {
                    format!(
                        "{} {}",
                        urlencoding::encode(&p).to_string(),
                        urlencoding::encode(&n).to_string()
                    )
                }
            }
        )
    }
}
