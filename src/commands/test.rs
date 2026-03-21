use std::{
    io::empty,
    path::{Path, PathBuf},
};

use clap::ArgMatches;
use similar::{ChangeTag, TextDiff};

use crate::core::{
    branch::branch::{Branch, BranchSource, HeadType},
    data::{
        blob::Blob,
        commit::{Commit, CommitState},
        tree::Tree,
    },
    modification::change::Change,
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

    // match Commit::get_state(
    //     match ObjectID::from_string(
    //         "ad689344e1c55d60c6eb0d615830374d644bada373e63911ea61bb7b139a88d9",
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
    //         "38b9f53395c045d88c38e51d2fab95c9b37607f85a43461ca55e2d7fb102f3cd",
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
    //     CommitState::Ahead(l) => {
    //         println!("ahead");
    //         for i in l {
    //             println!("OID: {}", i.get_oid().to_string());
    //         }
    //     }
    //     CommitState::Behind(l) => {
    //         println!("behind");
    //         for i in l {
    //             println!("OID: {}", i.get_oid().to_string());
    //         }
    //     }
    //     i => println!("{i:?}"),
    // }

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

    // let c = match ObjectID::from_string(
    //     "bca8fd26f1f9b4b6093960dab35d3b51a1b2545cc006e2146e84d59b7d50ea66",
    // )
    // .construct(&state.get_sanctum_path())
    // .unwrap()
    // {
    //     Object::Commit(c) => c,
    //     _ => panic!(),
    // };

    // println!("COMMIT TREE : {}", c.tree.to_string());

    // println!(
    //     "{:?}",
    //     crate::core::write::write_commit(
    //         &(Path::new("./lorem/ipsum/").into()),
    //         &state.get_sanctum_path(),
    //         &c,
    //     )
    // );

    // println!("{:?}", state.fetch_head_commit());
    // match Branch::get_head(state, &BranchSource::Local).unwrap() {
    //     HeadType::Branch(b) => println!(
    //         "branch: {}\n{}",
    //         b.name.clone(),
    //         b.get_commit(&state.get_sanctum_path()).unwrap().as_string()
    //     ),
    //     HeadType::Detached(c) => println!("detached: {}", c.as_string()),
    //     HeadType::Empty => println!("empty"),
    // }

    // 162436b5a034e432657cc2f44c089beedeeed9b40a23aa8c4111c33b38623194 state.rs
    // ca801104c3ca1974985cd4d12f0988dd5bd803b656aaeb7751dc835648533240 oid.rs

    // match Branch::get_head(state, &BranchSource::Local).unwrap() {
    //     HeadType::Branch(b) => {
    //         let c = b.get_commit(&state.get_sanctum_path()).unwrap();

    //         let t = c
    //             .tree
    //             .construct_strict::<Tree>(&state.get_sanctum_path())
    //             .unwrap();
    //         t.traverse(
    //             &state.get_sanctum_path(),
    //             PathBuf::new(),
    //             &|path, t, o| {
    //                 let oid = match &o {
    //                     Object::Blob(b) => b.oid,
    //                     Object::Tree(b) => b.oid,
    //                     Object::Commit(b) => b.oid,
    //                 };
    //                 println!("{} - {path:?} ({:?})", oid.to_string(), o.object_type());
    //             },
    //             &t,
    //         );
    //     }
    //     _ => {}
    // }

    // return;

    let a =
        ObjectID::from_string("5517d4ed1fc4048a6097ebdf4f290f216aebf9b1a1c22fad0a01ce47c6245611")
            .unwrap()
            .construct_strict::<Tree>(&state.get_sanctum_path())
            .unwrap();
    let b =
        ObjectID::from_string("7f8e41b14f39fc3c712e6b98423e41a44780fa076a38b4ffcc405242beff2202")
            .unwrap()
            .construct_strict::<Tree>(&state.get_sanctum_path())
            .unwrap();

    let change = Change::get_change_all(&a, &b, &state.get_sanctum_path(), &state.root_path);
    println!("{}", change.serialise_changes());

    // println!("{}", a.get_body_as_string().unwrap());
    // println!("{}", b.get_body_as_string().unwrap());

    // let upstream = format!("{}\n", b.get_body_as_string().unwrap());
    // let current = format!("{}\n", a.get_body_as_string().unwrap());

    // let diff = TextDiff::from_lines(&upstream, &current);

    // for change in diff.iter_all_changes().filter_map(|c| match c.tag() {
    //     ChangeTag::Equal => None,
    //     _ => Some(c),
    // }) {
    //     match change.tag() {
    //         ChangeTag::Delete => {
    //             println!(
    //                 "- {} {}",
    //                 change.old_index().unwrap(),
    //                 change.to_string().strip_suffix("\n").unwrap().to_string()
    //             );
    //         }
    //         ChangeTag::Insert => {
    //             println!(
    //                 "+{} {}",
    //                 change.new_index().unwrap(),
    //                 change.to_string().strip_suffix("\n").unwrap().to_string()
    //             );
    //         }
    //         _ => panic!("Unmatched change type: {}", change),
    //     }
    // }
}
