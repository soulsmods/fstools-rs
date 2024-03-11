use std::path::PathBuf;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use fstools::dvdbnd::DvdBnd;
use fstools_dvdbnd::FileKeyProvider;

pub fn vfs_open_benchmark(c: &mut Criterion) {
    c.bench_function("er_vfs_open", |b| {
        b.iter_with_large_drop(|| {
            let er_path = PathBuf::from(std::env::var("ER_PATH").expect("er_path"));
            let keys_path = PathBuf::from(std::env::var("ER_KEYS_PATH").expect("er_keys_path"));
            let keys = FileKeyProvider::new(keys_path);
            let archives = [
                er_path.join("Data0"),
                er_path.join("Data1"),
                er_path.join("Data2"),
                er_path.join("Data3"),
                er_path.join("sd/sd"),
            ];

            let vfs = DvdBnd::create(archives.clone(), &keys).expect("unable to create vfs");

            black_box(vfs)
        })
    });
}

criterion_group!(
    name = benches;
    config = Criterion::default().sample_size(20);
    targets = vfs_open_benchmark
);
criterion_main!(benches);
