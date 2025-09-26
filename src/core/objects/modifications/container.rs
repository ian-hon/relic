use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Tree {
    // denote that parent doesnt exist?

    // creation/deletion of files & folders
    CreateTree(
        String, // parent directory
        String, // name
    ),
    DeleteTree(
        String, // parent directory
        String, // name
    ),

    CreateBlob(
        String, // parent directory
        String, // name
    ),
    DeleteBlob(
        String, // parent directory
        String, // name
    ),
}
impl Tree {
    pub fn extract_data(&self) -> (String, String) {
        match self {
            Tree::CreateTree(path, name)
            | Tree::DeleteTree(path, name)
            | Tree::CreateBlob(path, name)
            | Tree::DeleteBlob(path, name) => (path.clone(), name.clone()),
        }
    }

    pub fn serialise(&self) -> String {
        format!(
            "{} {}",
            match self {
                Tree::CreateTree(_, _) => {
                    "+ D"
                }
                Tree::DeleteTree(_, _) => {
                    "- D"
                }
                Tree::CreateBlob(_, _) => {
                    "+ F"
                }
                Tree::DeleteBlob(_, _) => {
                    "- F"
                }
            },
            match self {
                Tree::CreateTree(p, n)
                | Tree::DeleteTree(p, n)
                | Tree::CreateBlob(p, n)
                | Tree::DeleteBlob(p, n) => {
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
