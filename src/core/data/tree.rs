/*
Order by type (trees first) then filename (a-z)

Tree format:
T {byte length}\0
T {tree_name} hash
T {tree_name} hash
T {tree_name} hash
B {blob_name} hash
B {blob_name} hash
*/

use std::{fs, path::Path};

use crate::core::{
    data::{
        blob::Blob,
        object::{Object, ObjectType},
        oid::ObjectID,
        util::oid_digest,
    },
    error::RelicError,
};

#[derive(Debug)]
pub struct Tree {
    pub children: Vec<TreeEntry>,
    oid: ObjectID,
}

impl Tree {
    pub fn new() -> Tree {
        Tree {
            children: vec![],
            oid: ObjectID::empty(),
        }
    }

    fn from_children(children: Vec<TreeEntry>) -> Tree {
        Tree {
            oid: oid_digest(&Tree::string_from_children(&children)).into(),
            children,
        }
    }

    // walks this path and constructs a Tree object from it
    pub fn build_tree(root_path: &Path, sanctum_path: &Path) -> Result<Tree, RelicError> {
        let paths = match fs::read_dir(root_path) {
            Ok(r) => r,
            Err(e) => {
                println!("state.rs (content_at) get all dirs : {root_path:?} : {e:?}");
                return Err(RelicError::FileCantOpen);
            }
        };

        let mut children: Vec<TreeEntry> = vec![];

        // iterate through them all
        for path in paths {
            match path {
                Ok(p) => {
                    let file_type = p.file_type().unwrap();
                    let file_name = p.file_name().into_string().unwrap();
                    let file_path = p.path();

                    println!("{file_path:?}");

                    if file_type.is_file() {
                        match Blob::load_from_path(&file_path) {
                            Ok(b) => {
                                children.push(TreeEntry {
                                    oid: b.get_oid(),
                                    name: file_name,
                                    otype: ObjectType::Blob,
                                });
                            }
                            Err(e) => return Err(e),
                        }
                    } else if file_type.is_dir() {
                        if file_name.eq("target") {
                            continue;
                        }

                        if file_name.eq(".git") {
                            continue;
                        }

                        match Tree::build_tree(&file_path, sanctum_path) {
                            Ok(t) => children.push(TreeEntry {
                                oid: t.get_oid(),
                                name: file_name,
                                otype: ObjectType::Tree,
                            }),
                            Err(e) => return Err(e),
                        };
                    } else {
                        // symlink
                    }
                }
                Err(e) => {
                    println!("state.rs (content_at) read_dir : {e:?}");
                }
            }
        }

        Ok(Tree::from_children(children))
    }

    fn string_from_children(children: &Vec<TreeEntry>) -> String {
        children
            .iter()
            .map(|c| format!("{} {} {}", c.otype.to_string(), c.name, c.oid.to_string()))
            .fold(String::new(), |mut left, right| {
                left.reserve(right.len() + 1);
                left.push_str(&right);
                left.push_str("\n");
                left
            })
            .trim_end()
            .to_string() // EXPENSIVE!
    }
}

impl Object for Tree {
    fn get_oid(&self) -> ObjectID {
        self.oid
    }

    fn as_string(&self) -> String {
        Tree::string_from_children(&self.children)
    }
}

#[derive(Debug)]
pub struct TreeEntry {
    oid: ObjectID,
    name: String,
    otype: ObjectType,
}
