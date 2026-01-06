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

    // upstream:
    // 2a627be346271aa7d92e75c40fecf1c6eeb580f1edcc0d7a89b72fa83d387b50
    // 3453a3171d753f9c330660516e17d858ce038b8717627751028852bdd7607cc1
    // 082abe8f89802f126260fc094dce9dab6377be2d3e1fe6d9ce192cfe809f9bf0 <
    // 62cf36b74df75fac94b17ba324e1ed41457288a6416f6ed5580452ab70c70b9c
    // 3f2f82ac1feedbba44968b9ccacdc116b642ededc4240c9fc3a6f2a84e5c999d
    // 36b04bc9caa6ad24c58c2bab388c26b6170ed4503cf40d8c36c966d64ea58d04 (contains local[5] as surrogate)

    // local
    // 2a627be346271aa7d92e75c40fecf1c6eeb580f1edcc0d7a89b72fa83d387b50
    // 3453a3171d753f9c330660516e17d858ce038b8717627751028852bdd7607cc1
    // 082abe8f89802f126260fc094dce9dab6377be2d3e1fe6d9ce192cfe809f9bf0 <
    // eb72583727a851a6aeb229d829b0fa3d89f9d858354981e5c489d202a10e8818
    // 839850acdfa832ab1125d22d4eb936ed317a9dca637ca19d4c96dd21143cecc5

    // merge local[5] into upstream
    // local[5]: 839850acdfa832ab1125d22d4eb936ed317a9dca637ca19d4c96dd21143cecc5

    match Commit::get_state(
        match ObjectID::from_string(
            "36b04bc9caa6ad24c58c2bab388c26b6170ed4503cf40d8c36c966d64ea58d04",
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
            "839850acdfa832ab1125d22d4eb936ed317a9dca637ca19d4c96dd21143cecc5",
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
    //         "9c638178b4fa917750bc1ee1a23f9896515013977f31cd3c935400f01fd76950"
    //     )
    //     .construct(&state.get_sanctum_path())
    //     .unwrap()
    //     {
    //         Object::Commit(c) => c.get_oid().to_string(),
    //         _ => panic!(),
    //     }
    // );
}
