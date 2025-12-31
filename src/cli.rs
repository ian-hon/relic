use std::collections::HashMap;

use clap::Command;

// pub type CommandType = fn(&mut State, &ArgMatches);
pub type CommandType = fn();

pub struct CommandHandler {
    commands: HashMap<String, CommandType>,
    pub handler: Command,
}

pub fn build() -> CommandHandler {
    CommandHandler {
        commands: HashMap::new(),
        handler: Command::new("Relic"),
    }
}

// pub fn handle(command_handler: CommandHandler, args: ArgMatches, state: Result<State, RelicError>) {
pub fn handle() {}
