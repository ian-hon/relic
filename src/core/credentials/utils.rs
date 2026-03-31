use std::slice::Iter;

use num_bigint::BigUint;
use sha2::{Digest, Sha256};

pub fn hash(numbers: &mut Iter<'_, &BigUint>, current: BigUint) -> BigUint {
    match numbers.next() {
        Some(i) => {
            let mut m = i.to_bytes_be().to_vec();
            m.append(&mut current.to_bytes_be().to_vec());

            let result = Sha256::digest(m);
            let c = BigUint::from_bytes_be(&result);

            // EXPENSIVE!
            hash(numbers, c.clone())
        }
        None => current,
    }
}
