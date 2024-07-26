use ark_crypto_primitives::sponge::Absorb;
use ark_ff::Field;
use lattirust_arithmetic::challenge_set::latticefold_challenge_set::OverField;

use crate::{
    arith::{Witness, LCCCS},
    transcript::Transcript,
    utils::sumcheck::SumCheckProof,
};

use super::{error::FoldingError, NIFSProver, NIFSVerifier};

#[derive(Clone)]
pub struct FoldingProof<F: Field, R: OverField<F>>
where
    F: Absorb,
{
    // Step 2.
    pub pointshift_sumcheck_proof: SumCheckProof<F, R>,
    // Step 3
    pub theta_s: Vec<R>,
    pub eta_s: Vec<R>,
}

pub trait FoldingProver<F: Field, R: OverField<F>, T: Transcript<F, R>> {
    type Proof: Clone;
    type Error: std::error::Error;

    fn prove(
        cm_i_s: &[LCCCS<R>],
        w_s: &[Witness<R>],
        transcript: &mut impl Transcript<F, R>,
    ) -> Result<(LCCCS<R>, Witness<R>, Self::Proof), Self::Error>;
}

pub trait FoldingVerifier<F: Field, R: OverField<F>, T: Transcript<F, R>> {
    type Prover: FoldingProver<F, R, T>;
    type Error = <Self::Prover as FoldingProver<F, R, T>>::Error;

    fn verify(
        cm_i_s: &[LCCCS<R>],
        proof: &<Self::Prover as FoldingProver<F, R, T>>::Proof,
        transcript: &mut impl Transcript<F, R>,
    ) -> Result<LCCCS<R>, Self::Error>;
}

impl<F: Field, R: OverField<F>, T: Transcript<F, R>> FoldingProver<F, R, T> for NIFSProver<F, R, T>
where
    F: Absorb,
{
    type Proof = FoldingProof<F, R>;
    type Error = FoldingError<R>;

    fn prove(
        _cm_i_s: &[LCCCS<R>],
        _w_s: &[Witness<R>],
        _transcript: &mut impl Transcript<F, R>,
    ) -> Result<(LCCCS<R>, Witness<R>, FoldingProof<F, R>), FoldingError<R>> {
        todo!()
    }
}

impl<F: Field, R: OverField<F>, T: Transcript<F, R>> FoldingVerifier<F, R, T>
    for NIFSVerifier<F, R, T>
where
    F: Absorb,
{
    type Prover = NIFSProver<F, R, T>;

    fn verify(
        _cm_i_s: &[LCCCS<R>],
        _proof: &<Self::Prover as FoldingProver<F, R, T>>::Proof,
        _transcript: &mut impl Transcript<F, R>,
    ) -> Result<LCCCS<R>, FoldingError<R>> {
        todo!()
    }
}
