use glass_pumpkin::prime;
use num_bigint::BigUint;

pub struct VerifierContext {
    // held by system
    pub generator: BigUint,
    pub group: BigUint,
}
impl VerifierContext {
    pub fn new(generator: BigUint, group: Option<BigUint>) -> Result<Self, ContextError> {
        let group = match group {
            Some(g) => {
                if prime::check(&g) {
                    g
                } else {
                    return Err(ContextError::InvalidGroup);
                }
            }
            None => prime::new(128).unwrap(),
        };

        Ok(Self { generator, group })
    }
}

#[derive(Debug)]
pub enum ContextError {
    InvalidGroup,
}
