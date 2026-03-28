use serde::{Deserialize, Serialize};

use crate::core::modification::change::ModOp;

#[derive(Debug, Clone, Serialize, Deserialize, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum TreeType {
    Tree,
    Blob,
}
impl TreeType {
    pub fn get_notation(&self) -> &str {
        match self {
            TreeType::Tree => "T",
            TreeType::Blob => "B",
        }
    }

    pub fn from_notation(n: &str) -> Option<TreeType> {
        match n {
            "T" | "D" => Some(TreeType::Tree),
            "B" | "F" => Some(TreeType::Blob),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct TreeOp {
    pub tree_type: TreeType,
    pub mod_op: ModOp,
    pub parent: String,
    pub name: String,
}
impl TreeOp {
    pub fn new(tree_type: TreeType, mod_op: ModOp, parent: String, name: String) -> TreeOp {
        TreeOp {
            tree_type,
            mod_op,
            parent,
            name,
        }
    }

    pub fn serialise(&self) -> String {
        format!(
            "{} {}",
            format!(
                "{} {}",
                self.mod_op.get_notation(),
                self.tree_type.get_notation()
            ),
            format!(
                "{} {}",
                urlencoding::encode(&self.parent).to_string(),
                urlencoding::encode(&self.name).to_string(),
            )
        )
    }
}
