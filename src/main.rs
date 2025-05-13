use std::fs;
use std::path::Path;
use std::{collections::HashMap, env};

mod error;
mod ignore;
mod state;
mod utils;

mod relic;
mod commit;
mod branch;
mod stash;

mod content;
mod change;

use clap::{arg, ArgMatches, Command};
use relic::Relic;
use change::Change;
use ignore::IgnoreSet;
use utils::generate_tree;

use crate::commit::{add, commit, push, pull, fetch, cherry, rollback};
use crate::branch::branch;
use crate::stash::{stash, restore};
use crate::state::State;

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

pub fn init(_: State, _: &ArgMatches) {

}

fn main() {
    // #region commands
    // TODO : automate this
    let command_handler = Command::new("relic")
        .about(r#"This is the Relic Version Control System.

The best way to learn is to stupidly and
blindly reinvent the wheel.

Relic is a simple hobby project, because
remaking Git sounded fun and interesting.

Most common features like committing,
pushing and pulling, are implemented."#)
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("commit")
                .about("Commit current changes.")
                .arg_required_else_help(true)
                .arg(arg!(-m --message <MESSAGE> "Commit message").required(true))
                .arg(arg!(-d --description <DESCRIPTION> "Commit description"))
        )
        .subcommand(
            Command::new("init")
                .about("Initialises a Relic repository in the current directory.")
        )
        .subcommand(
            Command::new("push")
                .about("Pushes local changes to remote.")
        )
        .subcommand(
            Command::new("pull")
                .about("Pull changes from remote to local.")
        )
        .subcommand(
            Command::new("fetch")
                .about("Check remote for new changes.")
        )
        .subcommand(
            Command::new("branch")
                .about("")
        )
        .subcommand(
            Command::new("stash")
                // pseudo-commits basically
                // clear stash after a commit
                // stash create
                // stash view
                // stash restore
                // stash delete
                .about("")
        )
        .subcommand(
            Command::new("rollback")
                .about("Discard all current changes. Rolls back to most recent commit (or pending commit).")
        )
        .subcommand(
            Command::new("cherry")
                .about("Go to specific commit.")
        )
        .subcommand(
            Command::new("tree")
                .about("Generate content tree of current directory.")
        )


        .subcommand(
            Command::new("pending")
                .about("DEBUG : view all pending changes")
        )
    ;

    type CommandType = fn(State, &ArgMatches);
    let commands: HashMap<String, CommandType> = HashMap::from_iter::<Vec<(String, CommandType)>>(vec![
        ("commit".to_string(), commit),
        ("push".to_string(), push),
        ("pull".to_string(), pull),
        ("fetch".to_string(), fetch),
        ("branch".to_string(), branch),
        ("stash".to_string(), stash),
        ("restore".to_string(), restore),
        ("rollback".to_string(), rollback),
        ("cherry".to_string(), cherry),

        ("tree".to_string(), |s, _| {
            println!("{}", generate_tree(&s));
        }),

        ("init".to_string(), init),

        ("pending".to_string(), |s, _| {
            println!("{}", s.get_changes().serialise_changes())
        })
    ]);
    // #endregion
    
    // get current path
    // (used to be more complicated than this, but keeping it as a relative path just makes more sense now)
    let path = Path::new(".");
    //

    // get ignorance set
    let ignore_set = IgnoreSet::create(fs::read_to_string(path.join(".relic_ignore")).unwrap_or("".to_string()));
    //

    match State::create(".".to_string(), &ignore_set) {
        Ok(s) => {
            let c = command_handler.get_matches();
            let (command_name, sub_matches) = c.subcommand().unwrap();
            match commands.get(command_name) {
                Some(command) => {
                    command(s, sub_matches);
                },
                None => {
                    println!("Relic Error, command not defined.")
                }
            }
        },
        Err(e) => {
            println!("{e:?} error encountered.");
        }
    }
}
