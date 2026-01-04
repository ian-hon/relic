use std::path::PathBuf;

use crate::core::state::State;
use clap::ArgMatches;

pub fn track(state: Option<&mut State>, args: &ArgMatches) {
    let Some(state) = state else { return };

    let paths = args
        .get_many::<PathBuf>("PATHS")
        .unwrap()
        .map(|x| x.clone())
        .collect::<Vec<PathBuf>>();

    state.tracking_set.append(paths);
    state.update_tracking_set();
}
