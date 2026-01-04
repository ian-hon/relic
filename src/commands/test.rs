use std::path::Path;

use clap::ArgMatches;

use crate::core::{
    data::commit::{Commit, CommitState},
    object::{Object, ObjectLike},
    oid::ObjectID,
};

pub fn test(_: &Path, relic_path: &Path, _: &ArgMatches) {
    // if let Ok(s) = Tree::build_tree(path, &path.join(".relic/sanctum")) {
    //     println!("{}", s.get_oid().to_string());
    // }

    let sanctum_path = relic_path.join("sanctum");

    // tree: fcd55f8ce8996546d9a9001bddae06c6800f92f5015943535a7bf6980c0e9600
    // let c = Commit::new(
    //     string_to_oid("fcd55f8ce8996546d9a9001bddae06c6800f92f5015943535a7bf6980c0e9600").into(),
    //     Some(
    //         string_to_oid("5df883ee243ad58ae593a55deca45985681d7de9fbc5455cae455f2d672974ee")
    //             .into(),
    //     ),
    //     get_time(),
    //     "none".to_string(),
    //     "blehhh".to_string(),
    //     "".to_string(),
    //     &sanctum_path,
    // );

    // println!("{}\n\n{}", c.serialise(), c.get_oid().to_string());

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
    // 8b2e7aecdabfe9746c1292280fe297e27282dd5dd43c3f49285318defde21058 I +

    match Commit::get_state(
        match ObjectID::from_string(
            "11783f50520096f9e60e40fbaee474a0a9f715f8d2e496dfb4c240058f1b8223",
        )
        .construct(&sanctum_path)
        {
            Ok(r) => match r {
                Object::Commit(c) => c,
                _ => panic!("1"),
            },
            _ => panic!("2"),
        },
        match ObjectID::from_string(
            "8b2e7aecdabfe9746c1292280fe297e27282dd5dd43c3f49285318defde21058",
        )
        .construct(&sanctum_path)
        {
            Ok(r) => match r {
                Object::Commit(c) => c,
                _ => panic!("3"),
            },
            _ => panic!("4"),
        },
        &sanctum_path,
    ) {
        CommitState::Conflict(l) => println!("{}", l.as_string()),
        CommitState::Ahead(l) | CommitState::Behind(l) => {
            for i in l {
                println!("OID: {}", i.get_oid().to_string());
            }
        }
        i => println!("{i:?}"),
    }

    // let c = match ObjectID::from_string(
    //     "d6ec2763bee6e67ec489bf7edcc08678c94c9d9b40d37dcecf88d681ba204336",
    // )
    // .construct(&sanctum_path)
    // {
    //     Ok(r) => match r {
    //         Object::Commit(c) => c,
    //         _ => panic!("3"),
    //     },
    //     _ => panic!("4"),
    // };

    // println!("{}", c.get_oid().to_string());

    // let s = Commit::deserialise(c.serialise().as_bytes().to_vec()).unwrap();
    // println!("{}", s.get_oid().to_string());

    // d6ec2763bee6e67ec489bf7edcc08678c94c9d9b40d37dcecf88d681ba204336
}
