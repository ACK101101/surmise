use std::time::Duration;

use criterion::{Criterion, criterion_group, criterion_main};
use image::{Rgb, RgbImage};
use surmise::config::{DEFAULT_WINDOW_HEIGHT, DEFAULT_WINDOW_WIDTH, SMA_WINDOW_SIZE};
use surmise::geometry::{Point, Rect};
use surmise::transform::average;
use surmise::transform::lattice::PixelLattice;
use surmise::transform::{downsample, rbg_image_to_u32, reflect_y};
use surmise::window::{EffectMode, WinState};

fn average_bench(c: &mut Criterion) {
    let image = RgbImage::new(1920, 1080);
    let top_left = Point { x: 0, y: 0 };
    let source_chunk_matrix = Rect::new(64, 32);
    c.bench_function("average 1920x1080", |b| {
        b.iter(|| average(&image, top_left, source_chunk_matrix))
    });
}

fn pixel_lattice_sma_bench(c: &mut Criterion) {
    let mut lattice =
        PixelLattice::new(DEFAULT_WINDOW_WIDTH, DEFAULT_WINDOW_HEIGHT, SMA_WINDOW_SIZE);
    let pixel = Rgb([1, 1, 1]);
    c.bench_function("sma 60x67", |b| b.iter(|| lattice.sma(pixel, 1, 1)));
}

fn downsample_bench(c: &mut Criterion) {
    let image = RgbImage::new(1920, 1080);
    let origin = Point { x: 0, y: 0 };
    let window_dims = Rect::new(DEFAULT_WINDOW_WIDTH as u32, DEFAULT_WINDOW_HEIGHT as u32);
    let pixel_dims = Rect::new(32, 16);
    let pixel_chunk_matrix = Rect::new(30, 33);
    let source_chunk_matrix = Rect::new(64, 32);
    let mut memory =
        PixelLattice::new(DEFAULT_WINDOW_WIDTH, DEFAULT_WINDOW_HEIGHT, SMA_WINDOW_SIZE);
    c.bench_function("downsample 1920x1080", |b| {
        b.iter(|| {
            downsample(
                &image,
                origin,
                window_dims,
                pixel_dims,
                pixel_chunk_matrix,
                source_chunk_matrix,
                EffectMode::Default,
                &mut memory,
            )
        })
    });
}

fn rgb_image_to_u32_bench(c: &mut Criterion) {
    let image = RgbImage::new(1920, 1080);
    let mut v = Vec::new();
    c.bench_function("rbg_image_to_u32 1920x1080", |b| {
        b.iter(|| rbg_image_to_u32(&image, &mut v))
    });
}

fn reflect_y_bench(c: &mut Criterion) {
    let mut image = RgbImage::new(1920, 1080);
    c.bench_function("reflect_y 1920x1080", |b| b.iter(|| reflect_y(&mut image)));
}

fn calculate_frame_bench(c: &mut Criterion) {
    let modes = [EffectMode::Default, EffectMode::Reveal, EffectMode::Sma];
    let image = RgbImage::new(1920, 1080);

    let mut group = c.benchmark_group("pipeline");
    group.sample_size(50);

    for mode in modes {
        group.bench_function(format!("{mode}"), |b| {
            let mut win_state = WinState::new(mode);

            b.iter(|| win_state.calculate_and_save_frame(&image));
        });
    }

    group.finish();
}

criterion_group!(
    name = benches;
    config = Criterion::default().measurement_time(Duration::from_secs(10));
    targets = average_bench,
    pixel_lattice_sma_bench,
    downsample_bench,
    rgb_image_to_u32_bench,
    reflect_y_bench,
    calculate_frame_bench,
);
criterion_main!(benches);
