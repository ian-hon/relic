use std::time::{SystemTime, UNIX_EPOCH};

use crate::{content::Content, state::State};

pub fn get_value(args: &Vec<String>, key: &str) -> Option<String> {
    for (index, i) in args.iter().enumerate() {
        if i.starts_with("-") && (i[1..] == key[..]) {

        }
    }

    None
}

pub fn generate_tree(state: &State) -> String {
    return fetch_contents(&Content::Directory(state.current.clone()));
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
                        r.push(format!(" {} {line}", if index == length { if inner_index == 0 { "└" } else { "" } } else { if inner_index == 0 { "├" } else { "│" } }));
                    }
                }
            }
            result.push(r.join("\n"));
        },
        Content::File(f) => {
            result.push(f.name.clone());
        }
    }

    result.join("\n")
}

pub fn get_time() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time went backwards (???)")
        .as_secs() as u128
}
