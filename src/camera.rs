use anyhow::{Context, Result};
use arc_swap::ArcSwap;
use image::RgbImage;
use nokhwa::{Camera, pixel_format::RgbFormat, utils::*};
use std::sync::Arc;

use crate::config::{DEFAULT_CAMERA_HEIGHT, DEFAULT_CAMERA_WIDTH};
use crate::transform::reflect_y;

pub struct Cam {
    camera: Camera,
    scratch: RgbImage,
    frame: ArcSwap<RgbImage>,
}

impl Cam {
    pub fn new() -> Result<Cam> {
        let camera_index = CameraIndex::Index(0);
        let requested_format =
            RequestedFormat::new::<RgbFormat>(RequestedFormatType::AbsoluteHighestFrameRate);

        // my camera is 1920 x 1080, 30fps
        let mut camera = Camera::new(camera_index, requested_format)?;

        // tries to open camera stream
        camera.open_stream()?;

        Ok(Cam {
            camera,
            scratch: RgbImage::new(DEFAULT_CAMERA_WIDTH as u32, DEFAULT_CAMERA_HEIGHT as u32),
            frame: ArcSwap::from_pointee(RgbImage::new(
                DEFAULT_CAMERA_WIDTH as u32,
                DEFAULT_CAMERA_HEIGHT as u32,
            )),
        })
    }

    pub fn load_next_frame(&mut self) -> Result<()> {
        self.camera
            .write_frame_to_buffer::<RgbFormat>(self.scratch.as_mut())
            .context("Sum fucked up with getting a frame")?;

        reflect_y(&mut self.scratch);

        self.frame.store(Arc::new(self.scratch.clone()));

        Ok(())
    }

    pub fn get_frame(&self) -> Arc<RgbImage> {
        self.frame.load_full()
    }
}
