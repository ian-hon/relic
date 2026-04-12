use std::path::PathBuf;
use std::str::FromStr;
use std::{fs, path::Path};

use crate::core::error::IOError;
use crate::core::object::{Object, ObjectLike, ObjectType};
use crate::core::oid::ObjectID;
use crate::core::state::State;
use crate::core::util::{empty_oid, oid_digest, oid_digest_data, string_to_oid};
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

#[derive(Debug, Clone)]
pub struct Tree {
    pub oid: ObjectID,
    pub children: Vec<TreeEntry>,
}

impl Tree {
    pub fn empty() -> Tree {
        // TODO: consider a less hacky solution
        // WILL NOT BE WRITTEN TO SANCTUM
        // USED ONLY FOR DIFFING
        let mut t = Tree {
            oid: empty_oid().into(),
            children: vec![],
        };

        t.oid = oid_digest_data(&t.serialise()).into();

        t
    }

    fn from_children(children: Vec<TreeEntry>, sanctum_path: &Path) -> Tree {
        let mut t = Tree {
            // oid: oid_digest(&Tree::string_from_children(&children)).into(),
            oid: empty_oid().into(),
            children,
        };

        t.oid = oid_digest_data(&t.serialise()).into();

        t.write(sanctum_path);

        t
    }

    pub fn deserialise(payload: Vec<u8>) -> Option<Tree> {
        // takes payload and deserialises into Option<Tree>
        let payload = str::from_utf8(&payload).unwrap();

        let mut children = vec![];

        // let lines = match payload.strip_prefix(HEADER) {
        //     Some(l) => l,
        //     None => return None,
        // };
        let lines = payload.strip_prefix(HEADER)?;
        let mut lines = lines.lines();

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
                oid: string_to_oid(oid)?.into(),
                name: file_name,
                otype: ObjectType::from_str(otype).ok()?,
            })
        }

        Some(Tree {
            oid: oid_digest(
                format!("{}{}", HEADER, &Tree::string_from_children(&children)).as_str(),
            )
            .into(),
            children,
        })
    }

    // walks this path and constructs a Tree object from it
    pub fn build_tree(
        state: &State,
        root_path: &Path,
        sanctum_path: &Path,
        relative_path: &Path,
    ) -> Result<Tree, RelicError> {
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

                    let relative =
                        Into::<String>::into(relative_path.join(&file_name).to_string_lossy());

                    if file_type.is_file() {
                        // logically speaking, only files in root will be tracked
                        // everything inside a directory will be tracked by that directory
                        if (relative_path == Path::new("."))
                            && (!state.tracking_set.files.contains(&relative))
                        {
                            continue;
                        }

                        // TODO: FIX
                        // its not as simple as this
                        // need to check if any file in ignore_set is a suffix of relative
                        // eg:
                        // ignore   : .DS_Store
                        // relative : /lorem/ipsum/.DS_Store
                        // since a file in ignore is a suffix, ignore this blob
                        // do the same for trees
                        //
                        // need to add a specifier for only root files too
                        // /.DS_Store to only ignore files at root, nowhere else
                        if state.ignore_set.files.contains(&relative) {
                            continue;
                        }

                        // TOOD: add fix for this
                        if file_name.eq(".DS_Store") {
                            continue;
                        }

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
                        if (relative_path == Path::new("."))
                            && (!state.tracking_set.directories.contains(&relative))
                        {
                            continue;
                        }

                        if state.ignore_set.directories.contains(&relative) {
                            continue;
                        }

                        if file_name.eq(".relic") {
                            continue;
                        }

                        match Tree::build_tree(
                            state,
                            &file_path,
                            sanctum_path,
                            &relative_path.join(&file_name),
                        ) {
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

    // pub fn iter_children(&self, sanctum_path: &Path) {
    //     self.children.iter().map(|x| x.oid.construct_strict(sanctum_path))
    // }

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

    fn as_payload(&self) -> Vec<u8> {
        // EXPENSIVE
        format!("{HEADER}{}", Self::string_from_children(&self.children))
            .as_bytes()
            .to_vec()
    }

    pub fn traverse<F>(
        &self,
        sanctum_path: &Path,
        current_path: PathBuf,
        func: &F,
        parent: &Tree,
    ) -> Option<RelicError>
    where
        // current path, parent tree, current content
        F: Fn(&PathBuf, &Tree, Object),
    {
        let c = self.clone();

        // EXPENSIVE!
        func(&current_path, &parent, Object::Tree(c));

        for (entry, child) in &mut self
            .children
            .iter()
            .map(|c| (c, c.oid.construct(sanctum_path)))
        {
            match child {
                Ok(o) => match o {
                    Object::Blob(b) => {
                        func(&current_path.join(&entry.name), self, Object::Blob(b));
                    }
                    Object::Tree(t) => {
                        if let Some(e) =
                            t.traverse(sanctum_path, current_path.join(&entry.name), func, self)
                        {
                            return Some(e);
                        }
                    }
                    Object::Commit(_) => return Some(RelicError::ConfigurationIncorrect),
                },
                Err(e) => return Some(e),
            }
        }
        None
    }
}

impl ObjectLike for Tree {
    const OBJECT_TYPE: ObjectType = ObjectType::Tree;

    fn get_oid(&self) -> ObjectID {
        self.oid
    }

    fn as_string(&self) -> String {
        // returns without header
        Tree::string_from_children(&self.children)
    }

    fn serialise(&self) -> Vec<u8> {
        // returns with header
        self.as_payload()
    }
}

#[derive(Debug, Clone)]
pub struct TreeEntry {
    pub oid: ObjectID,
    pub name: String, // use OsString instead?
    pub otype: ObjectType,
}
