use std::str::FromStr;
use std::{fs, path::Path};

use crate::core::error::IOError;
use crate::core::object::{ObjectLike, ObjectType};
use crate::core::oid::ObjectID;
use crate::core::util::{empty_oid, oid_digest, string_to_oid};
use crate::core::{data::blob::Blob, error::RelicError};

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

const HEADER: &str = "T\0";

#[derive(Debug)]
pub struct Tree {
    oid: ObjectID,
    pub children: Vec<TreeEntry>,
}

impl Tree {
    fn from_children(children: Vec<TreeEntry>, sanctum_path: &Path) -> Tree {
        let mut t = Tree {
            // oid: oid_digest(&Tree::string_from_children(&children)).into(),
            oid: empty_oid().into(),
            children,
        };

        // t.oid = t.as_payload()
        // TODO: test
        t.oid = oid_digest(&t.serialise()).into();

        t.write(sanctum_path);

        t
    }

    pub fn deserialise(payload: Vec<u8>) -> Option<Tree> {
        // takes payload and deserialises into Option<Tree>
        let payload = str::from_utf8(&payload).unwrap();

        let mut children = vec![];

        let mut lines = payload.lines();
        lines.next(); // skip the header

        while let Some(line) = lines.next() {
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
                return Err(RelicError::IOError(IOError::FileCantOpen));
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
                        match Blob::build_blob(&file_path, sanctum_path) {
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

    // pub fn construct_children() ->

    fn string_from_children(children: &Vec<TreeEntry>) -> String {
        // format:
        // T abcdef12345 tree_name
        // T abcdef12345 tree_name
        // B abcdef12345 blob_name
        // T abcdef12345 tree_name
        // B abcdef12345 blob_name
        children
            .iter()
            .map(|c| format!("{} {} {}", c.otype.to_string(), c.oid.to_string(), c.name))
            .fold(String::new(), |mut left, right| {
                left.reserve(right.len() + 1);
                left.push_str(&right);
                left.push_str("\n");
                left
            })
            .trim_end() // remove the singular trailing \n
            .to_string() // EXPENSIVE!
    }

    fn as_payload(&self) -> String {
        format!("{HEADER}{}", Self::string_from_children(&self.children))
    }
}

impl ObjectLike for Tree {
    fn get_oid(&self) -> ObjectID {
        self.oid
    }

    fn as_string(&self) -> String {
        // returns without header
        Tree::string_from_children(&self.children)
    }

    fn serialise(&self) -> String {
        // returns with header
        self.as_payload()
    }
}

#[derive(Debug)]
pub struct TreeEntry {
    oid: ObjectID,
    name: String, // use OsString instead?
    otype: ObjectType,
}
