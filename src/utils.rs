use std::time::{Duration, SystemTime, UNIX_EPOCH};

use chrono::{DateTime, Utc};

use crate::content::{Content, Directory};

pub fn generate_tree(dir: &Directory) -> String {
    return fetch_contents(&Content::Directory(dir.clone()));
}

fn fetch_contents(c: &Content) -> String {
    let mut result = vec![];

    match c {
        Content::Directory(d) => {
            let mut r = vec![d.name.clone()];
            if d.content.len() >= 1 {
                let length = d.content.len() - 1;
                for (index, i) in d.content.iter().enumerate() {
                    for (inner_index, line) in fetch_contents(i).split("\n").enumerate() {
                        r.push(format!(
                            " {} {line}",
                            if index == length {
                                if inner_index == 0 {
                                    "└"
                                } else {
                                    ""
                                }
                            } else {
                                if inner_index == 0 {
                                    "├"
                                } else {
                                    "│"
                                }
                            }
                        ));
                    }
                }
            }
            result.push(r.join("\n"));
        }
        Content::File(f) => {
            result.push(f.name.clone());
            // result.push(format!("{} ({})", f.name, sha256::digest(&f.content)));
        }
    }

    result.join("\n")
}

pub fn get_time() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time went backwards (???)")
        .as_millis() as u64
}

pub fn into_human_readable(t: u64) -> String {
    // accepts unix time, but only in milliseconds format
    DateTime::<Utc>::from(UNIX_EPOCH + Duration::from_millis(t as u64))
        .format("%Y-%m-%d %H:%M:%S")
        .to_string()
}
