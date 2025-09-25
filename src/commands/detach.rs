use std::fs;

use clap::ArgMatches;

use crate::core::{paths, State};

pub fn detach(_: &mut State, _: &ArgMatches) {
    let _ = fs::remove_dir_all(paths::RELIC_PATH_PARENT);
    let _ = fs::remove_file(paths::RELIC_PATH_IGNORE);

    println!("Relic repository successfully removed.");
}
