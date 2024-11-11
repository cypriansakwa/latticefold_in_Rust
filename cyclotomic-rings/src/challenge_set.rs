//!
//!  Short challenge set API.
//!

use error::ChallengeSetError;
use lattirust_ring::{
    cyclotomic_ring::models::pow2_debug::{Pow2CyclotomicPolyRing, Pow2CyclotomicPolyRingNTT},
    zn::z_q::Zq,
};

use crate::rings::SuitableRing;

pub mod error;

/// A trait to specify short challenge set for use in the LatticeFold protocol.
pub trait LatticefoldChallengeSet<R: SuitableRing> {
    /// Amount of bytes needed to obtain a single short challenge.
    const BYTES_NEEDED: usize;

    /// Given a slice of bytes `bs` returns the short challenge encode with these bytes
    /// in the coefficient form. Returns `TooFewBytes` error if there is not enough bytes
    /// to obtain a short challenge.
    fn short_challenge_from_random_bytes(
        bs: &[u8],
    ) -> Result<R::CoefficientRepresentation, ChallengeSetError>;
}

pub struct BinarySmallSet<const Q: u64, const N: usize>;

impl<const Q: u64, const N: usize> LatticefoldChallengeSet<Pow2CyclotomicPolyRingNTT<Q, N>>
    for BinarySmallSet<Q, N>
where
    Pow2CyclotomicPolyRingNTT<Q, N>:
        SuitableRing<CoefficientRepresentation = Pow2CyclotomicPolyRing<Q, N>>,
{
    const BYTES_NEEDED: usize = N * 8;

    fn short_challenge_from_random_bytes(
        bs: &[u8],
    ) -> Result<Pow2CyclotomicPolyRing<Q, N>, ChallengeSetError> {
        if bs.len() != Self::BYTES_NEEDED {
            return Err(ChallengeSetError::TooFewBytes(bs.len(), Self::BYTES_NEEDED));
        }

        Ok(Pow2CyclotomicPolyRing::from(
            bs.iter().map(|&x| Zq::from(x)).collect::<Vec<_>>(),
        ))
    }
}
