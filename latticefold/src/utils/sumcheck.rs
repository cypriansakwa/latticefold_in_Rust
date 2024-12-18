use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use ark_std::{fmt::Display, marker::PhantomData};
use prover::{ProverMsg, ProverState};
use stark_rings::{OverField, Ring};
use stark_rings_poly::polynomials::{ArithErrors, DenseMultilinearExtension};
use thiserror::Error;

use self::verifier::SubClaim;
use crate::{ark_base::*, transcript::Transcript};

pub mod prover;
pub mod utils;
pub mod verifier;

/// Interactive Proof for Multilinear Sumcheck
pub struct IPForMLSumcheck<R, T> {
    #[doc(hidden)]
    _marker: PhantomData<(R, T)>,
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

/// Sumcheck for products of multilinear polynomial
pub struct MLSumcheck<R, T>(#[doc(hidden)] PhantomData<(R, T)>);

/// proof generated by prover
#[derive(Clone, Debug, PartialEq, CanonicalSerialize, CanonicalDeserialize)]
pub struct Proof<R1: Ring>(Vec<ProverMsg<R1>>);

impl<R: OverField, T: Transcript<R>> MLSumcheck<R, T> {
    /// extract sum from the proof
    pub fn extract_sum(proof: &Proof<R>) -> R {
        proof.0[0].evaluations[0] + proof.0[0].evaluations[1]
    }

    /// This function does the same thing as `prove`, but it uses cryptographic sponge as the transcript/to generate the
    /// verifier challenges. Additionally, it returns the prover's state in addition to the proof.
    /// Both of these allow this sumcheck to be better used as a part of a larger protocol.
    pub fn prove_as_subprotocol(
        transcript: &mut T,
        mles: Vec<DenseMultilinearExtension<R>>,
        nvars: usize,
        degree: usize,
        comb_fn: impl Fn(&[R]) -> R + Sync + Send,
    ) -> (Proof<R>, ProverState<R>) {
        transcript.absorb(&R::from(nvars as u128));
        transcript.absorb(&R::from(degree as u128));
        let mut prover_state = IPForMLSumcheck::<R, T>::prover_init(mles, nvars, degree);
        let mut verifier_msg = None;
        let mut prover_msgs = Vec::with_capacity(nvars);
        for _ in 0..nvars {
            let prover_msg =
                IPForMLSumcheck::<R, T>::prove_round(&mut prover_state, &verifier_msg, &comb_fn);
            transcript.absorb_slice(&prover_msg.evaluations);
            prover_msgs.push(prover_msg);
            let next_verifier_msg = IPForMLSumcheck::<R, T>::sample_round(transcript);
            transcript.absorb(&next_verifier_msg.randomness.into());

            verifier_msg = Some(next_verifier_msg);
        }
        prover_state
            .randomness
            .push(verifier_msg.unwrap().randomness);

        (Proof(prover_msgs), prover_state)
    }

    /// This function does the same thing as `prove`, but it uses a cryptographic sponge as the transcript/to generate the
    /// verifier challenges. This allows this sumcheck to be used as a part of a larger protocol.
    pub fn verify_as_subprotocol(
        transcript: &mut T,
        nvars: usize,
        degree: usize,
        claimed_sum: R,
        proof: &Proof<R>,
    ) -> Result<SubClaim<R>, SumCheckError<R>> {
        transcript.absorb(&R::from(nvars as u128));
        transcript.absorb(&R::from(degree as u128));

        let mut verifier_state = IPForMLSumcheck::<R, T>::verifier_init(nvars, degree);
        for i in 0..nvars {
            let prover_msg = proof.0.get(i).expect("proof is incomplete");
            transcript.absorb_slice(&prover_msg.evaluations);
            let verifier_msg =
                IPForMLSumcheck::verify_round(prover_msg.clone(), &mut verifier_state, transcript);
            transcript.absorb(&verifier_msg.randomness.into());
        }

        IPForMLSumcheck::<R, T>::check_and_generate_subclaim(verifier_state, claimed_sum)
    }
}

#[cfg(test)]
mod tests {
    use ark_serialize::{CanonicalDeserialize, CanonicalSerialize, Compress, Validate};
    use ark_std::io::Cursor;
    use cyclotomic_rings::{challenge_set::LatticefoldChallengeSet, rings::SuitableRing};
    use rand::Rng;

    use crate::{
        ark_base::*,
        transcript::poseidon::PoseidonTranscript,
        utils::sumcheck::{
            utils::{rand_poly, rand_poly_comb_fn},
            MLSumcheck, Proof,
        },
    };

    fn generate_sumcheck_proof<R, CS>(
        nvars: usize,
        mut rng: &mut (impl Rng + Sized),
    ) -> (usize, R, Proof<R>)
    where
        R: SuitableRing,
        CS: LatticefoldChallengeSet<R>,
    {
        let mut transcript = PoseidonTranscript::<R, CS>::default();

        let ((poly_mles, poly_degree), products, sum) =
            rand_poly(nvars, (2, 5), 3, &mut rng).unwrap();

        let comb_fn = |vals: &[R]| -> R { rand_poly_comb_fn(vals, &products) };

        let (proof, _) = MLSumcheck::prove_as_subprotocol(
            &mut transcript,
            poly_mles,
            nvars,
            poly_degree,
            comb_fn,
        );
        (poly_degree, sum, proof)
    }

    fn test_sumcheck<R, CS>()
    where
        R: SuitableRing,
        CS: LatticefoldChallengeSet<R>,
    {
        let mut rng = ark_std::test_rng();
        let nvars = 5;

        for _ in 0..20 {
            let (poly_degree, sum, proof) = generate_sumcheck_proof::<R, CS>(nvars, &mut rng);

            let mut transcript: PoseidonTranscript<R, CS> = PoseidonTranscript::default();
            let res =
                MLSumcheck::verify_as_subprotocol(&mut transcript, nvars, poly_degree, sum, &proof);
            assert!(res.is_ok())
        }
    }

    fn test_sumcheck_proof_serialization<R, CS>()
    where
        R: SuitableRing,
        CS: LatticefoldChallengeSet<R>,
    {
        let mut rng = ark_std::test_rng();
        let nvars = 5;

        let proof = generate_sumcheck_proof::<R, CS>(nvars, &mut rng).2;

        let mut serialized = Vec::new();
        proof
            .serialize_with_mode(&mut serialized, Compress::Yes)
            .expect("Failed to serialize proof");

        let mut cursor = Cursor::new(&serialized);
        assert_eq!(
            proof,
            Proof::deserialize_with_mode(&mut cursor, Compress::Yes, Validate::Yes)
                .expect("Failed to deserialize proof")
        );
    }

    fn test_failing_sumcheck<R, CS>()
    where
        R: SuitableRing,
        CS: LatticefoldChallengeSet<R>,
    {
        let mut rng = ark_std::test_rng();

        for _ in 0..20 {
            let mut transcript: PoseidonTranscript<R, CS> = PoseidonTranscript::default();

            let nvars = 5;
            let ((poly_mles, poly_degree), products, _) =
                rand_poly(nvars, (2, 5), 3, &mut rng).unwrap();

            let comb_fn = |vals: &[R]| -> R { rand_poly_comb_fn(vals, &products) };

            let (proof, _) = MLSumcheck::prove_as_subprotocol(
                &mut transcript,
                poly_mles,
                nvars,
                poly_degree,
                comb_fn,
            );

            let not_sum = R::zero();

            let res = MLSumcheck::verify_as_subprotocol(
                &mut transcript,
                nvars,
                poly_degree,
                not_sum,
                &proof,
            );
            assert!(res.is_err());
        }
    }

    mod stark {
        use cyclotomic_rings::rings::StarkChallengeSet;
        use stark_rings::cyclotomic_ring::models::stark_prime::RqNTT;

        type CS = StarkChallengeSet;

        #[test]
        fn test_sumcheck() {
            super::test_sumcheck::<RqNTT, CS>();
        }

        #[test]
        fn test_sumcheck_proof_serialization() {
            super::test_sumcheck_proof_serialization::<RqNTT, CS>();
        }

        #[test]
        fn test_failing_sumcheck() {
            super::test_failing_sumcheck::<RqNTT, CS>();
        }
    }

    mod frog {
        use cyclotomic_rings::rings::FrogChallengeSet;
        use stark_rings::cyclotomic_ring::models::frog_ring::RqNTT;

        type CS = FrogChallengeSet;

        #[test]
        fn test_sumcheck() {
            super::test_sumcheck::<RqNTT, CS>();
        }

        #[test]
        fn test_sumcheck_proof_serialization() {
            super::test_sumcheck_proof_serialization::<RqNTT, CS>();
        }

        #[test]
        fn test_failing_sumcheck() {
            super::test_failing_sumcheck::<RqNTT, CS>();
        }
    }

    mod goldilocks {
        use cyclotomic_rings::rings::GoldilocksChallengeSet;
        use stark_rings::cyclotomic_ring::models::goldilocks::RqNTT;

        type CS = GoldilocksChallengeSet;

        #[test]
        fn test_sumcheck() {
            super::test_sumcheck::<RqNTT, CS>();
        }

        #[test]
        fn test_sumcheck_proof_serialization() {
            super::test_sumcheck_proof_serialization::<RqNTT, CS>();
        }

        #[test]
        fn test_failing_sumcheck() {
            super::test_failing_sumcheck::<RqNTT, CS>();
        }
    }

    mod babybear {
        use cyclotomic_rings::rings::BabyBearChallengeSet;
        use stark_rings::cyclotomic_ring::models::babybear::RqNTT;

        type CS = BabyBearChallengeSet;

        #[test]
        fn test_sumcheck() {
            super::test_sumcheck::<RqNTT, CS>();
        }

        #[test]
        fn test_sumcheck_proof_serialization() {
            super::test_sumcheck_proof_serialization::<RqNTT, CS>();
        }

        #[test]
        fn test_failing_sumcheck() {
            super::test_failing_sumcheck::<RqNTT, CS>();
        }
    }
}
