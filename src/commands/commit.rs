use std::path::Path;

use clap::ArgMatches;

pub fn commit(path: &Path, args: &ArgMatches) {
    let message = args.get_one::<String>("message").unwrap().clone();
    let description = args
        .get_one::<String>("description")
        .map_or("".to_string(), String::clone);

    // TODO: add remote checking in here
    // for now, just worry about upstream

    
}
