use std::fs;

use clap::ArgMatches;

use crate::core::{content_set, paths, state, State};

pub fn init(_: &mut State, _: &ArgMatches) {
    // create
    // .relic
    //      history/ (empty)
    //      pending/ (empty)
    //      root (empty)
    //      tracked (empty)
    //      upstream (empty)
    // .relic_ignore (use default (const in content_set))

    // if origin is set
    // update root
    // update upstream

    let _ = fs::create_dir(paths::RELIC_PATH_PARENT);
    let _ = fs::create_dir(paths::RELIC_PATH_HISTORY);
    let _ = fs::create_dir(paths::RELIC_PATH_PENDING);
    let _ = fs::write(paths::RELIC_PATH_INFO, state::DEFAULT_INFO);
    let _ = fs::write(paths::RELIC_PATH_ROOT, "");
    let _ = fs::write(paths::RELIC_PATH_TRACKED, "");
    let _ = fs::write(paths::RELIC_PATH_UPSTREAM, state::DEFAULT_UPSTREAM);

    let _ = fs::write(paths::RELIC_PATH_IGNORE, content_set::DEFAULT_IGNORE);

    println!("Empty Relic repository created.");
}
