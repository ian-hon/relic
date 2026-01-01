use std::path::Path;

use crate::core::{data::tree::Tree, object::ObjectLike};

mod core;

mod cli;
mod commands;

fn main() {
    let s = Tree::build_tree(Path::new("."), Path::new(".relic/sanctum"));
    println!("{:?}", s);

    if let Ok(t) = s {
        println!("{}", t.as_string());

        println!("{}", t.get_oid().to_string());
    }

    // println!("{:?}", "T\0lorem ipsum".as_bytes());

    // let command_handler = cli::build();
    // let state = State::create(PathBuf::from("."));
    // let args = command_handler.handler.clone().get_matches();

    // cli::handle(command_handler, args, state);
}
