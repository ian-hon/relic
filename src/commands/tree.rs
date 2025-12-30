use clap::ArgMatches;

use crate::core::{utils, State};

pub fn tree(s: &mut State, _: &ArgMatches) {
    println!("{}", utils::generate_tree(&s.current));
}
