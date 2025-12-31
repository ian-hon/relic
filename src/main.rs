use std::path::Path;

use crate::core::data::{object::Object, tree::Tree};

mod core;

mod cli;
mod commands;

fn main() {
    let s = Tree::build_tree(Path::new("."), Path::new(""));
    println!("{:?}", s);

    if let Ok(t) = s {
        println!("{}", t.as_string());
    }

    // let command_handler = cli::build();
    // let state = State::create(PathBuf::from("."));
    // let args = command_handler.handler.clone().get_matches();

    // cli::handle(command_handler, args, state);
}
