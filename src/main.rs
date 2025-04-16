use std::{collections::HashMap, env};

mod commit;
mod branch;
mod stash;

use crate::commit::{add, commit, push, pull, fetch, cherry, rollback};
use crate::branch::{branch};
use crate::stash::{stash, restore};

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


pub fn help(_: String, _: Vec<String>) {
    println!(r#"This is the Barebones Version Control System."#);
}

pub fn about(_: String, _: Vec<String>) {
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
    let commands: HashMap<String, fn(String, Vec<String>)> = HashMap::from_iter::<Vec<(String, fn(String, Vec<String>))>>(vec![
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
        ("help".to_string(), help)
    ]);

    let arguments = env::args().collect::<Vec<String>>();
    if arguments.len() <= 1 {
        help("".to_string(), vec![]);
        return;
    }

    match commands.get(&arguments[1]) {
        Some(c) => {
            c(arguments[0].clone(), arguments[2..arguments.len()].to_vec());
        },
        None => {
            println!("Command not found.");
            help("".to_string(), vec![]);
        }
    }
}
