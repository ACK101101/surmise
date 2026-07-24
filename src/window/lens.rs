use anyhow::{Result, anyhow};
use image::{Rgb, RgbImage};
use rayon::prelude::*;

use crate::config::{
    DEFAULT_CAMERA_HEIGHT, DEFAULT_CAMERA_WIDTH, DEFAULT_PIXEL_HEIGHT, DEFAULT_PIXEL_WIDTH,
    DEFAULT_WINDOW_HEIGHT, DEFAULT_WINDOW_WIDTH, SMA_WINDOW_SIZE,
};
use crate::geometry::{Point, Rect};
use crate::transform::{
    color::ColorMode, cuboid::TileCuboid, effect::EffectMode, pattern::PatternMode,
    rbg_image_to_u32,
};

pub struct Lens {
    // framebuf memory
    frame: Vec<u32>,
    scratch: Vec<u32>,

    // memory for transforms, influenced by user input
    tile: Rect,
    tile_cuboid: TileCuboid,

    // flags for what transforms to do, influenced by user input
    effect_mode: EffectMode,
    pattern_mode: PatternMode,
    color_mode: ColorMode,
}

impl Lens {
    pub fn new() -> Lens {
        Lens {
            frame: vec![0u32; (DEFAULT_CAMERA_WIDTH * DEFAULT_CAMERA_HEIGHT) as usize],
            scratch: vec![0u32; (DEFAULT_CAMERA_WIDTH * DEFAULT_CAMERA_HEIGHT) as usize],

            tile: Rect::new(DEFAULT_PIXEL_WIDTH, DEFAULT_PIXEL_HEIGHT),
            tile_cuboid: TileCuboid::new(
                DEFAULT_WINDOW_WIDTH,
                DEFAULT_WINDOW_HEIGHT,
                SMA_WINDOW_SIZE,
            ),

            effect_mode: EffectMode::Default,
            pattern_mode: PatternMode::Default,
            color_mode: ColorMode::Default,
        }
    }

    // --- Handles for input from Driver
    pub fn halve_tile_width(&mut self, win_size_snap: &Rect) {
        let (w, h) = self.tile.get_dims();
        if w > 1 {
            self.tile.resize(w / 2, h);
            self.rebuild_tile_cuboid(win_size_snap);
        }
    }

    pub fn halve_tile_height(&mut self, win_size_snap: &Rect) {
        let (w, h) = self.tile.get_dims();
        if h > 1 {
            self.tile.resize(w, h / 2);
            self.rebuild_tile_cuboid(win_size_snap);
        }
    }

    pub fn halve_tile(&mut self, win_size_snap: &Rect) {
        let (w, h) = self.tile.get_dims();
        let new_w = if w > 1 { w / 2 } else { w };
        let new_h = if h > 1 { h / 2 } else { h };
        self.tile.resize(new_w, new_h);
        self.rebuild_tile_cuboid(win_size_snap);
    }

    pub fn double_tile_width(&mut self, win_size_snap: &Rect) {
        let (tile_w, tile_h) = self.tile.get_dims();
        let win_w = win_size_snap.get_width();
        if tile_w < win_w / 2 {
            self.tile.resize(tile_w * 2, tile_h);
            self.rebuild_tile_cuboid(win_size_snap);
        }
    }

    pub fn double_tile_height(&mut self, win_size_snap: &Rect) {
        let (tile_w, tile_h) = self.tile.get_dims();
        let win_h = win_size_snap.get_height();
        if tile_h < win_h / 2 {
            self.tile.resize(tile_w, tile_h * 2);
            self.rebuild_tile_cuboid(win_size_snap);
        }
    }

    pub fn double_tile(&mut self, win_size_snap: &Rect) {
        let (tile_w, tile_h) = self.tile.get_dims();
        let (win_w, win_h) = win_size_snap.get_dims();

        let new_tile_w = if tile_w < win_w / 2 {
            tile_w * 2
        } else {
            tile_w
        };
        let new_tile_h = if tile_h < win_h / 2 {
            tile_h * 2
        } else {
            tile_h
        };

        self.tile.resize(new_tile_w, new_tile_h);
        self.rebuild_tile_cuboid(win_size_snap);
    }

    pub fn toggle_effect_mode(&mut self, win_size_snap: &Rect) {
        self.effect_mode.toggle();
        self.rebuild_tile_cuboid(win_size_snap);
    }

    pub fn toggle_color_mode(&mut self) {
        self.color_mode.toggle();
    }

    pub fn toggle_pattern_mode(&mut self) {
        self.pattern_mode.toggle();
    }

    fn rebuild_tile_cuboid(&mut self, win_size_snap: &Rect) {
        if matches!(self.effect_mode, EffectMode::Sma) {
            self.tile_cuboid = TileCuboid::new(
                win_size_snap.get_width() / self.tile.get_width(),
                win_size_snap.get_height() / self.tile.get_height(),
                SMA_WINDOW_SIZE,
            );
        }
    }

    // --- Calculate frames
    pub fn calculate_and_save_frame(
        &mut self,
        raw_image: &RgbImage,
        win_size_snap: Rect, // TODO: weird that not a ref?
        win_pos_snap: Point, // TODO: weird that not a ref?
    ) -> Result<()> {
        let (pixel_chunk_matrix, source_chunk_matrix, origin) =
            match self.calc_source_chunk_dims(raw_image, win_size_snap, win_pos_snap) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Chunking oopsie: {e}");
                    return Err(e);
                }
            };

        let downsampled = self.downsample(
            raw_image,
            origin,
            pixel_chunk_matrix,
            source_chunk_matrix,
            win_size_snap,
        );

        rbg_image_to_u32(&downsampled, &mut self.scratch, self.color_mode);
        std::mem::swap(&mut self.frame, &mut self.scratch);

        Ok(())
    }

    // TODO: refactor to only be called when image dims change, win size change, win pos change, tile change
    // TODO: break up above
    fn calc_source_chunk_dims(
        &self,
        raw_image: &RgbImage,
        win_size_snap: Rect, // TODO: weird that not a ref?
        win_pos_snap: Point, // TODO: weird that not a ref?
    ) -> Result<(Rect, Rect, Point)> {
        let source_dims = Rect::new(raw_image.width(), raw_image.height());
        if !source_dims.can_contain(&win_size_snap) {
            return Err(anyhow!(
                "Can not downsample when the source is smaller than window bruh"
            ));
        }

        // figure out how many chunky pixels fit into the target
        let pixel_chunk_matrix = win_size_snap / self.tile;

        let relevant_source_matrix: Rect = match self.effect_mode {
            EffectMode::Reveal => win_size_snap,
            _ => source_dims,
        };

        // based on matrix of chunky pixels, map source chunks to pixel chunks
        let source_chunk_matrix = relevant_source_matrix / pixel_chunk_matrix;

        // where to start processing source image
        let origin: Point = match self.effect_mode {
            EffectMode::Reveal => win_pos_snap,
            _ => Point { x: 0, y: 0 },
        };

        Ok((pixel_chunk_matrix, source_chunk_matrix, origin))
    }

    // TODO: refactor
    pub fn downsample(
        &mut self,
        source: &RgbImage,
        origin: Point,
        pixel_chunk_matrix: Rect,
        source_chunk_matrix: Rect,
        win_size_snap: Rect, // TODO: weird that not a ref?
    ) -> RgbImage {
        let (pixel_matrix_width, pixel_matrix_height) = pixel_chunk_matrix.get_dims();
        let use_cuboid = matches!(self.effect_mode, EffectMode::Sma)
            && self
                .tile_cuboid
                .use_cuboid(pixel_matrix_width, pixel_matrix_height);

        // calculate new pixelchunk values in parallel
        let averaged: Vec<Rgb<u8>> = (0..pixel_chunk_matrix.area())
            .into_par_iter()
            .map(|idx| {
                let row_i = idx / pixel_matrix_width;
                let col_i = idx % pixel_matrix_width;
                let top_left = Point {
                    x: origin.x + (col_i * source_chunk_matrix.get_width()) as i32,
                    y: origin.y + (row_i * source_chunk_matrix.get_height()) as i32,
                };

                self.color_mode
                    .color_tile(source, top_left, source_chunk_matrix)
            })
            .collect();

        let mut new_image: RgbImage =
            RgbImage::new(win_size_snap.get_width(), win_size_snap.get_height());

        for row_i in 0..pixel_matrix_height {
            for col_i in 0..pixel_matrix_width {
                let mut new_v = averaged[(row_i * pixel_matrix_width + col_i) as usize];

                if use_cuboid {
                    new_v = self.tile_cuboid.sma(new_v, row_i, col_i);
                }

                self.pattern_mode
                    .pattern_tile(&mut new_image, self.tile, new_v, col_i, row_i);
            }
        }

        if use_cuboid {
            self.tile_cuboid.bump_write_idx();
        }

        new_image
    }
}
