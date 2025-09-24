use std::{collections::HashSet, fs, path::PathBuf};

use clap::ArgMatches;

use crate::{
    change::Change,
    models::content_set::{ContentSet, TrackingSet},
    paths::RELIC_PATH_TRACKED,
    state::State,
    utils,
};

const PENDING_TAG: &str = "LOCAL";

#[derive(Debug, Clone)]
pub struct Commit {
    pub id: Option<u32>,
    pub message: String,
    pub description: String,
    pub change: Change,
    pub timestamp: u64,

    pub author: String,
}
impl Commit {
    pub fn hash(&self) -> String {
        sha256::digest(self.serialise())
    }

    pub fn header(&self) -> String {
        // "integrated backwards compatibility" (2025-5-26 16:30) (affected : change.rs, content.rs, ...)

        let mut file_names = vec![];
        for (_, parent) in self.change.as_map().1 {
            for (f, _) in parent {
                file_names.push(f);
            }
        }

        format!(
            "({}) \"{}\" (affected : {}{})",
            utils::into_human_readable(self.timestamp),
            self.message,
            file_names
                .iter()
                .take(5)
                .map(|x| x.to_string())
                .collect::<Vec<String>>()
                .join(", "),
            if file_names.len() > 5 { ", ..." } else { "" }
        )
    }

    pub fn serialise(&self) -> String {
        format!(
            "= {} {} {:?} {:?} {}\n{}",
            self.id
                .map_or(PENDING_TAG.to_string(), |i| format!("{:06x}", i).clone()),
            self.timestamp,
            urlencoding::encode(&self.message).to_string(),
            urlencoding::encode(&self.description).to_string(),
            self.author,
            self.change.serialise_changes()
        )
    }

    pub fn deserialise(s: String) -> Option<Commit> {
        // = LOCAL 1747682692319414000 "initial%20commit" "" no_one

        let lines = s.split("\n").collect::<Vec<&str>>();
        if lines.len() < 2 {
            // return None;
        }

        let metadata = lines[0].split(" ").collect::<Vec<&str>>();
        if metadata.len() != 6 {
            // return None;
        }

        let [_, status, time, message, description, author] = *metadata.as_slice() else {
            return None;
        };

        Some(Commit {
            id: status.parse::<u32>().map_or(None, |t| Some(t)),
            message: urlencoding::decode(&message[1..message.len() - 1].to_string())
                .unwrap()
                .to_string(),
            description: urlencoding::decode(&description[1..description.len() - 1].to_string())
                .unwrap()
                .to_string(),
            change: Change::deserialise_changes(lines[1..].join("\n")).unwrap_or(Change::empty()),
            timestamp: time.parse::<u64>().unwrap_or(0),
            author: author.to_string(),
        })
    }
}

pub fn add(_: &mut State, args: &ArgMatches) {
    let f = args
        .get_many::<PathBuf>("FILE")
        .unwrap()
        .map(|x| x.clone())
        .collect::<Vec<PathBuf>>();

    let mut result: HashSet<String> = HashSet::from_iter(
        fs::read_to_string(format!("./{RELIC_PATH_TRACKED}"))
            .unwrap()
            .split("\n")
            .filter(|x| !x.is_empty())
            .map(|x| x.to_string())
            .collect::<Vec<String>>(),
    );
    for p in f {
        // TODO : path.join for this? or concatenating / works?
        result.insert(format!(
            "{}{}",
            p.to_string_lossy().to_string(),
            if !p.to_string_lossy().to_string().ends_with("/") && p.is_dir() {
                "/"
            } else {
                ""
            }
        ));
    }
    let _ = fs::write(
        format!("./{RELIC_PATH_TRACKED}"),
        result.drain().collect::<Vec<String>>().join("\n"),
    );
}

pub fn remove(s: &mut State, args: &ArgMatches) {
    let f = args
        .get_many::<PathBuf>("FILE")
        .unwrap()
        .map(|x| x.clone())
        .collect::<Vec<PathBuf>>();

    let result: HashSet<String> = HashSet::from_iter(
        fs::read_to_string(format!("./{RELIC_PATH_TRACKED}"))
            .unwrap()
            .split("\n")
            .filter(|x| !x.is_empty())
            .map(|x| PathBuf::from(".").join(x).to_string_lossy().to_string())
            .collect::<Vec<String>>(),
    );

    // initialise removed_content
    let mut removed_content = ContentSet {
        files: HashSet::from_iter(
            f.iter()
                .filter(|x| !x.is_dir())
                .map(|x| PathBuf::from(".").join(x).to_string_lossy().to_string()),
        ),
        directories: HashSet::from_iter(
            f.iter()
                .filter(|x| x.is_dir())
                .map(|x| PathBuf::from(".").join(x).to_string_lossy().to_string()),
        ),
    }
    .initialise(&mut s.current);

    let mut to_subtract: HashSet<String> = HashSet::from_iter(
        removed_content
            .directories
            .drain()
            .collect::<Vec<String>>()
            .into_iter()
            .map(|x| format!("{x}/"))
            .collect::<Vec<String>>(),
    );
    to_subtract = to_subtract
        .union(&HashSet::from_iter(removed_content.files.drain()))
        .map(|x| x.to_string())
        .collect::<HashSet<String>>();

    // set operations
    // right join
    // result - removed_content

    let _ = fs::write(
        format!("./{RELIC_PATH_TRACKED}"),
        result
            .difference(&to_subtract)
            .map(|x| x[2..].to_string())
            .collect::<Vec<String>>()
            .join("\n"),
    );
}

pub fn commit(state: &mut State, args: &ArgMatches) {
    // push into pending stage
    // update upstream

    // everything after the first line will be generated by Change::serialise_change
    r#"= {commit id} {unix timestamp of commit} {message} {description} {author}
+ D "lorem/ipsum/dolor"
+ F "lorem/ipsum/dolor/earth.txt" "earth.txt"
- D "lorem/sit"
=
| "lorem/ipsum/dolor/earth.txt"
+ 3 asdfsdf
+ 5 sfsdf
- 7
| "lorem/ipsum/saturn/txt"
+ 4 lsdfljs"#;
    let message = args.get_one::<String>("message").unwrap().clone();
    let description = args
        .get_one::<String>("description")
        .map_or("".to_string(), String::clone);

    let commit = Commit {
        id: None,
        message,
        description,
        change: state.get_changes(),
        timestamp: utils::get_time(),
        author: "no_one".to_string(),
    };

    state.pending_add(commit);
    // update upstream
    (*state).update_upstream(&mut state.track_set.clone());
}

pub fn push(_: &mut State, _: &ArgMatches) {}

pub fn pull(_: &mut State, _: &ArgMatches) {}

pub fn fetch(_: &mut State, _: &ArgMatches) {}

pub fn cherry(_: &mut State, _: &ArgMatches) {}

pub fn rollback(_: &mut State, _: &ArgMatches) {}

pub fn pending(state: &mut State, args: &ArgMatches) {
    let pending = state.pending_get();

    if let Some(commit_number) = args
        .get_one::<String>("COMMIT")
        .map_or(None, |x| x.parse::<i32>().map_or(None, |x| Some(x)))
    {
        // display selected
        if (commit_number < 0) || (commit_number >= pending.len() as i32) {
            println!(
                "Invalid selection. Please select commit numbers in the range of (0-{})",
                pending.len() - 1
            );
            return;
        }

        println!("{}", pending[commit_number as usize].serialise());
    } else {
        // display all
        for (index, c) in pending.iter().enumerate() {
            println!("{index}. {}", c.header());
        }
    }
}
