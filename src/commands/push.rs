use crate::core::state::State;
use clap::Args;

#[derive(Args)]
pub struct PushArgs {}

pub fn push(state: &mut State, _args: PushArgs) {
}
