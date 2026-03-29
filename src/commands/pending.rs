use crate::core::state::State;
use clap::Args;

#[derive(Args)]
pub struct PendingArgs {
    /// Commit number.
    pub commit: Vec<String>,
}

pub fn pending(state: &mut State, _args: PendingArgs) {}
