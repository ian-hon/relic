use crate::core::{
    credentials::{signature::Signature, Identity},
    state::State,
};
use clap::Args;

#[derive(Args)]
pub struct TestArgs {}

pub fn test(_: &mut State, _args: TestArgs) {
    let identity = Identity::new("hmmm", "hii");

    let private_key = Signature::generate(&identity).unwrap();
    let public_key = private_key.public_key();

    println!("COMMENT: {}", public_key.comment());

    let data = b"commit data to sign";
    let signature = Signature::sign(data, &private_key).unwrap();

    let r = signature.serialise();
    println!("{r}");

    println!("{:?}", Signature::deserialise(&r));

    println!("Valid: {}", signature.verify(public_key, data));
}
