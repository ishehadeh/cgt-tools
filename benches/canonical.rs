use cgt::{
    grid::small_bit_grid::SmallBitGrid,
    numeric::{dyadic_rational_number::DyadicRationalNumber, nimber::Nimber},
    short::partizan::{
        canonical_form::{CanonicalForm, Moves, Nus},
        games::domineering::Domineering,
        partizan_game::PartizanGame,
        transposition_table::NoTranspositionTable,
    },
};
use criterion::{black_box, criterion_group, criterion_main, Criterion};

mod perf;

fn make_canonical(c: &mut Criterion) {
    c.bench_function("Moves::canonical_form(), 4/4 nus", |b| {
        let dy = |n: i64, d: u32| DyadicRationalNumber::new(n, d);
        let star = |n: u32| Nimber::new(n);

        let moves = Moves {
            left: vec![
                CanonicalForm::new_integer(3),
                CanonicalForm::new_integer(-1),
                CanonicalForm::new_nimber(dy(0, 0), star(3)),
                CanonicalForm::new_nus(Nus::new(dy(0, 0), 3, star(0))),
            ],
            right: vec![
                CanonicalForm::new_integer(3),
                CanonicalForm::new_integer(-1),
                CanonicalForm::new_nimber(dy(0, 0), star(3)),
                CanonicalForm::new_nus(Nus::new(dy(0, 0), 3, star(0))),
            ],
        };
        b.iter(|| {
            let canon = moves.clone().canonical_form();
            black_box(canon);
        });
    });
}

fn domineering(c: &mut Criterion) {
    c.bench_function(
        "PartizanGame::canonical_form(), NoTranspositionTable, domineering_3x3",
        |b| {
            let board = Domineering::new(SmallBitGrid::empty(3, 3).unwrap());
            let tt = NoTranspositionTable::new();
            b.iter(|| {
                let canon = board.clone().canonical_form(&tt);
                black_box(canon);
            });
        },
    );

    let mut group = c.benchmark_group("long-running");
    group.sample_size(10);
    group
        .bench_function(
            "PartizanGame::canonical_form(), NoTranspositionTable, domineering_5x5",
            |b| {
                let board = Domineering::new(SmallBitGrid::empty(4, 4).unwrap());
                let tt = NoTranspositionTable::new();
                b.iter(|| {
                    let canon = board.clone().canonical_form(&tt);
                    black_box(canon);
                });
            },
        )
        .sample_size(10);
}

criterion_group!(
    name = canonicalize;
    config = Criterion::default().with_profiler(perf::FlamegraphProfiler::new(100));
    targets = domineering
);
criterion_main!(canonicalize);
