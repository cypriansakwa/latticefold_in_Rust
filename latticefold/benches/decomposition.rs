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
use utils::wit_and_ccs_gen;

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
                let (_, _, _) = LFDecompositionProver::<_, PoseidonTranscript<R, CS>>::prove::<
                    W,
                    C,
                    P,
                >(lcccs, wit, &mut prover_transcript, ccs, scheme)
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

    let (_, _, decomposition_proof) =
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

fn decomposition_benchmarks<
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
            decomposition_benchmarks::<$io, $cw, $w,{$w * $l}, StarkChallengeSet, StarkRingNTT, [<DecompParamsWithB $b W $w b $b_small K $k>]>($crit);
        }
    };
}

#[macro_export]
macro_rules! run_single_goldilocks_benchmark {
    ($crit:expr, $io:expr, $cw:expr, $w:expr, $b:expr, $l:expr, $b_small:expr, $k:expr) => {
         define_params!($w, $b, $l, $b_small, $k);
        paste::paste! {
            decomposition_benchmarks::<$io, $cw, $w, {$w * $l}, GoldilocksChallengeSet, GoldilocksRingNTT, [<DecompParamsWithB $b W $w b $b_small K $k>]>($crit);

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
    // // Babybear
    // {
    //     let plot_config = PlotConfiguration::default().summary_scale(AxisScale::Logarithmic);
    //     let mut group = c.benchmark_group("Decomposition BabyBear");
    //     group.plot_config(plot_config.clone());

    //     // TODO: Update configurations
    //     run_single_babybear_benchmark!(1, &mut group, 6, 1024, 10, 2);
    // }

    // Godlilocks
    {
        let plot_config = PlotConfiguration::default().summary_scale(AxisScale::Logarithmic);
        let mut group = c.benchmark_group("Decomposition Godlilocks");
        group.plot_config(plot_config.clone());

        // Parameters Criterion, X_LEN, C, W, B, L, B_small, K
        run_single_goldilocks_benchmark!(&mut group, 1, 6, 512, 120, 9, 2, 7);
        run_single_goldilocks_benchmark!(&mut group, 1, 7, 512, 256, 8, 2, 8);
        run_single_goldilocks_benchmark!(&mut group, 1, 7, 512, 256, 8, 4, 4);
        run_single_goldilocks_benchmark!(&mut group, 1, 8, 512, 512, 7, 2, 9);
        run_single_goldilocks_benchmark!(&mut group, 1, 8, 1024, 512, 7, 2, 9);
        run_single_goldilocks_benchmark!(&mut group, 1, 8, 2048, 256, 8, 2, 8);
        run_single_goldilocks_benchmark!(&mut group, 1, 9, 1024, 1024, 7, 2, 10);
        run_single_goldilocks_benchmark!(&mut group, 1, 9, 2048, 512, 7, 2, 9);
        run_single_goldilocks_benchmark!(&mut group, 1, 10, 512, 2048, 6, 2, 11);
        run_single_goldilocks_benchmark!(&mut group, 1, 10, 1024, 2048, 6, 2, 11);
        run_single_goldilocks_benchmark!(&mut group, 1, 11, 1024, 4096, 6, 2, 12);
        run_single_goldilocks_benchmark!(&mut group, 1, 11, 2048, 2048, 6, 2, 12);
        run_single_goldilocks_benchmark!(&mut group, 1, 12, 1024, 8192, 6, 2, 13);
        run_single_goldilocks_benchmark!(&mut group, 1, 13, 1024, 16384, 5, 2, 14);
        run_single_goldilocks_benchmark!(&mut group, 1, 13, 2048, 8192, 5, 2, 13);
        run_single_goldilocks_benchmark!(&mut group, 1, 14, 1024, 32768, 5, 2, 15);
        run_single_goldilocks_benchmark!(&mut group, 1, 14, 2048, 16384, 5, 2, 14);
        run_single_goldilocks_benchmark!(&mut group, 1, 15, 2048, 32768, 4, 2, 15);
        run_single_goldilocks_benchmark!(&mut group, 1, 16, 2048, 65536, 4, 2, 16);
    }

    // StarkPrime
    {
        let plot_config = PlotConfiguration::default().summary_scale(AxisScale::Logarithmic);
        let mut group = c.benchmark_group("Decomposition StarkPrime");
        group.plot_config(plot_config.clone());

        // Parameters Criterion, X_LEN, C, W, B, L, B_small, K
        #[allow(clippy::identity_op)]
        {
            run_single_starkprime_benchmark!(&mut group, 1, 15, 512, 8633754724, 1, 92918, 2);
            run_single_starkprime_benchmark!(&mut group, 1, 15, 512, 8615125000, 1, 2050, 3);
            run_single_starkprime_benchmark!(&mut group, 1, 15, 512, 8540717056, 1, 304, 4);
            run_single_starkprime_benchmark!(&mut group, 1, 15, 1024, 6104921956, 1, 78134, 2);
            run_single_starkprime_benchmark!(&mut group, 1, 15, 1024, 6088387976, 1, 1826, 3);
            run_single_starkprime_benchmark!(&mut group, 1, 15, 1024, 5972816656, 1, 278, 4);
            run_single_starkprime_benchmark!(&mut group, 1, 15, 2048, 4317015616, 1, 65704, 2);
            run_single_starkprime_benchmark!(&mut group, 1, 15, 2048, 4314825152, 1, 1628, 3);
            run_single_starkprime_benchmark!(&mut group, 1, 15, 2048, 4294967296, 1, 256, 4);
            run_single_starkprime_benchmark!(&mut group, 1, 16, 512, 21195283396, 1, 145586, 2);
            run_single_starkprime_benchmark!(&mut group, 1, 16, 512, 21161991096, 1, 2766, 3);
            run_single_starkprime_benchmark!(&mut group, 1, 16, 512, 20851360000, 1, 380, 4);
            run_single_starkprime_benchmark!(&mut group, 1, 16, 1024, 14987635776, 1, 122424, 2);
            run_single_starkprime_benchmark!(&mut group, 1, 16, 1024, 14959673344, 1, 2464, 3);
            run_single_starkprime_benchmark!(&mut group, 1, 16, 1024, 14666178816, 1, 348, 4);
            run_single_starkprime_benchmark!(&mut group, 1, 16, 2048, 10597878916, 1, 102946, 2);
            run_single_starkprime_benchmark!(&mut group, 1, 16, 2048, 10590025536, 1, 2196, 3);
            run_single_starkprime_benchmark!(&mut group, 1, 16, 2048, 10485760000, 1, 320, 4);
            run_single_starkprime_benchmark!(&mut group, 1, 17, 512, 50614200576, 1, 224976, 2);
            run_single_starkprime_benchmark!(&mut group, 1, 17, 512, 50570904392, 1, 3698, 3);
            run_single_starkprime_benchmark!(&mut group, 1, 17, 512, 50479304976, 1, 474, 4);
            run_single_starkprime_benchmark!(&mut group, 1, 17, 1024, 35789072400, 1, 189180, 2);
            run_single_starkprime_benchmark!(&mut group, 1, 17, 1024, 35741336184, 1, 3294, 3);
            run_single_starkprime_benchmark!(&mut group, 1, 17, 1024, 35477982736, 1, 434, 4);
            run_single_starkprime_benchmark!(&mut group, 1, 17, 2048, 25307082724, 1, 159082, 2);
            run_single_starkprime_benchmark!(&mut group, 1, 17, 2048, 25256916504, 1, 2934, 3);
            run_single_starkprime_benchmark!(&mut group, 1, 17, 2048, 25091827216, 1, 398, 4);
            run_single_starkprime_benchmark!(&mut group, 1, 18, 512, 117850770436, 1, 343294, 2);
            run_single_starkprime_benchmark!(&mut group, 1, 18, 512, 117793118808, 1, 4902, 3);
            run_single_starkprime_benchmark!(&mut group, 1, 18, 512, 116319195136, 1, 584, 4);
            run_single_starkprime_benchmark!(&mut group, 1, 18, 1024, 83332678276, 1, 288674, 2);
            run_single_starkprime_benchmark!(&mut group, 1, 18, 1024, 83224499896, 1, 4366, 3);
            run_single_starkprime_benchmark!(&mut group, 1, 18, 1024, 82538991616, 1, 536, 4);
            run_single_starkprime_benchmark!(&mut group, 1, 18, 2048, 58925620516, 1, 242746, 2);
            run_single_starkprime_benchmark!(&mut group, 1, 18, 2048, 58863869000, 1, 3890, 3);
            run_single_starkprime_benchmark!(&mut group, 1, 18, 2048, 58594980096, 1, 492, 4);
            run_single_starkprime_benchmark!(&mut group, 1, 19, 512, 268120982416, 1, 517804, 2);
            run_single_starkprime_benchmark!(&mut group, 1, 19, 512, 268086587392, 1, 6448, 3);
            run_single_starkprime_benchmark!(&mut group, 1, 19, 512, 265764994576, 1, 718, 4);
            run_single_starkprime_benchmark!(&mut group, 1, 19, 1024, 189590576400, 1, 435420, 2);
            run_single_starkprime_benchmark!(&mut group, 1, 19, 1024, 189514870784, 1, 5744, 3);
            run_single_starkprime_benchmark!(&mut group, 1, 19, 1024, 187457825296, 1, 658, 4);
            run_single_starkprime_benchmark!(&mut group, 1, 19, 2048, 134059964164, 1, 366142, 2);
            run_single_starkprime_benchmark!(&mut group, 1, 19, 2048, 134060503032, 1, 5118, 3);
            run_single_starkprime_benchmark!(&mut group, 1, 19, 2048, 133090713856, 1, 604, 4);
            run_single_starkprime_benchmark!(&mut group, 1, 20, 512, 597108561984, 1, 772728, 2);
            run_single_starkprime_benchmark!(&mut group, 1, 20, 512, 596947688000, 1, 8420, 3);
            run_single_starkprime_benchmark!(&mut group, 1, 20, 512, 594262141456, 1, 878, 4);
            run_single_starkprime_benchmark!(&mut group, 1, 20, 1024, 422219246656, 1, 649784, 2);
            run_single_starkprime_benchmark!(&mut group, 1, 20, 1024, 422212590008, 1, 7502, 3);
            run_single_starkprime_benchmark!(&mut group, 1, 20, 1024, 422026932496, 1, 806, 4);
            run_single_starkprime_benchmark!(&mut group, 1, 20, 2048, 298552960000, 1, 546400, 2);
            run_single_starkprime_benchmark!(&mut group, 1, 20, 2048, 298345446568, 1, 6682, 3);
            run_single_starkprime_benchmark!(&mut group, 1, 20, 2048, 296637086736, 1, 738, 4);
            run_single_starkprime_benchmark!(&mut group, 1, 21, 512, 1303720941636, 1, 1141806, 2);
            run_single_starkprime_benchmark!(&mut group, 1, 21, 512, 1303602169024, 1, 10924, 3);
            run_single_starkprime_benchmark!(&mut group, 1, 21, 512, 1301023109376, 1, 1068, 4);
            run_single_starkprime_benchmark!(&mut group, 1, 21, 1024, 921868819600, 1, 960140, 2);
            run_single_starkprime_benchmark!(&mut group, 1, 21, 1024, 921735471168, 1, 9732, 3);
            run_single_starkprime_benchmark!(&mut group, 1, 21, 1024, 914861642256, 1, 978, 4);
            run_single_starkprime_benchmark!(&mut group, 1, 21, 2048, 651859234884, 1, 807378, 2);
            run_single_starkprime_benchmark!(&mut group, 1, 21, 2048, 651714363000, 1, 8670, 3);
            run_single_starkprime_benchmark!(&mut group, 1, 21, 2048, 650287411216, 1, 898, 4);
            run_single_starkprime_benchmark!(&mut group, 1, 22, 512, 2794687879824, 1, 1671732, 2);
            run_single_starkprime_benchmark!(&mut group, 1, 22, 512, 2793688944704, 1, 14084, 3);
            run_single_starkprime_benchmark!(&mut group, 1, 22, 512, 2786442301696, 1, 1292, 4);
            run_single_starkprime_benchmark!(&mut group, 1, 22, 1024, 1976138685504, 1, 1405752, 2);
            run_single_starkprime_benchmark!(&mut group, 1, 22, 1024, 1975711510592, 1, 12548, 3);
            run_single_starkprime_benchmark!(&mut group, 1, 22, 1024, 1965200244736, 1, 1184, 4);
            run_single_starkprime_benchmark!(&mut group, 1, 22, 2048, 1397341496464, 1, 1182092, 2);
            run_single_starkprime_benchmark!(&mut group, 1, 22, 2048, 1396665211752, 1, 11178, 3);
            run_single_starkprime_benchmark!(&mut group, 1, 22, 2048, 1390974924816, 1, 1086, 4);
            run_single_starkprime_benchmark!(&mut group, 1, 23, 512, 5888940837796, 1, 2426714, 2);
            run_single_starkprime_benchmark!(&mut group, 1, 23, 512, 5888557851112, 1, 18058, 3);
            run_single_starkprime_benchmark!(&mut group, 1, 23, 512, 5861899530496, 1, 1556, 4);
            run_single_starkprime_benchmark!(&mut group, 1, 23, 1024, 4164105496996, 1, 2040614, 2);
            run_single_starkprime_benchmark!(&mut group, 1, 23, 1024, 4163956393472, 1, 16088, 3);
            run_single_starkprime_benchmark!(&mut group, 1, 23, 1024, 4158271385856, 1, 1428, 4);
            run_single_starkprime_benchmark!(&mut group, 1, 23, 2048, 2944470674916, 1, 1715946, 2);
            run_single_starkprime_benchmark!(&mut group, 1, 23, 2048, 2943882002368, 1, 14332, 3);
            run_single_starkprime_benchmark!(&mut group, 1, 23, 2048, 2927055626496, 1, 1308, 4);
            run_single_starkprime_benchmark!(&mut group, 1, 24, 512, 12211739920900, 1, 3494530, 2);
            run_single_starkprime_benchmark!(&mut group, 1, 24, 512, 12211490117952, 1, 23028, 3);
            run_single_starkprime_benchmark!(&mut group, 1, 24, 512, 12176079851776, 1, 1868, 4);
            run_single_starkprime_benchmark!(&mut group, 1, 24, 1024, 8635005577444, 1, 2938538, 2);
            run_single_starkprime_benchmark!(&mut group, 1, 24, 1024, 8632787556744, 1, 20514, 3);
            run_single_starkprime_benchmark!(&mut group, 1, 24, 1024, 8630645337616, 1, 1714, 4);
            run_single_starkprime_benchmark!(&mut group, 1, 24, 2048, 6105870652036, 1, 2471006, 2);
            run_single_starkprime_benchmark!(&mut group, 1, 24, 2048, 6104406528576, 1, 18276, 3);
            run_single_starkprime_benchmark!(&mut group, 1, 24, 2048, 6075732010000, 1, 1570, 4);
            run_single_starkprime_benchmark!(&mut group, 1, 25, 512, 24945070206016, 1, 4994504, 2);
            run_single_starkprime_benchmark!(&mut group, 1, 25, 512, 24943158948232, 1, 29218, 3);
            run_single_starkprime_benchmark!(&mut group, 1, 25, 512, 24907645451536, 1, 2234, 4);
            run_single_starkprime_benchmark!(
                &mut group,
                1,
                25,
                1024,
                17638824019600,
                1,
                4199860,
                2
            );
            run_single_starkprime_benchmark!(&mut group, 1, 25, 1024, 17636910227000, 1, 26030, 3);
            run_single_starkprime_benchmark!(&mut group, 1, 25, 1024, 17592186044416, 1, 16, 11);
            run_single_starkprime_benchmark!(
                &mut group,
                1,
                25,
                2048,
                12472537595904,
                1,
                3531648,
                2
            );
            run_single_starkprime_benchmark!(&mut group, 1, 25, 2048, 12471027759000, 1, 23190, 3);
            run_single_starkprime_benchmark!(&mut group, 1, 25, 2048, 12438910749456, 1, 1878, 4);
        }
    }
    // // Frog
    // {
    //     let plot_config = PlotConfiguration::default().summary_scale(AxisScale::Logarithmic);
    //     let mut group = c.benchmark_group("Decomposition Frog");
    //     group.plot_config(plot_config.clone());

    //     // TODO: Update configurations
    //     run_single_frog_benchmark!(1, &mut group, 6, 1024, 10, 2);
    // }
}

criterion_group!(
    name=benches;
    config = Criterion::default().sample_size(10).measurement_time(Duration::from_secs(50)).warm_up_time(Duration::from_secs(1));
    targets = benchmarks_main);
criterion_main!(benches);
