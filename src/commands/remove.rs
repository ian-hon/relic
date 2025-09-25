use std::{collections::HashSet, fs, path::PathBuf};

use clap::ArgMatches;

use crate::core::{
    content_set::{ContentSet, TrackingSet},
    paths::RELIC_PATH_TRACKED,
    state::State,
};

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
