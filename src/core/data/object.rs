use std::{fs, path::Path};

use crate::core::data::{blob::Blob, oid::ObjectID, tree::Tree};

use strum_macros::{Display, EnumString, IntoStaticStr};

#[derive(Debug, Clone, Copy, Display, EnumString, IntoStaticStr)]
pub enum ObjectType {
    #[strum(serialize = "T")]
    Tree,

    #[strum(serialize = "B")]
    Blob,
}
// impl ToString for ObjectType {
//     fn to_string(&self) -> String {
//         match self {
//             ObjectType::Blob => "B".to_string(),
//             ObjectType::Tree => "T".to_string(),
//             _ => unimplemented!(),
//         }
//     }
// }

// Holds either a Blob or Tree
pub enum Object {
    Blob(Blob),
    Tree(Tree),
}

pub trait ObjectLike {
    fn get_oid(&self) -> ObjectID;
    fn as_string(&self) -> String;
    fn write(&self, sanctum_path: &Path) {
        let (prefix_path, suffix_path) = self.get_oid().get_paths(sanctum_path);

        // check if prefix exists
        if !prefix_path.exists() {
            fs::create_dir(prefix_path).expect("create_dir err");
        }

        let _ = fs::write(suffix_path, self.as_string());
    }
}
