use clap::ArgMatches;

use crate::core::State;

pub fn clone(_: &mut State, args: &ArgMatches) {
    if let Some(remote) = args.get_one::<String>("URL") {
        println!("remote : {remote}");

        // validate if remote is a relic repository
        // probably need some versioning system

        // let _ = fs::create_dir(paths::RELIC_PATH_PARENT);
        // let _ = fs::create_dir(paths::RELIC_PATH_HISTORY);
        // let _ = fs::create_dir(paths::RELIC_PATH_PENDING);
        // let _ = fs::write(paths::RELIC_PATH_INFO, DEFAULT_INFO);
        // let _ = fs::write(paths::RELIC_PATH_ROOT, "");
        // let _ = fs::write(paths::RELIC_PATH_TRACKED, "");
        // let _ = fs::write(paths::RELIC_PATH_UPSTREAM, DEFAULT_UPSTREAM);

        // let _ = fs::write(paths::RELIC_PATH_IGNORE, content_set::DEFAULT_IGNORE);
    } else {
        println!("No remote URL provided.");
    }
}
