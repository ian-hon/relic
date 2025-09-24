use std::{collections::HashSet, fs, path::PathBuf};

use clap::ArgMatches;

use crate::core::{paths::RELIC_PATH_TRACKED, state::State};

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
