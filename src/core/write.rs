use std::{
    collections::HashSet,
    ffi::OsString,
    fs,
    path::{Path, PathBuf},
};

use crate::core::{
    data::{commit::Commit, tree::Tree},
    error::{IOError, RelicError},
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
    let file_iter = match fs::read_dir(current_path) {
        Ok(i) => i,
        Err(_) => return Some(RelicError::IOError(IOError::DirectoryCantOpen)),
    };
    let mut files: HashSet<OsString> = HashSet::new();
    let mut directories: HashSet<OsString> = HashSet::new();
    for file in file_iter {
        if let Ok(file) = file {
            // TODO: handle by OsString instead of string
            let file_name = file.file_name();
            let file_type = file.file_type().unwrap();
            if file_type.is_dir() {
                directories.insert(file_name);
            } else if file_type.is_file() {
                files.insert(file_name);
            }
        }
    }

    let mut trees = vec![];
    let mut blobs = vec![];

    for children in &tree.children {
        // TODO: throw this recursive function into Tree instead of making it here
        match children.oid.construct(&sanctum_path) {
            Ok(object) => match object {
                Object::Tree(t) => {
                    let dir_path = current_path.join(children.name.clone());
                    let _ = fs::create_dir(&dir_path);
                    directories.remove(&OsString::from(&children.name));

                    trees.push((t, dir_path));

                    // if let Some(r) = write_tree(&dir_path, &sanctum_path, &t) {
                    //     return Some(r);
                    // }
                }
                Object::Blob(b) => {
                    let file_path = current_path.join(children.name.clone());
                    // files.remove(&(file_path.clone()).into_os_string());
                    files.remove(&OsString::from(&children.name));

                    blobs.push((b, file_path));
                    // if let Some(body) = b.get_body() {
                    //     let _ = fs::write(file_path, body);
                    // }
                }
                _ => unimplemented!(),
            },
            Err(e) => return Some(e),
        }
    }

    for file in files {
        // println!("remove file: {}", file.into_string().unwrap());
        fs::remove_file(Path::new(&file)).unwrap();
    }

    for directory in directories {
        // println!("remove directory: {}", directory.into_string().unwrap());
        fs::remove_dir_all(Path::new(&directory)).unwrap();
        // fs::remove_dir_all(path)
    }

    for (t, p) in trees {
        if let Some(r) = write_tree(&p, &sanctum_path, &t) {
            return Some(r);
        }
    }

    for (b, p) in blobs {
        if let Some(body) = b.get_body() {
            let _ = fs::write(p, body);
        }
    }

    None
}
