use std::path::PathBuf;

mod core;

mod error;
mod utils;

mod cli;
mod commands;

use crate::core::state::State;

fn main() {
    let command_handler = cli::build();
    let state = State::create(PathBuf::from("."));
    let args = command_handler.handler.clone().get_matches();

    cli::handle(command_handler, args, state);
}
