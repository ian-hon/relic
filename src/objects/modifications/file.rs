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
