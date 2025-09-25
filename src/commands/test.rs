use clap::ArgMatches;

use crate::core::State;

pub fn test(s: &mut State, _: &ArgMatches) {
    println!("{:?}", s.info);
}
