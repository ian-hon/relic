use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use crate::commands as command_module;
use clap::{arg, value_parser, ArgMatches, Command};

// pub type CommandType = fn(&mut State, &ArgMatches);
pub type CommandType = fn(&Path, &ArgMatches);

pub struct CommandHandler {
    commands: HashMap<String, CommandType>,
    pub handler: Command,
}

pub fn build() -> CommandHandler {
    let mut command_handler = Command::new("Relic")
        .about(
            r#"This is the Relic Version Control System.

Relic is a CAS + Merkle DAG (Directed
Acyclic Graph) using SHA256. Objects are
stored inside ./.relic/sanctum/.

I wanted to truly understand how Git
works, so I made Relic. Everyone knows
the best way to learn is to stupidly and
naively reinvent the wheel."#,
        )
        .subcommand_required(true)
        .arg_required_else_help(true);

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
            Command::new("clone")
                .about("Clone a remote Relic repository in the current directory.")
                .arg_required_else_help(true)
                .arg(arg!([URL] "URL of the remote Relic repository").required(true)),
        ),
        (
            command_module::detach,
            Command::new("detach").about("Completely removes Relic from the current directory."),
        ),
        (
            command_module::track,
            Command::new("track")
                .about("Adds file(s) to be tracked")
                .arg_required_else_help(true)
                .arg(
                    arg!([FILE]... "File(s) to track (* for all)")
                        .required(true)
                        .value_parser(value_parser!(PathBuf)),
                ),
        ),
        (
            command_module::untrack,
            Command::new("untrack")
                .about("Removes file(s) from being tracked")
                .arg_required_else_help(true)
                .arg(
                    arg!([FILE]... "File(s) to untrack (* for all)")
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
            Command::new("push").about("Pushes pending commits to remote."),
        ),
        (
            command_module::pull,
            Command::new("pull").about("Pull pending commits from remote to local."),
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
            Command::new("pending")
                .about("View all pending commits.")
                .arg(arg!([COMMIT]... "Commit number.")),
        ),
        (
            command_module::status,
            Command::new("status").about("View status of staging, pending and remote."),
        ),
        (command_module::qhar, Command::new("qhar").about("??")),
        (
            command_module::test,
            Command::new("test").about("this is here for debug purposes"),
        ),
    ]) {
        commands.insert(c.get_name().to_string(), f);
        command_handler = command_handler.subcommand(c);
    }

    CommandHandler {
        commands,
        handler: command_handler,
    }
}

pub fn handle(command_handler: CommandHandler, args: ArgMatches, relic_path: Option<&Path>) {
    let (command_name, sub_matches) = args.subcommand().unwrap();

    // TODO : shorten and undry this
    if let Some(r) = relic_path {
        match command_name {
            "clone" | "init" => {
                // let this run only for
                // clone, init
                println!("Unable to '{command_name}' an already existing Relic repository.");
                return;
            }
            _ => match command_handler.commands.get(command_name) {
                Some(command) => {
                    command(r, sub_matches);
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
                        command(Path::new("."), sub_matches);
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
