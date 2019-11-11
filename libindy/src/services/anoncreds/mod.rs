pub mod helpers;
pub mod prover;
pub mod verifier;

use crate::services::anoncreds::prover::Prover;
use crate::services::anoncreds::verifier::Verifier;

pub struct AnoncredsService {
    pub prover: Prover,
    pub verifier: Verifier
}

impl AnoncredsService {
    pub fn new() -> AnoncredsService {
        AnoncredsService {
            prover: Prover::new(),
            verifier: Verifier::new()
        }
    }
}