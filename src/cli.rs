use std::collections::HashMap;
use std::path::PathBuf;

use clap::{arg, value_parser, ArgMatches, Command};

use crate::commands as command_module;
use crate::core::error::RelicError;
use crate::core::state::State;

// add
// commit {message}
// push
// pull
// fetch
// branch {name}
//      will change to that branch
//      if branch doesnt exist, create
//      ask to create stash (if changes present)
// stash {name|optional}
//      stashes are bound to a branch
//      optional to have a name
// restore
//      select stash to restore
// rollback
//      resets to current head
// cherry {commit hash}

pub type CommandType = fn(&mut State, &ArgMatches);

pub struct CommandHandler {
    commands: HashMap<String, CommandType>,
    pub handler: Command,
}

pub fn build() -> CommandHandler {
    let mut command_handler = Command::new("relic")
        .about(
            r#"This is the Relic Version Control System.

The best way to learn is to stupidly and
blindly reinvent the wheel.

Relic is a simple hobby project, because
remaking Git sounded fun and interesting.

Most common features like committing,
pushing and pulling, are implemented."#,
        )
        .subcommand_required(true)
        .arg_required_else_help(true);

    type CommandType = fn(&mut State, &ArgMatches);
    let mut commands: HashMap<String, CommandType> = HashMap::new();
    for (f, c) in HashMap::<CommandType, clap::Command>::from_iter::<
        Vec<(CommandType, clap::Command)>,
    >(vec![
        (
            command_module::init,
            Command::new("init").about("Initialises a Relic repository in the current directory."),
        ),
        (
            command_module::clone,
            Command::new("clone").about("Clone a remote Relic repository in the current directory.")
            .arg_required_else_help(true)
            .arg(
                arg!([URL] "URL of the remote Relic repository")
                .required(true)
            )
        ),
        (
            command_module::detach,
            Command::new("detach").about("Completely removes Relic from the current directory.")
        ),
        (
            command_module::add,
            Command::new("add")
                .about("Adds a file(s) to staging")
                .arg_required_else_help(true)
                .arg(
                    arg!([FILE]... "File(s) to add (* for all)")
                        .required(true)
                        .value_parser(value_parser!(PathBuf)),
                ),
        ),
        (
            command_module::remove,
            Command::new("remove")
                .about("Removes a file(s) to staging")
                .arg_required_else_help(true)
                .arg(
                    arg!([FILE]... "File(s) to remove (* for all)")
                        .required(true)
                        .value_parser(value_parser!(PathBuf)),
                ),
        ),
        (
            command_module::commit,
            Command::new("commit")
                .about("Commit current changes.")
                .arg_required_else_help(true)
                .arg(arg!(-m --message <MESSAGE> "Commit message").required(true))
                .arg(arg!(-d --description <DESCRIPTION> "Commit description")),
        ),
        (
            command_module::push,
            Command::new("push").about("Pushes local changes to remote."),
        ),
        (
            command_module::pull,
            Command::new("pull").about("Pull changes from remote to local."),
        ),
        (
            command_module::fetch,
            Command::new("fetch").about("Check remote for new changes."),
        ),
        (
            command_module::branch,
            Command::new("branch").about("")
        ),
        (
            command_module::stash,
            Command::new("stash")
                // pseudo-commits basically
                // clear stash after a commit
                // stash create
                // stash view
                // stash restore
                // stash delete
                .about(""),
        ),
        (
            command_module::restore,
            Command::new("restore"), // unimplemented
        ),
        (
            command_module::rollback,
            Command::new("rollback").about("Discard all current changes. Rolls back to most recent commit (or pending commit)."),
        ),
        (
            command_module::cherry,
            Command::new("cherry").about("Go to specific commit."),
        ),
        (
            command_module::tree,
            Command::new("tree").about("Generate content tree of current directory."),
        ),
        (
            command_module::staging,
            Command::new("staging").about("View all staging changes."),
        ),
        (
            command_module::pending,
            Command::new("pending").about("View all pending commits.")
                .arg(arg!([COMMIT]... "Commit number."))
        ),
        (
            command_module::qhar,
            Command::new("qhar").about("??")
        ),
        (
            command_module::test,
            Command::new("test").about("this is here for debug purposes")
        )
    ]) {
        commands.insert(c.get_name().to_string(), f);
        command_handler = command_handler.subcommand(c);
    }

    CommandHandler {
        handler: command_handler,
        commands: commands,
    }
}

pub fn handle(command_handler: CommandHandler, args: ArgMatches, state: Result<State, RelicError>) {
    let (command_name, sub_matches) = args.subcommand().unwrap();

    // TODO : shorten and undry this
    if let Ok(mut s) = state {
        match command_name {
            "clone" | "init" => {
                // let this run only for
                // clone, init
                println!("Unable to '{command_name}' an already existing Relic repository.");
                return;
            }
            _ => match command_handler.commands.get(command_name) {
                Some(command) => {
                    command(&mut s, sub_matches);
                }
                None => {
                    unimplemented!("Relic Error, command not defined.");
                }
            },
        }
    } else {
        match command_name {
            "clone" | "init" => {
                // let this run only for
                // clone, init
                match command_handler.commands.get(command_name) {
                    Some(command) => {
                        command(&mut State::empty(), sub_matches);
                    }
                    None => {
                        unimplemented!("Relic Error, command not defined.");
                    }
                }
            }
            _ => {
                println!("No valid Relic repository found in current directory. Consider executing 'relic init' or 'relic clone'.");
                return;
            }
        }
    }
}
