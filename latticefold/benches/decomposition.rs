#![allow(incomplete_features)]
use criterion::{
    criterion_group, criterion_main, AxisScale, BenchmarkId, Criterion, PlotConfiguration,
};
use cyclotomic_rings::{
    challenge_set::LatticefoldChallengeSet,
    rings::{
        GoldilocksChallengeSet, GoldilocksRingNTT, StarkChallengeSet, StarkRingNTT, SuitableRing,
    },
};
use std::{fmt::Debug, time::Duration};
mod macros;
mod utils;
use ark_std::UniformRand;
use latticefold::{
    arith::{Witness, CCCS, CCS},
    commitment::AjtaiCommitmentScheme,
    decomposition_parameters::DecompositionParams,
    nifs::{
        decomposition::{
            DecompositionProver, DecompositionVerifier, LFDecompositionProver,
            LFDecompositionVerifier,
        },
        linearization::{
            LFLinearizationProver, LFLinearizationVerifier, LinearizationProver,
            LinearizationVerifier,
        },
    },
    transcript::poseidon::PoseidonTranscript,
};
use utils::{wit_and_ccs_gen, wit_and_ccs_gen_non_scalar};

fn prover_decomposition_benchmark<
    const C: usize,
    const W: usize,
    P: DecompositionParams,
    R: Clone + UniformRand + Debug + SuitableRing,
    CS: LatticefoldChallengeSet<R>,
>(
    c: &mut criterion::BenchmarkGroup<criterion::measurement::WallTime>,
    cm_i: &CCCS<C, R>,
    wit: &Witness<R>,
    ccs: &CCS<R>,
    scheme: &AjtaiCommitmentScheme<C, W, R>,
) {
    let mut prover_transcript = PoseidonTranscript::<R, CS>::default();
    let mut verifier_transcript = PoseidonTranscript::<R, CS>::default();

    let (_, linearization_proof) = LFLinearizationProver::<_, PoseidonTranscript<R, CS>>::prove(
        cm_i,
        wit,
        &mut prover_transcript,
        ccs,
    )
    .unwrap();

    let lcccs = LFLinearizationVerifier::<_, PoseidonTranscript<R, CS>>::verify(
        cm_i,
        &linearization_proof,
        &mut verifier_transcript,
        ccs,
    )
    .unwrap();

    c.bench_with_input(
        BenchmarkId::new(
            "Decomposition Prover",
            format!(
                "Param. Kappa={}, Cols={},  B={}, L={}, B_small={}, K={}",
                C,
                { W / P::L },
                P::B,
                P::L,
                P::B_SMALL,
                P::K
            ),
        ),
        &(lcccs, wit, ccs),
        |b, (lcccs, wit, ccs)| {
            b.iter(|| {
                let (_, _, _, _) = LFDecompositionProver::<_, PoseidonTranscript<R, CS>>::prove::<
                    W,
                    C,
                    P,
                >(
                    lcccs, wit, &mut prover_transcript, ccs, scheme
                )
                .unwrap();
            })
        },
    );
}

fn verifier_decomposition_benchmark<
    const C: usize,
    const W: usize,
    P: DecompositionParams,
    R: Clone + UniformRand + Debug + SuitableRing,
    CS: LatticefoldChallengeSet<R> + Clone,
>(
    c: &mut criterion::BenchmarkGroup<criterion::measurement::WallTime>,
    cm_i: &CCCS<C, R>,
    wit: &Witness<R>,
    ccs: &CCS<R>,
    scheme: &AjtaiCommitmentScheme<C, W, R>,
) {
    let mut prover_transcript = PoseidonTranscript::<R, CS>::default();
    let mut verifier_transcript = PoseidonTranscript::<R, CS>::default();

    let (_, linearization_proof) = LFLinearizationProver::<_, PoseidonTranscript<R, CS>>::prove(
        cm_i,
        wit,
        &mut prover_transcript,
        ccs,
    )
    .expect("Failed to generate linearization proof");

    let lcccs = LFLinearizationVerifier::<_, PoseidonTranscript<R, CS>>::verify(
        cm_i,
        &linearization_proof,
        &mut verifier_transcript,
        ccs,
    )
    .expect("Failed to verify linearization proof");

    let (_, _, _, decomposition_proof) =
        LFDecompositionProver::<_, PoseidonTranscript<R, CS>>::prove::<W, C, P>(
            &lcccs,
            wit,
            &mut prover_transcript,
            ccs,
            scheme,
        )
        .expect("Failed to generate decomposition proof");

    c.bench_with_input(
        BenchmarkId::new(
            "Decomposition Verifier",
            format!(
                "Param. Kappa={}, Cols={},  B={}, L={}, B_small={}, K={}",
                C,
                { W / P::L },
                P::B,
                P::L,
                P::B_SMALL,
                P::K
            ),
        ),
        &(lcccs, decomposition_proof, ccs),
        |b, (lcccs, proof, ccs)| {
            b.iter_batched(
                || verifier_transcript.clone(),
                |mut bench_verifier_transcript| {
                    let _ =
                        LFDecompositionVerifier::<_, PoseidonTranscript<R, CS>>::verify::<C, P>(
                            lcccs,
                            proof,
                            &mut bench_verifier_transcript,
                            ccs,
                        )
                        .expect("Failed to verify decomposition proof");
                },
                criterion::BatchSize::SmallInput,
            );
        },
    );
}

fn decomposition_benchmarks_scalar<
    const X_LEN: usize,
    const C: usize,
    const WIT_LEN: usize,
    const W: usize,
    CS: LatticefoldChallengeSet<R> + Clone,
    R: Clone + UniformRand + Debug + SuitableRing,
    P: DecompositionParams + Clone,
>(
    group: &mut criterion::BenchmarkGroup<criterion::measurement::WallTime>,
) {
    let r1cs_rows = X_LEN + WIT_LEN + 1;
    let (cm_i, wit, ccs, scheme) = wit_and_ccs_gen::<X_LEN, C, WIT_LEN, W, P, R>(r1cs_rows);

    prover_decomposition_benchmark::<C, W, P, R, CS>(group, &cm_i, &wit, &ccs, &scheme);

    verifier_decomposition_benchmark::<C, W, P, R, CS>(group, &cm_i, &wit, &ccs, &scheme);
}

fn decomposition_benchmarks_non_scalar<
    const X_LEN: usize,
    const C: usize,
    const WIT_LEN: usize,
    const W: usize,
    CS: LatticefoldChallengeSet<R> + Clone,
    R: Clone + UniformRand + Debug + SuitableRing,
    P: DecompositionParams + Clone,
>(
    group: &mut criterion::BenchmarkGroup<criterion::measurement::WallTime>,
) {
    let r1cs_rows = X_LEN + WIT_LEN + 1;
    let (cm_i, wit, ccs, scheme) =
        wit_and_ccs_gen_non_scalar::<X_LEN, C, WIT_LEN, W, P, R>(r1cs_rows);

    prover_decomposition_benchmark::<C, W, P, R, CS>(group, &cm_i, &wit, &ccs, &scheme);

    verifier_decomposition_benchmark::<C, W, P, R, CS>(group, &cm_i, &wit, &ccs, &scheme);
}

// Macros
macro_rules! define_params {
    ($w:expr, $b:expr, $l:expr, $b_small:expr, $k:expr) => {
        paste::paste! {

            #[derive(Clone)]
            struct [<DecompParamsWithB $b W $w b $b_small K $k>];

            impl DecompositionParams for [<DecompParamsWithB $b W $w b $b_small K $k>] {
                const B: u128 = $b;
                const L: usize = $l;
                const B_SMALL: usize = $b_small;
                const K: usize = $k;
            }
        }
    };
}

macro_rules! run_single_starkprime_benchmark {
    ($crit:expr, $io:expr, $cw:expr, $w:expr, $b:expr, $l:expr, $b_small:expr, $k:expr) => {
        define_params!($w, $b, $l, $b_small, $k);
        paste::paste! {
            decomposition_benchmarks_scalar::<$io, $cw, $w,{$w * $l}, StarkChallengeSet, StarkRingNTT, [<DecompParamsWithB $b W $w b $b_small K $k>]>($crit);
        }
    };
}

macro_rules! run_single_starkprime_non_scalar_benchmark {
    ($crit:expr, $io:expr, $cw:expr, $w:expr, $b:expr, $l:expr, $b_small:expr, $k:expr) => {
        define_params!($w, $b, $l, $b_small, $k);
        paste::paste! {
            decomposition_benchmarks_non_scalar::<$io, $cw, $w,{$w * $l}, StarkChallengeSet, StarkRingNTT, [<DecompParamsWithB $b W $w b $b_small K $k>]>($crit);
        }
    };
}

#[macro_export]
macro_rules! run_single_goldilocks_benchmark {
    ($crit:expr, $io:expr, $cw:expr, $w:expr, $b:expr, $l:expr, $b_small:expr, $k:expr) => {
         define_params!($w, $b, $l, $b_small, $k);
        paste::paste! {
            decomposition_benchmarks_scalar::<$io, $cw, $w, {$w * $l}, GoldilocksChallengeSet, GoldilocksRingNTT, [<DecompParamsWithB $b W $w b $b_small K $k>]>($crit);

        }
    };
}

#[macro_export]
macro_rules! run_single_goldilocks_non_scalar_benchmark {
    ($crit:expr, $io:expr, $cw:expr, $w:expr, $b:expr, $l:expr, $b_small:expr, $k:expr) => {
         define_params!($w, $b, $l, $b_small, $k);
        paste::paste! {
            decomposition_benchmarks_non_scalar::<$io, $cw, $w, {$w * $l}, GoldilocksChallengeSet, GoldilocksRingNTT, [<DecompParamsWithB $b W $w b $b_small K $k>]>($crit);

        }
    };
}
#[macro_export]
macro_rules! run_single_babybear_benchmark {
    ($crit:expr, $io:expr, $cw:expr, $w:expr, $b:expr, $l:expr, $b_small:expr, $k:expr) => {
        define_params!($w, $b, $l, $b_small, $k);
        paste::paste! {
            decomposition_benchmarks::<$io, $cw, $w, {$w * $l}, BabyBearChallengeSet, BabyBearRingNTT, [<DecompParamsWithB $b W $w b $b_small K $k>]>($crit);

        }
    };
}
#[macro_export]
macro_rules! run_single_frog_benchmark {
    ($crit:expr, $io:expr, $cw:expr, $w:expr, $b:expr, $l:expr, $b_small:expr, $k:expr) => {
        define_params!($w, $b, $l, $b_small, $k);
        paste::paste! {
            decomposition_benchmarks::<$io, $cw, $w, {$w * $l}, FrogChallengeSet, FrogRingNTT, [<DecompParamsWithB $b W $w b $b_small K $k>]>($crit);

        }
    };
}

fn benchmarks_main(c: &mut Criterion) {
    // Goldilocks
    {
        let plot_config = PlotConfiguration::default().summary_scale(AxisScale::Logarithmic);
        let mut group = c.benchmark_group("Decomposition Godlilocks");
        group.plot_config(plot_config.clone());
        #[allow(clippy::identity_op)]
        {
            run_goldilocks_benchmarks!(group);
        }
    }
    // Goldilocks non scalar
    {
        let plot_config = PlotConfiguration::default().summary_scale(AxisScale::Logarithmic);
        let mut group = c.benchmark_group("Linearization Goldilocks non scalar");
        group.plot_config(plot_config.clone());

        run_goldilocks_non_scalar_benchmarks!(group);
    }
    // StarkPrime
    {
        let plot_config = PlotConfiguration::default().summary_scale(AxisScale::Logarithmic);
        let mut group = c.benchmark_group("Decomposition StarkPrime");
        group.plot_config(plot_config.clone());
        #[allow(clippy::identity_op)]
        {
            run_starkprime_benchmarks!(group);
        }
    }
    // StarkPrime non scalar
    {
        let plot_config = PlotConfiguration::default().summary_scale(AxisScale::Logarithmic);
        let mut group = c.benchmark_group("Decomposition StarkPrime non scalar");
        group.plot_config(plot_config.clone());

        run_starkprime_non_scalar_benchmarks!(group);
    }
}

criterion_group!(
    name=benches;
    config = Criterion::default().sample_size(10).measurement_time(Duration::from_secs(50)).warm_up_time(Duration::from_secs(1));
    targets = benchmarks_main);
criterion_main!(benches);
