pub mod prover;
pub mod univ_poly;
pub mod verifier;

use std::fmt::Display;

use crate::transcript::Transcript;
use lattirust_arithmetic::challenge_set::latticefold_challenge_set::OverField;
use lattirust_arithmetic::polynomials::ArithErrors;
use lattirust_arithmetic::polynomials::VPAuxInfo;
use lattirust_arithmetic::ring::Ring;
use thiserror::Error;
use univ_poly::UnivPoly;

pub struct SumCheckIP<R: OverField> {
    pub claimed_sum: R,
    pub poly_info: VPAuxInfo<R>,
}

impl<R: OverField> SumCheckIP<R> {
    pub fn new(claimed_sum: R, poly_info: VPAuxInfo<R>) -> Self {
        SumCheckIP {
            claimed_sum,
            poly_info,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SumCheckProof<R: OverField> {
    pub rounds: Vec<SumCheckRound<R>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SumCheckRound<R: OverField> {
    pub unipoly: UnivPoly<R>,
}

#[derive(Error, Debug)]
pub enum SumCheckError<R: Ring + Display> {
    #[error("univariate polynomial evaluation error")]
    EvaluationError(ArithErrors),
    #[error("incorrect sumcheck sum. Expected `{0}`. Received `{1}`")]
    SumCheckFailed(R, R),
    #[error("max degree exceeded")]
    MaxDegreeExceeded,
}

impl<R: Ring> From<ArithErrors> for SumCheckError<R> {
    fn from(arith_error: ArithErrors) -> Self {
        Self::EvaluationError(arith_error)
    }
}

impl<R: OverField> SumCheckProof<R> {
    pub fn new(num_rounds: usize) -> SumCheckProof<R> {
        SumCheckProof {
            rounds: Vec::with_capacity(num_rounds),
        }
    }

    pub fn add_round(&mut self, transcript: &mut impl Transcript<R>, unipoly: UnivPoly<R>) {
        transcript.absorb_ring_vec(&unipoly.coeffs);
        let round = SumCheckRound { unipoly };

        self.rounds.push(round);
    }
}

impl<R: OverField> SumCheckRound<R> {
    pub fn new(unipoly: UnivPoly<R>) -> SumCheckRound<R> {
        SumCheckRound { unipoly }
    }
}
