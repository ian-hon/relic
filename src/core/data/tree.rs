/*
Order by type (trees first) then filename (a-z)

Tree format:
T\0
T hash {tree_name}
T hash {tree_name}
T hash {tree_name}
B hash {blob_name}
B hash {blob_name}
*/

use std::str::FromStr;
use std::{fs, path::Path};

use crate::core::{
    data::{
        blob::Blob,
        object::{ObjectLike, ObjectType},
        oid::ObjectID,
        util::{oid_digest, string_to_oid},
    },
    error::RelicError,
};

#[derive(Debug)]
pub struct Tree {
    pub children: Vec<TreeEntry>,
    pub oid: ObjectID,
}

impl Tree {
    fn from_children(children: Vec<TreeEntry>, sanctum_path: &Path) -> Tree {
        let t = Tree {
            oid: oid_digest(&Tree::string_from_children(&children)).into(),
            children,
        };

        t.write(sanctum_path);

        t
    }

    pub fn from_string(content: &str) -> Option<Tree> {
        let mut children = vec![];

        for line in content.lines() {
            let mut l = line.split(" ");
            let otype = l.next()?;
            let oid = l.next()?;
            let file_name = l.collect::<Vec<&str>>();

            if file_name.is_empty() {
                return None;
            }
            let file_name = file_name.join(" ");

            children.push(TreeEntry {
                oid: string_to_oid(oid).into(),
                name: file_name,
                otype: ObjectType::from_str(otype).ok()?,
            })
        }

        Some(Tree {
            oid: oid_digest(&Tree::string_from_children(&children)).into(),
            children,
        })
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
                        match Blob::load_from_path(&file_path, sanctum_path) {
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

                        if file_name.eq(".relic") {
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

        Ok(Tree::from_children(children, sanctum_path))
    }

    fn string_from_children(children: &Vec<TreeEntry>) -> String {
        children
            .iter()
            .map(|c| format!("{} {} {}", c.otype.to_string(), c.oid.to_string(), c.name))
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

impl ObjectLike for Tree {
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
    name: String, // use OsString instead?
    otype: ObjectType,
}
