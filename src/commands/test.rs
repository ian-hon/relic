use clap::ArgMatches;

use crate::core::{content_set::TrackingSet, State};

pub fn test(s: &mut State, _: &ArgMatches) {
    let c = s.get_changes();
    println!("trees: {:?}\n\n", c.trees);

    let t = s.track_set.initialise(&mut s.current);

    println!("content set: {:?}\n\n", t.directories);

    println!(
        "{:?}",
        // s.get_changes()
        c.filter_changes(&t).trees
    );
}
