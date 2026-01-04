use std::path::Path;

use clap::ArgMatches;

use crate::core::{data::tree::Tree, state::State, tracking::content_set::ContentSet};

pub fn test(state: Option<&mut State>, _: &ArgMatches) {
    let Some(state) = state else { return };

    // upstream:
    // 9752336e563af7b50b0063e60b8ca6f2a434fb8f8051534757d82df7d8411510 A
    // d97ae6d1de0111a4efdd10404c2e7733fbc377f085d8980f5d0f1f9779f7c09b B
    // fc2b9e12232e98ac6f069a6ec8797a8c719718f9d5dd6b6ba053a2ef9d79a8e4 C
    // 7968589d1461e55157fad1c76791a0efd0c913af4a2868259c951907ce75635f D +
    // b496157b71b466bbc383628575f80df9125ad791bdeb96d88047e0a7271eaf2c E +
    // 11783f50520096f9e60e40fbaee474a0a9f715f8d2e496dfb4c240058f1b8223 F +
    // 568ebafd5e7a3046e3c184d29cc953a3afcaf8d5694d1982e0c6029a8710a0d0 G +

    // local
    // 9752336e563af7b50b0063e60b8ca6f2a434fb8f8051534757d82df7d8411510 A
    // d97ae6d1de0111a4efdd10404c2e7733fbc377f085d8980f5d0f1f9779f7c09b B
    // fc2b9e12232e98ac6f069a6ec8797a8c719718f9d5dd6b6ba053a2ef9d79a8e4 C
    // a93ba7256c3e68352a420092592dd45a704c401deab3dd9ed870d70d3cc8c17a H +
    // 8b2e7aecdabfe9746c1292280fe297e27282dd5dd43c3f49285318defde21058 I +3

    // match Commit::get_state(
    //     match ObjectID::from_string(
    //         "568ebafd5e7a3046e3c184d29cc953a3afcaf8d5694d1982e0c6029a8710a0d0",
    //     )
    //     .construct(&state.get_sanctum_path())
    //     {
    //         Ok(r) => match r {
    //             Object::Commit(c) => c,
    //             _ => panic!("1"),
    //         },
    //         _ => panic!("2"),
    //     },
    //     match ObjectID::from_string(
    //         "d97ae6d1de0111a4efdd10404c2e7733fbc377f085d8980f5d0f1f9779f7c09b",
    //     )
    //     .construct(&state.get_sanctum_path())
    //     {
    //         Ok(r) => match r {
    //             Object::Commit(c) => c,
    //             _ => panic!("3"),
    //         },
    //         _ => panic!("4"),
    //     },
    //     &state.get_sanctum_path(),
    // ) {
    //     CommitState::Conflict(l) => println!("luca: {}", l.get_oid().to_string()),
    //     CommitState::Ahead(l) | CommitState::Behind(l) => {
    //         for i in l {
    //             println!("OID: {}", i.get_oid().to_string());
    //         }
    //     }
    //     i => println!("{i:?}"),
    // }

    // println!("{}", state.ignore_set.serialise());

    // let _ = Tree::build_tree(
    //     state,
    //     &state.root_path,
    //     &state.get_sanctum_path(),
    //     Path::new("."),
    // );

    let c = ContentSet::deserialise("lorem".to_string());
    println!("{}", c.serialise());
}
