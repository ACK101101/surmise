use criterion::{Criterion, criterion_group, criterion_main};
use image::{Rgb, RgbImage};
use std::hint::black_box;
use surmise::config::SMA_WINDOW_SIZE;
use surmise::transform::lattice::PixelLattice;
use surmise::transform::reflect_y;

fn reflect_y_bench(c: &mut Criterion) {
    let mut image = RgbImage::new(1920, 1080);
    c.bench_function("reflect_y 1920x1080", |b| b.iter(|| reflect_y(&mut image)));
}

fn new_pixel_lattice_bench(c: &mut Criterion) {
    c.bench_function("PixelLattice::new 60x67", |b| {
        b.iter(|| PixelLattice::new(60, 67, SMA_WINDOW_SIZE))
    });
}

fn pixel_lattice_sma_bench(c: &mut Criterion) {
    let mut lattice = PixelLattice::new(60, 67, SMA_WINDOW_SIZE);
    let pixel = Rgb([1, 1, 1]);
    c.bench_function("sma 60x67", |b| b.iter(|| lattice.sma(pixel, 1, 1)));
}

criterion_group!(
    benches,
    reflect_y_bench,
    new_pixel_lattice_bench,
    pixel_lattice_sma_bench
);
criterion_main!(benches);
