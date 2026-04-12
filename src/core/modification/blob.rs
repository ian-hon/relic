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
pub struct BlobOp {
    pub mod_op: ModOp,
    pub parent: String,
    pub file: String,
    pub line: usize,
    pub text: String,
}
impl BlobOp {
    pub fn new(mod_op: ModOp, parent: String, file: String, line: usize, text: String) -> BlobOp {
        BlobOp {
            mod_op,
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

    pub fn extract_change(&self) -> String {
        format!(
            "{} {}",
            self.mod_op.get_notation(),
            format!("{} {:?}", self.line, self.text)
        )
    }
}
