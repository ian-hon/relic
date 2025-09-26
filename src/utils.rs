use std::time::{Duration, SystemTime, UNIX_EPOCH};

use chrono::{DateTime, Utc};

use crate::core::{Content, Tree};

pub fn generate_tree(tree: &Tree) -> String {
    return generate_subtree(&Content::Tree(tree.clone()));
}

fn generate_subtree(c: &Content) -> String {
    let mut result = vec![];

    match c {
        Content::Tree(t) => {
            let mut r = vec![t.name.clone()];
            if t.content.len() >= 1 {
                let length = t.content.len() - 1;
                for (index, i) in t.content.iter().enumerate() {
                    for (inner_index, line) in generate_subtree(i).split("\n").enumerate() {
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
        Content::Blob(b) => {
            result.push(b.name.clone());
            // result.push(format!("{} ({})", b.name, sha256::digest(&b.content)));
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
