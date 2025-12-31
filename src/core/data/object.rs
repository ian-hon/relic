use crate::core::data::{blob::Blob, oid::ObjectID, tree::Tree};

pub enum ObjectType {
    Tree,
    Blob,
}
impl ToString for ObjectType {
    fn to_string(&self) -> String {
        match self {
            ObjectType::Blob => "B".to_string(),
            ObjectType::Tree => "T".to_string(),
            _ => unimplemented!(),
        }
    }
}

// Holds either a Blob or Tree
pub enum ObjectKind {
    Blob(Blob),
    Tree(Tree),
}

pub trait Object {
    fn get_oid(&self) -> ObjectID;
    fn as_string(&self) -> String;
}
