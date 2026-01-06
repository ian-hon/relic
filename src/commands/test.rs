use std::{io::empty, path::Path};

use clap::ArgMatches;

use crate::core::{
    data::{
        commit::{Commit, CommitState},
        tree::Tree,
    },
    object::{Object, ObjectLike},
    oid::ObjectID,
    state::State,
    tracking::content_set::ContentSet,
    util::{empty_oid, oid_digest},
};

pub fn test(state: Option<&mut State>, _: &ArgMatches) {
    let Some(state) = state else { return };

    // b875a2f0b56ec1f409a4f1c8f459eff3dae0ded5b389cf9aed50e10e5c34d001
    // 7780bf9fe8fcc104f7a4ed5966597629fdef49a3d6224da15659879c1628b8df
    // 03a0bdb5f55dca017213ef209e8b81b8fb4ecf61ecf0b8b37892a18ac082e76c <
    // 24fc8a2a3a465740221c59e0bdc029a0738cde2228f8121a900f9f33d814e6f2
    // 38b9f53395c045d88c38e51d2fab95c9b37607f85a43461ca55e2d7fb102f3cd

    // b875a2f0b56ec1f409a4f1c8f459eff3dae0ded5b389cf9aed50e10e5c34d001
    // 7780bf9fe8fcc104f7a4ed5966597629fdef49a3d6224da15659879c1628b8df
    // 03a0bdb5f55dca017213ef209e8b81b8fb4ecf61ecf0b8b37892a18ac082e76c <
    // 93ffed3e8af5b67c1e7f1e576acb6a675bd34f84c1f5935b1c9e62bbf3c6d701
    // ad689344e1c55d60c6eb0d615830374d644bada373e63911ea61bb7b139a88d9

    match Commit::get_state(
        match ObjectID::from_string(
            "ad689344e1c55d60c6eb0d615830374d644bada373e63911ea61bb7b139a88d9",
        )
        .construct(&state.get_sanctum_path())
        {
            Ok(r) => match r {
                Object::Commit(c) => c,
                _ => panic!("1"),
            },
            _ => panic!("2"),
        },
        match ObjectID::from_string(
            "38b9f53395c045d88c38e51d2fab95c9b37607f85a43461ca55e2d7fb102f3cd",
        )
        .construct(&state.get_sanctum_path())
        {
            Ok(r) => match r {
                Object::Commit(c) => c,
                _ => panic!("3"),
            },
            _ => panic!("4"),
        },
        &state.get_sanctum_path(),
    ) {
        CommitState::Conflict(l) => println!("luca: {}", l.get_oid().to_string()),
        CommitState::Ahead(l) => {
            println!("ahead");
            for i in l {
                println!("OID: {}", i.get_oid().to_string());
            }
        }
        CommitState::Behind(l) => {
            println!("behind");
            for i in l {
                println!("OID: {}", i.get_oid().to_string());
            }
        }
        i => println!("{i:?}"),
    }

    // println!(
    //     "{}",
    //     match ObjectID::from_string(
    //         "204e3546cc770b1880afb6969ff24c364e9e24d8db145df9e2d11c3ad17a894f"
    //     )
    //     .construct(&state.get_sanctum_path())
    //     .unwrap()
    //     {
    //         Object::Commit(c) => c.get_oid().to_string(),
    //         _ => panic!(),
    //     }
    // );
}
