use std::path::PathBuf;

use crate::core::state::State;
use clap::Args;

#[derive(Args)]
pub struct UntrackArgs {
    /// Paths to untrack (* for all)
    pub paths: Vec<PathBuf>,
}

pub fn untrack(state: &mut State, args: UntrackArgs) {
    state.tracking_set.remove(args.paths);
    state.update_tracking_set();
}
