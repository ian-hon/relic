use clap::ArgMatches;

use crate::core::{content_set::TrackingSet, State};

pub fn staging(s: &mut State, _: &ArgMatches) {
    println!(
        "{}",
        s.get_changes()
            .filter_changes(&s.track_set.initialise(&mut s.current))
            // TODO: filter_changes is removing DeletedTree/DeletedBlob
            // this is not intended behaviour
            // repro: delete a file/tree, then view if its shown in `relic staging`
            .as_human_readable(&s.upstream) // .trees
    );
}
