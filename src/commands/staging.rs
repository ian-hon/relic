use clap::ArgMatches;

use crate::core::{content_set::TrackingSet, State};

pub fn staging(s: &mut State, _: &ArgMatches) {
    println!(
        "{}",
        s.get_changes()
            .filter_changes(&s.track_set.initialise(&mut s.current))
            .serialise_changes()
    );
}
