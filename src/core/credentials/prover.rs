use glass_pumpkin::prime;
use num_bigint::BigUint;

use crate::core::credentials::{proof::FiatShamirProof, utils::hash, verifier::VerifierContext};

pub struct ProverContext {
    // held by user
    private_key: BigUint,    // maybe dont store this
    pub public_key: BigUint, // public_key: g^x mod q
}
impl ProverContext {
    pub fn new(private_key: BigUint, verifier_context: &VerifierContext) -> Self {
        Self {
            public_key: verifier_context
                .generator
                .modpow(&(private_key.clone()), &verifier_context.group),
            private_key,
        }
    }

    pub fn create_proof<'a, T>(
        &'a self,
        pre_commit_hash: T,
        ctx: &VerifierContext,
    ) -> FiatShamirProof<'a>
    where
        T: Into<BigUint>,
    {
        let pre_commit_hash = pre_commit_hash.into();

        // u = g^r mod q
        // c = hash(g, q, h, u, commit_hash)
        // z = r + cx

        // let r = prime::new(128).unwrap(); // lets hope this doesnt come back to haunt me
        let r = prime::new(128).unwrap(); // lets hope this doesnt come back to haunt me

        // signature: h1 h2 h3 h4

        let commitment = ctx.generator.modpow(&r, &ctx.group);
        let challenge = hash(
            &mut vec![&ctx.group, &self.public_key, &commitment, &pre_commit_hash].iter(),
            ctx.generator.clone(),
        );
        let response = r + (challenge.clone() * self.private_key.clone());

        FiatShamirProof::new(commitment, challenge, response, pre_commit_hash, &self)
    }
}
