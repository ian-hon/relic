use crate::core::state::State;
use clap::Args;

#[derive(Args)]
pub struct DetachArgs {}

pub fn detach(state: &mut State, _args: DetachArgs) {}
