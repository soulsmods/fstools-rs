use criterion::{criterion_group, criterion_main, Criterion};
use fstools::{
    dvdbnd::recover_keys,
    game::{GameId, GameInstallation},
    Assets,
};

pub fn open_assets_benchmark(c: &mut Criterion) {
    c.bench_function("er_vfs_open", |b| {
        let install = GameInstallation::find(GameId::ELDEN_RING).unwrap();
        let keys = recover_keys(&install.exe, &install.bhds).unwrap();

        b.iter_with_large_drop(move || {
            std::hint::black_box(Assets::open_with(install.clone(), &keys))
        });
    });
}

pub fn index_assets_benchmark(c: &mut Criterion) {
    c.bench_function("er_vfs_index", |b| {
        let install = GameInstallation::find(GameId::ELDEN_RING).unwrap();
        let keys = recover_keys(&install.exe, &install.bhds).unwrap();
        let assets = Assets::open_with(install.clone(), &keys)
            .unwrap()
            .with_dictionary(fstools_elden_ring_support::dictionary());

        b.iter_with_large_drop(move || std::hint::black_box(assets.index()));
    });
}

criterion_group!(
    name = benches;
    config = Criterion::default().sample_size(10);
    targets = open_assets_benchmark, index_assets_benchmark
);
criterion_main!(benches);
