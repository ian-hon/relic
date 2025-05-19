use std::fs;
use std::path::{Path, PathBuf};
use std::{collections::HashMap, env};

mod error;
mod content_set;
mod state;
mod utils;

mod relic;
mod commit;
mod branch;
mod stash;

mod content;
mod change;

use clap::{arg, value_parser, ArgMatches, Command};
use commit::remove;
use content::Content;
use relic::Relic;
use change::{Change, Modification};
use content_set::{ContentSet, IgnoreSet, TrackingSet};
use similar::{ChangeTag, TextDiff};
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

pub fn init(_: &mut State, _: &ArgMatches) {

}

fn main() {
    // issue with similar diffing library
    // a\nb\nc diff a\nx\ny\nz\n

    // let a = "a\nb\nc".to_string();
    // let b = "a\nb\n\nz\n".to_string();
    // let diff = TextDiff::from_lines(&a, &b);
    // for change in diff
    //     .iter_all_changes()
    //     .filter_map(|c| match c.tag() {
    //         ChangeTag::Equal => None,
    //         _ => Some(c)
    //     }
    // ) {
    //     println!("{:?}",
    //         match change.tag() {
    //             ChangeTag::Delete => Modification::Delete(
    //                 "".to_string(),
    //                 "".to_string(),
    //                 change.old_index().unwrap()
    //             ),
    //             ChangeTag::Insert => Modification::Create(
    //                 "".to_string(),
    //                 "".to_string(),
    //                 change.new_index().unwrap(),
    //                 // change.to_string()
    //                 change.to_string().strip_suffix("\n").unwrap().to_string()
    //             ),
    //             _ => panic!()
    //         }
    //     );
    // }
    // return;

    // let _ = fs::write(
    //     ".relic/upstream",
    //     match State::content_at(
    //         &"".to_string(),
    //         &PathBuf::from("."),
    //         &IgnoreSet::create("target/".to_string())
    //     ).unwrap() {
    //         content::Content::Directory(d) => {
    //             d.serialise()
    //         },
    //         _ => panic!()
    //     }
    // );
    // return;

    // let mut f = content::Directory {
    //     path: PathBuf::from("."),
    //     name: "".to_string(),
    //     content: vec![
    //         content::Content::Directory(content::Directory {
    //             path: PathBuf::from("."),
    //             name: "huh".to_string(),
    //             content: vec![]
    //         })
    //     ]
    // };

    // println!("{}", utils::generate_tree(&f));

    // f.apply_changes(Change {
    //     container_modifications: vec![
    //         change::ContainerModification::CreateDirectory(".".to_string(), "lorem".to_string())
    //     ],
    //     modifications: vec![]
    // });

    // println!("{}", utils::generate_tree(&f));

    // return;

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
            Command::new("init")
                .about("Initialises a Relic repository in the current directory.")
        )

        .subcommand(
            Command::new("add")
                .about("Adds a file(s) to staging")
                .arg_required_else_help(true)
                .arg(
                    arg!([FILE]... "File(s) to add (* for all)")
                    .required(true)
                    .value_parser(value_parser!(PathBuf))
                )
        )
        .subcommand(
            Command::new("remove")
                .about("Removes a file(s) to staging")
                .arg_required_else_help(true)
                .arg(
                    arg!([FILE]... "File(s) to remove (* for all)")
                    .required(true)
                    .value_parser(value_parser!(PathBuf))
                )
        )
        .subcommand(
            Command::new("commit")
                .about("Commit current changes.")
                .arg_required_else_help(true)
                .arg(arg!(-m --message <MESSAGE> "Commit message").required(true))
                .arg(arg!(-d --description <DESCRIPTION> "Commit description"))
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
            Command::new("staging")
                .about("View all staging changes")
        )

        .subcommand(
            Command::new("test")
                .about("test")
        )
    ;

    type CommandType = fn(&mut State, &ArgMatches);
    let commands: HashMap<String, CommandType> = HashMap::from_iter::<Vec<(String, CommandType)>>(vec![
        // TODO : pass user credentials into commands too
        ("init".to_string(), init),

        ("add".to_string(), add),
        ("remove".to_string(), remove),
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
            println!("{}", generate_tree(&s.current));
        }),
        ("staging".to_string(), |s, _| {
            println!("{}", s.get_changes().filter_changes(&s.track_set.initialise(&mut s.current)).serialise_changes());
        }),

        ("test".to_string(), |s, _| {
            s.upstream.apply_changes(s.get_changes());
        })
    ]);
    // #endregion

    match State::create(PathBuf::from(".")) {
        Ok(mut s) => {
            let c = command_handler.get_matches();
            let (command_name, sub_matches) = c.subcommand().unwrap();
            match commands.get(command_name) {
                Some(command) => {
                    command(&mut s, sub_matches);
                },
                None => {
                    println!("Relic Error, command not defined.")
                }
            }
        },
        Err(e) => {
            println!("main.rs (main) {e:?} error encountered.");
        }
    }
}
