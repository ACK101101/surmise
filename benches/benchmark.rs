use criterion::{Criterion, criterion_group, criterion_main};
use image::{Rgb, RgbImage};
use std::time::Duration;

use surmise::config::{DEFAULT_WINDOW_HEIGHT, DEFAULT_WINDOW_WIDTH, SMA_WINDOW_SIZE};
use surmise::geometry::{Point, Rect};
use surmise::transform::color::ColorMode;
use surmise::transform::cuboid::TileCuboid;
use surmise::transform::effect::EffectMode;
use surmise::transform::rbg_image_to_u32;
use surmise::window::lens::Lens;

fn color_tile_bench(c: &mut Criterion) {
    let color = ColorMode::Default;
    let image = RgbImage::new(1920, 1080);
    let top_left = Point { x: 0, y: 0 };
    let source_chunk_matrix = Rect::new(64, 32);
    c.bench_function("color_tile 1920x1080", |b| {
        b.iter(|| color.color_tile(&image, top_left, source_chunk_matrix))
    });
}

fn pixel_lattice_sma_bench(c: &mut Criterion) {
    let mut lattice = TileCuboid::new(DEFAULT_WINDOW_WIDTH, DEFAULT_WINDOW_HEIGHT, SMA_WINDOW_SIZE);
    let pixel = Rgb([1, 1, 1]);
    c.bench_function("sma 60x67", |b| b.iter(|| lattice.sma(pixel, 1, 1)));
}

fn downsample_bench(c: &mut Criterion) {
    let mut lens = Lens::new();
    let image = RgbImage::new(1920, 1080);
    let origin = Point { x: 0, y: 0 };
    let pixel_chunk_matrix = Rect::new(30, 33);
    let source_chunk_matrix = Rect::new(64, 32);

    c.bench_function("downsample 1920x1080", |b| {
        b.iter(|| lens.downsample(&image, origin, pixel_chunk_matrix, source_chunk_matrix))
    });
}

fn rgb_image_to_u32_bench(c: &mut Criterion) {
    let image = RgbImage::new(1920, 1080);
    let mut v = Vec::new();
    c.bench_function("rbg_image_to_u32 1920x1080", |b| {
        b.iter(|| rbg_image_to_u32(&image, &mut v, ColorMode::Default))
    });
}

fn calculate_frame_bench(c: &mut Criterion) {
    let modes = [EffectMode::Default, EffectMode::Reveal, EffectMode::Sma];
    let image = RgbImage::new(1920, 1080);

    let mut group = c.benchmark_group("pipeline");
    group.sample_size(50);

    for mode in modes {
        group.bench_function(format!("{mode}"), |b| {
            let mut lens = Lens::new();

            b.iter(|| lens.calculate_and_save_frame(&image));
        });
    }

    group.finish();
}

criterion_group!(
    name = benches;
    config = Criterion::default().measurement_time(Duration::from_secs(10));
    targets = color_tile_bench,
    pixel_lattice_sma_bench,
    downsample_bench,
    rgb_image_to_u32_bench,
    calculate_frame_bench,
);
criterion_main!(benches);
