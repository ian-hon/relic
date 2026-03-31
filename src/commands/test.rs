use std::{
    io::empty,
    path::{Path, PathBuf},
    str::FromStr,
};

use num_bigint::BigUint;
use sha2::{Digest, Sha256};
use similar::{ChangeTag, TextDiff};

use crate::core::{
    branch::branch::{Branch, BranchSource, HeadType},
    credentials::{self, FiatShamirProof, ProverContext, VerifierContext},
    data::{blob::Blob, commit::Commit, tree::Tree},
    modification::change::Change,
    object::{Object, ObjectLike},
    oid::ObjectID,
    state::State,
    tracking::content_set::ContentSet,
    util::{empty_oid, oid_digest},
};
use clap::Args;

#[derive(Args)]
pub struct TestArgs {}

pub fn test(state: &mut State, _args: TestArgs) {
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

    // let current =
    //     ObjectID::from_string("aa960aa5692ca817a7aa911b0f9ecdf41e0da88d1d26abd2180b5dd910ff2d09")
    //         .unwrap()
    //         .construct_strict::<Tree>(&state.get_sanctum_path())
    //         .unwrap();
    // let upstream =
    //     ObjectID::from_string("7f8e41b14f39fc3c712e6b98423e41a44780fa076a38b4ffcc405242beff2202")
    //         .unwrap()
    //         .construct_strict::<Tree>(&state.get_sanctum_path())
    //         .unwrap();

    // let change = Change::get_change_all(
    //     &upstream,
    //     &current,
    //     &state.get_sanctum_path(),
    //     &state.root_path,
    // );
    // println!(
    //     "{}",
    //     change.as_human_readable(&current, &state.get_sanctum_path())
    // );

    // for (b, s) in Branch::get_all_branches(state).unwrap() {
    //     println!(
    //         "{b}: {}",
    //         s.iter()
    //             .map(|i| format!("{:?}", i.0))
    //             .collect::<Vec<String>>()
    //             .join(", ")
    //     );
    // }

    // for p in Branch::construct_from_name("main", state, &BranchSource::Local)
    //     .unwrap()
    //     .get_commit(&state.get_sanctum_path())
    //     .unwrap()
    //     .get_all_parents(&state.get_sanctum_path())
    // {
    //     println!("{}", p.get_nickname(true));
    // }

    // println!("");

    // for (b, s) in Branch::get_all_branches(state).unwrap() {
    //     // println!(
    //     //     "{b}: {}",
    //     //     s.iter()
    //     //         .map(|i| format!("{:?}", i.0))
    //     //         .collect::<Vec<String>>()
    //     //         .join(", ")
    //     // );

    //     if b.eq("lorem") {
    //         continue;
    //     }

    //     println!("{}", b);
    //     for p in Branch::construct_from_name(&b, state, &BranchSource::Local)
    //         .unwrap()
    //         .get_commit(&state.get_sanctum_path())
    //         .unwrap()
    //         .get_all_parents(&state.get_sanctum_path())
    //     {
    //         println!("{}", p.get_nickname(true));
    //     }
    //     println!()
    // }

    // println!("{}", BigUint::from_bytes_be(&20u128.to_be_bytes()));

    // 91634880152443617534842621287039938041581081254914058002978601050179556493499
    // 28106838057724633541991236405213533498809717615002287594759165789551252471965
    // 21027550693477535543327579570081618952892630736730429980018215117041635618758
    // 21027550693477535543327579570081

    // 7700013830284619221829551641861

    // let a = BigUint::from_bytes_be(Sha256::digest(&"a").as_slice());
    // let b = BigUint::from_bytes_be(Sha256::digest(&"b").as_slice());
    // let c = BigUint::from_str("21027550693477535543327579570081").unwrap();

    // // println!("{a}");
    // // println!("{b}");
    // // println!("{c}");

    // let r = a.modpow(&b, &c);
    // println!("{r}");

    /*
    earth: 48aa92ff9395abe218782324ad2c195152565ba7d57c0cc3358afa9f1b1d3378
    main: 66ec1b6763661b753620737b1f9e7a37d7a22c0a3f86462a7a36669ef588f455


     */

    // println!();

    let h = |s: &str| {
        return BigUint::from_bytes_be(Sha256::digest(s).as_slice());
    };

    // let pc = ProverContext
    let vc = VerifierContext::new(h("blah blah blah"), None).unwrap();
    let pc = ProverContext::new(h("z"), &vc);

    assert!(pc.create_proof(h("s"), &vc).verify(&vc));
    assert!(pc.create_proof(h("2"), &vc).verify(&vc));
    assert!(pc.create_proof(h("3"), &vc).verify(&vc));

    // for p in Branch::construct_from_name("new_branch", state, &BranchSource::Upstream)
    //     .unwrap()
    //     .get_commit(&state.get_sanctum_path())
    //     .unwrap()
    //     .get_all_parents(&state.get_sanctum_path())
    // {
    //     println!("{}", p.get_nickname(true));
    // }

    // let t = crate::core::modification::TreeOp::new();
    // let t = crate::core::modification::TreeOp::DeleteTree();
    // let t = crate::core::modification::TreeOp::DeleteBlob();
    // let t = crate::core::modification::TreeOp::CreateTree();
    // let t = crate::core::modification::TreeOp::CreateBlob();

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
