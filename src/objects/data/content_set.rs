use std::{
    collections::HashSet,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use serde::{Deserialize, Serialize};

use crate::objects::data::{directory::Directory, ContentMutRef};

pub const DEFAULT_IGNORE: &str = r#"-- Added by Relic: Automatically ignore all git content
.git/
.gitignore"#;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ContentSet {
    pub directories: HashSet<String>,
    pub files: HashSet<String>,
}
impl ContentSet {
    pub fn empty() -> ContentSet {
        ContentSet {
            directories: HashSet::new(),
            files: HashSet::new(),
        }
    }

    pub fn as_set(&self) -> HashSet<String> {
        HashSet::new()
    }
}

pub trait IgnoreSet {
    fn create(content: String) -> Self;
}
impl IgnoreSet for ContentSet {
    fn create(content: String) -> ContentSet {
        let mut result = ContentSet {
            directories: HashSet::new(),
            files: HashSet::new(),
        };

        // always ignore the .relic directory
        result.directories.insert(".relic".to_string());

        for line in content.split("\n") {
            if line.is_empty() {
                continue;
            }

            // skip comments
            if line.starts_with("-- ") {
                continue;
            }

            // doesnt take into account cases like
            // some_directory// <- double slashes
            if line.ends_with("/") {
                let i = line[0..line.len() - 1].to_string();
                if i.is_empty() {
                    continue;
                }

                result.directories.insert(i);
            } else {
                result.files.insert(line.to_string());
            }
        }

        result
    }
}

pub trait TrackingSet {
    fn deserialise(content: String) -> Self;
    fn initialise(&self, d: &mut Directory) -> Self;
}
impl TrackingSet for ContentSet {
    fn deserialise(content: String) -> Self {
        let mut result = ContentSet::empty();

        for d in content
            .split("\n")
            .map(|x| x.to_string())
            .collect::<Vec<String>>()
        {
            if d.ends_with("/") {
                // dir
                result.directories.insert(d[..d.len() - 1].to_string());
            } else {
                // file
                result.files.insert(d);
            }
        }

        result
    }

    fn initialise(&self, d: &mut Directory) -> ContentSet {
        let tracked_mutex = Arc::new(Mutex::new(self.clone()));
        d.traverse(
            PathBuf::from("."),
            &|path, _, current| {
                // println!("traversing at : {path:?}");

                let mut tracked_unlock = tracked_mutex.lock().unwrap();

                match current {
                    ContentMutRef::Directory(d) => {
                        // if parent in set
                        // add to content set
                        if tracked_unlock
                            .directories
                            .contains(&d.path.parent().unwrap().to_string_lossy().to_string())
                        {
                            tracked_unlock
                                .directories
                                .insert(d.path.to_string_lossy().to_string());
                        }
                    }
                    ContentMutRef::File(f) => {
                        if tracked_unlock
                            .directories
                            .contains(&path.to_string_lossy().to_string())
                        {
                            tracked_unlock
                                .files
                                .insert(path.join(&f.name).to_string_lossy().to_string());
                        }
                    }
                }
            },
            &d.clone(),
        );

        // dont ask me
        let result = tracked_mutex.lock().unwrap().clone();
        result
    }
}
