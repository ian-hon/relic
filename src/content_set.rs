use std::collections::HashSet;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct ContentSet {
    pub directories: HashSet<String>,
    pub files: HashSet<String>
}
impl ContentSet {
    pub fn empty() -> ContentSet {
        ContentSet {
            directories: HashSet::new(),
            files: HashSet::new()
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
            files: HashSet::new()
        };

        // always ignore the .relic directory
        result.directories.insert(".relic".to_string());

        for line in content.split("\n") {
            if line.is_empty() {
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
}
impl TrackingSet for ContentSet {
    fn deserialise(content: String) -> Self {
        let mut result = ContentSet::empty();

        for d in content.split("\n").map(|x| x.to_string()).collect::<Vec<String>>() {
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
}