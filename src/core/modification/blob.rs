use serde::{Deserialize, Serialize};

use crate::core::modification::change::ModOp;

// #[derive(Debug, Clone, Serialize, Deserialize, Hash, PartialEq, Eq, PartialOrd, Ord)]
// pub enum Blob {
//     // creation/deletion of lines in files
//     Create(
//         String, // parent directory
//         String, // file name
//         usize,  // line
//         String, // text
//     ),
//     Delete(
//         String, // parent directory
//         String, // file name
//         usize,  // line
//         String, // text
//     ),
// }

#[derive(Debug, Clone, Serialize, Deserialize, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct BlobOpInfo {
    pub parent: String,
    pub file: String,
    pub line: usize,
    pub text: String,
}
impl BlobOpInfo {
    pub fn new(parent: String, file: String, line: usize, text: String) -> BlobOpInfo {
        BlobOpInfo {
            parent,
            file,
            line,
            text,
        }
    }

    pub fn extract_path(&self) -> (String, String) {
        // EXPENSIVE!
        (self.parent.clone(), self.file.clone())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct BlobOp {
    pub mod_op: ModOp,
    pub info: BlobOpInfo,
}
impl BlobOp {
    pub fn new(mod_op: ModOp, info: BlobOpInfo) -> BlobOp {
        BlobOp { mod_op, info }
    }

    pub fn extract_change(&self) -> String {
        format!(
            "{} {}",
            self.mod_op.get_notation(),
            format!("{} {:?}", self.info.line, self.info.text)
        )
    }
}
