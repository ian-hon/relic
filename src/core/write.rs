use std::{fs, path::PathBuf};

use crate::core::{
    data::{commit::Commit, tree::Tree},
    error::RelicError,
    object::Object,
};

pub fn write_commit(
    destination: &PathBuf, // destination to write to
    sanctum_path: &PathBuf,
    commit: &Commit, // Commit to write
) -> Option<RelicError> {
    let tree = match commit.tree.construct(&sanctum_path).unwrap() {
        Object::Tree(t) => t,
        _ => return Some(RelicError::ConfigurationIncorrect),
    };

    println!("STARTING: {}", tree.oid.to_string());

    write_tree(destination, sanctum_path, &tree)
}

fn write_tree(current_path: &PathBuf, sanctum_path: &PathBuf, tree: &Tree) -> Option<RelicError> {
    println!("started at {current_path:?}");
    for children in &tree.children {
        // TODO: throw this recursive function into Tree instead of making it here
        match children.oid.construct(&sanctum_path) {
            Ok(object) => match object {
                Object::Tree(t) => {
                    let dir_path = current_path.join(children.name.clone());
                    let _ = fs::create_dir(&dir_path);

                    if let Some(r) = write_tree(&dir_path, &sanctum_path, &t) {
                        return Some(r);
                    }
                }
                Object::Blob(b) => {
                    let file_path = current_path.join(children.name.clone());
                    if let Some(body) = b.get_body() {
                        let _ = fs::write(file_path, body);
                    }
                }
                _ => unimplemented!(),
            },
            Err(e) => return Some(e),
        }
    }

    None
}
