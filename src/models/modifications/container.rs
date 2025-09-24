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
