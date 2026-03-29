use crate::core::state::State;
use clap::Args;

#[derive(Args)]
pub struct PullArgs {}

pub fn pull(state: &mut State, _args: PullArgs) {
}
