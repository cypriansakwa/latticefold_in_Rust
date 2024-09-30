use ark_std::fmt::Debug;
use lattirust_ring::OverField;

use cyclotomic_rings::{challenge_set::LatticefoldChallengeSet, SuitableRing};

pub mod poseidon;

pub trait Transcript<R: OverField> {
    type TranscriptConfig: Debug;

    fn new(config: &Self::TranscriptConfig) -> Self;

    fn absorb(&mut self, v: &R);

    fn absorb_field_element(&mut self, v: &R::BaseRing) {
        self.absorb(&From::from(*v))
    }

    fn absorb_slice(&mut self, v: &[R]) {
        for ring in v {
            self.absorb(ring);
        }
    }

    fn get_challenge(&mut self) -> R::BaseRing;

    fn get_challenges(&mut self, n: usize) -> Vec<R::BaseRing> {
        let mut challenges = Vec::with_capacity(n);
        challenges.extend((0..n).map(|_| self.get_challenge()));
        challenges
    }
}

pub trait TranscriptWithSmallChallenges<R: SuitableRing>: Transcript<R> {
    type ChallengeSet: LatticefoldChallengeSet<R>;

    fn get_small_challenge(&mut self) -> R::CoefficientRepresentation;

    fn get_small_challenges(&mut self, n: usize) -> Vec<R::CoefficientRepresentation> {
        let mut challenges = Vec::with_capacity(n);
        challenges.extend((0..n).map(|_| self.get_small_challenge()));
        challenges
    }
}
