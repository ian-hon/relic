use std::path::Path;

use clap::ArgMatches;

use crate::core::{
    data::{
        commit::{Commit, CommitState},
        tree::Tree,
    },
    object::{Object, ObjectLike},
    oid::ObjectID,
    util::{get_time, string_to_oid},
};

pub fn test(path: &Path, relic_path: &Path, _: &ArgMatches) {
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
    // 0a5d78505b7904241133c6a06ac130cc4dc5f8f378177970c3cd5f000f81ad02 A
    // cf3e6c3bfc28b2c1e4cecf2e46e4cb916ac8e0b0a424fb671a85fa418f4cf1cd B
    // ce67cbf919340f60d484304afa8b0f9011c505b4528986f8826e8bd349562aaa C
    // 5df883ee243ad58ae593a55deca45985681d7de9fbc5455cae455f2d672974ee D +
    // 792132629d1c33eda2c171408f6cadf7328f1ffa43355a797400ab77046b716c E +

    // local
    // 0a5d78505b7904241133c6a06ac130cc4dc5f8f378177970c3cd5f000f81ad02 A
    // cf3e6c3bfc28b2c1e4cecf2e46e4cb916ac8e0b0a424fb671a85fa418f4cf1cd B
    // ce67cbf919340f60d484304afa8b0f9011c505b4528986f8826e8bd349562aaa C
    // 0af3ee59e2fc28a72f9fd8ae62f3f9ef1faf28e6338290ddf2473b132fa0f541 F +
    // e70dc52f20550140dd4ad5b4d65a50daa34260d75e0bdcd0316a62238b907025 G +

    println!(
        "{:?}",
        match Commit::get_state(
            match ObjectID::from_string(
                "3ff716050b04ab3a7dd4575d0f436018ad03df6718f0c57a52cd15f039069454"
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
                "b565162264c6be6be2e62918cc67022ffd49f4953511dca56a4687714f20bf97"
            )
            .construct(&sanctum_path)
            {
                Ok(r) => match r {
                    Object::Commit(c) => c,
                    _ => panic!("3"),
                },
                _ => panic!("4"),
            },
            &sanctum_path
        ) {
            CommitState::Conflict(l) => l.to_string(),
            i => format!("{i:?}"),
        }
    );

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
