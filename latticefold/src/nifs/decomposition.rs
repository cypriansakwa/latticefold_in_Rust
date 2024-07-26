use std::marker::PhantomData;

use ark_crypto_primitives::sponge::Absorb;
use ark_ff::Field;
use lattirust_arithmetic::challenge_set::latticefold_challenge_set::OverField;

use crate::{
    arith::{Witness, LCCCS},
    transcript::Transcript,
};

use super::{error::DecompositionError, NIFSProver, NIFSVerifier};

#[derive(Clone)]
pub struct DecompositionProof<F: Field, R: OverField<F>>
where
    F: Absorb,
{
    _f: PhantomData<F>,
    pub u_s: Vec<Vec<R>>,
    pub v_s: Vec<R>,
    pub x_s: Vec<Vec<R>>,
    pub y_s: Vec<Vec<R>>,
}

pub trait DecompositionProver<F: Field, R: OverField<F>, T: Transcript<F, R>> {
    type Proof: Clone;
    type Error: std::error::Error;

    fn prove(
        cm_i: &LCCCS<R>,
        wit: &Witness<R>,
        transcript: &mut impl Transcript<F, R>,
    ) -> Result<(Vec<LCCCS<R>>, Vec<Witness<R>>, Self::Proof), Self::Error>;
}

pub trait DecompositionVerifier<F: Field, R: OverField<F>, T: Transcript<F, R>> {
    type Prover: DecompositionProver<F, R, T>;
    type Error = <Self::Prover as DecompositionProver<F, R, T>>::Error;

    fn verify(
        cm_i: &LCCCS<R>,
        proof: &<Self::Prover as DecompositionProver<F, R, T>>::Proof,
        transcript: &mut impl Transcript<F, R>,
    ) -> Result<Vec<LCCCS<R>>, Self::Error>;
}

impl<F: Field, R: OverField<F>, T: Transcript<F, R>> DecompositionProver<F, R, T>
    for NIFSProver<F, R, T>
where
    F: Absorb,
{
    type Proof = DecompositionProof<F, R>;
    type Error = DecompositionError<R>;

    fn prove(
        _cm_i: &LCCCS<R>,
        _wit: &Witness<R>,
        _transcript: &mut impl Transcript<F, R>,
    ) -> Result<(Vec<LCCCS<R>>, Vec<Witness<R>>, DecompositionProof<F, R>), DecompositionError<R>>
    {
        todo!()
    }
}

impl<F: Field, R: OverField<F>, T: Transcript<F, R>> DecompositionVerifier<F, R, T>
    for NIFSVerifier<F, R, T>
where
    F: Absorb,
{
    type Prover = NIFSProver<F, R, T>;

    fn verify(
        _cm_i: &LCCCS<R>,
        _proof: &<Self::Prover as DecompositionProver<F, R, T>>::Proof,
        _transcript: &mut impl Transcript<F, R>,
    ) -> Result<Vec<LCCCS<R>>, DecompositionError<R>> {
        todo!()
    }
}
