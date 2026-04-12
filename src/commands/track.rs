use std::path::PathBuf;

use crate::core::state::State;
use clap::Args;

#[derive(Args)]
pub struct TrackArgs {
    /// Paths to track (* for all)
    pub paths: Vec<PathBuf>,
}

pub fn track(state: &mut State, args: TrackArgs) {
    state.tracking_set.append(args.paths);
    state.update_tracking_set();
}
