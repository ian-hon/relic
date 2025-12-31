use std::path::PathBuf;

mod core;

mod cli;
mod commands;

fn main() {
    let command_handler = cli::build();
    // let state = State::create(PathBuf::from("."));
    let args = command_handler.handler.clone().get_matches();

    // cli::handle(command_handler, args, state);
}
