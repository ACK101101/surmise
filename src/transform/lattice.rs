use std::vec;

use crate::config::SMA_WINDOW_SIZE;

use image::Rgb;

pub struct PixelLattice {
    pixel_slices: Vec<Vec<Rgb<u8>>>,
    width: u32,
    height: u32,
    length: usize,
    write_idx: usize,
}

impl PixelLattice {
    pub fn new(width: u32, height: u32, length: usize) -> Self {
        Self {
            pixel_slices: vec![vec![Rgb([0, 0, 0]); (width * height) as usize]; length],
            width,
            height,
            length,
            write_idx: 0,
        }
    }

    pub fn use_memory(&self, width: u32, height: u32) -> bool {
        width == self.width && height == self.height
    }

    pub fn sma(&mut self, new_p: Rgb<u8>, chunk_r: u32, chunk_c: u32) -> Rgb<u8> {
        let p_idx = (chunk_r * self.width + chunk_c) as usize;
        self.pixel_slices[self.write_idx][p_idx] = new_p;

        let mut sum_new = Rgb::<usize>([0, 0, 0]);
        for slice in self.pixel_slices.iter() {
            let old_p = slice[p_idx];
            sum_new = Rgb([
                (old_p.0[0] as usize).saturating_add(sum_new.0[0]),
                (old_p.0[1] as usize).saturating_add(sum_new.0[1]),
                (old_p.0[2] as usize).saturating_add(sum_new.0[2]),
            ]);
        }

        Rgb([
            (sum_new.0[0].saturating_div(self.length)) as u8,
            (sum_new.0[1].saturating_div(self.length)) as u8,
            (sum_new.0[2].saturating_div(self.length)) as u8,
        ])
    }

    pub fn bump_write_idx(&mut self) {
        self.write_idx = (self.write_idx + 1) % SMA_WINDOW_SIZE;
    }
}
