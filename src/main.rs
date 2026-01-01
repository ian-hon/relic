use std::path::Path;

use relic_vcs::core::util::oid_to_string;

use crate::core::{data::tree::Tree, object::ObjectLike, oid::ObjectID, util::string_to_oid};

mod core;

mod cli;
mod commands;

/*
get ref system working


*/

fn main() {
    let s = Tree::build_tree(Path::new("."), Path::new(".relic/sanctum"));
    println!("{:?}", s);

    if let Ok(t) = s {
        println!("{}", t.as_string());

        println!("{}", t.get_oid().to_string());
    }

    let s: ObjectID =
        string_to_oid("4f54324bb72cefa823170ca06d42cfde41e7d955cdd46ab6532d2a3447968124").into();

    // println!("{:?}", s.construct(Path::new(".relic/sanctum")));

    // println!("{:?}", "T\0lorem ipsum".as_bytes());

    // let command_handler = cli::build();
    // let state = State::create(PathBuf::from("."));
    // let args = command_handler.handler.clone().get_matches();

    // cli::handle(command_handler, args, state);
}
