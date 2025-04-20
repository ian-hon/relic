use std::fs;
use std::{collections::HashMap, env};

mod error;
mod ignore;
mod state;
mod utils;

mod commit;
mod branch;
mod stash;

mod content;
mod bones;

use ignore::IgnoreSet;

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

pub fn init(_: String, _: Vec<String>) {

}


pub fn help(_: State, _: Vec<String>) {
    println!(r#"This is the Barebones Version Control System."#);
}

pub fn about(_: State, _: Vec<String>) {
    println!(r#"This is the Barebones Version Control System.

The best way to learn is to stupidly and
blindly reinvent the wheel.

Barebones VCS (or just Bones) is a simple
hobby project, because remaking Git sounded
fun and interesting.

Most common features like adding,
committing, pushing and pulling, are
implemented."#);
}

fn main() {
    // build all commands
    type CommandType = fn(State, Vec<String>);
    let commands: HashMap<String, CommandType> = HashMap::from_iter::<Vec<(String, CommandType)>>(vec![
        ("add".to_string(), add),
        ("commit".to_string(), commit),
        ("push".to_string(), push),
        ("pull".to_string(), pull),
        ("fetch".to_string(), fetch),
        ("branch".to_string(), branch),
        ("stash".to_string(), stash),
        ("restore".to_string(), restore),
        ("rollback".to_string(), rollback),
        ("cherry".to_string(), cherry),
        ("help".to_string(), help),
        ("about".to_string(), about)
    ]);
    //
    
    // collect all arguments
    let arguments = env::args().collect::<Vec<String>>();
    if arguments.len() <= 1 {
        help(State::empty(), vec![]);
        return;
    }
    //

    // get current path
    let path = std::env::current_dir();
    if path.is_err() {
        println!("Can't get current path.");
        return;
    }
    let path = path.unwrap();
    //

    //
    let ignore_set = IgnoreSet::create(fs::read_to_string(path.join(".bones_ignore")).unwrap_or("".to_string()));
    //

    match State::create(path.to_str().unwrap().to_string(), &ignore_set) {
        Ok(s) => {
            match commands.get(&arguments[1]) {
                Some(c) => {
                    c(s, arguments[2..arguments.len()].to_vec());
                },
                None => {
                    println!("Command not found.");
                    help(State::empty(), vec![]);
                }
            }
        },
        Err(e) => {
            println!("{e:?} error encountered.")
        }
    }
}
