/*

x (priv key)
h = g^x (pub key)

g = generator
r = random nonce
q = prime
G = cyclic group of size q

u = g^r mod q (commitment)
c = hash(g, q, h, u, pre-commit hash) (challenge)
z = r + cx (response)

*/

// 64 * 16 / 8

// ca978112ca1bbdcafac231b39a23dc4da786eff8147c4e72b9807785afee48bb

use num_bigint::BigUint;

use crate::core::credentials::{prover::ProverContext, utils::hash, verifier::VerifierContext};

pub struct FiatShamirProof<'a> {
    commitment: BigUint,
    challenge: BigUint,
    response: BigUint,

    pre_commit_hash: BigUint,
    prover: &'a ProverContext,
}
impl<'a> FiatShamirProof<'a> {
    pub fn new(
        commitment: BigUint,
        challenge: BigUint,
        response: BigUint,

        pre_commit_hash: BigUint,
        prover: &'a ProverContext,
    ) -> Self {
        Self {
            commitment,
            challenge,
            response,
            pre_commit_hash,
            prover,
        }
    }

    pub fn verify(&self, ctx: &VerifierContext) -> bool {
        let challenge = hash(
            &mut vec![
                &ctx.group,
                &self.prover.public_key,
                &self.commitment,
                &self.pre_commit_hash,
            ]
            .iter(),
            ctx.generator.clone(),
        );

        // g^z = u⋅h^c
        let u_h_c = (self.commitment.clone()
            * (self.prover.public_key.modpow(&challenge, &ctx.group)))
            % &ctx.group;
        let g_z = ctx.generator.clone().modpow(&self.response, &ctx.group);

        g_z.eq(&u_h_c) && challenge.eq(&self.challenge)
    }

    pub fn serialise(&self) -> String {
        format!(
            "{} {} {} {}",
            self.commitment, self.challenge, self.response, self.prover.public_key
        )
    }
}
