use anyhow::{Context, Error, Result};
use image::RgbImage;
use nokhwa::{Camera, pixel_format::RgbFormat, utils::*};

use crate::config::{DEFAULT_CAMERA_HEIGHT, DEFAULT_CAMERA_WIDTH};

pub struct Cam {
    camera: Camera,
    frame: RgbImage,
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
            frame: RgbImage::new(DEFAULT_CAMERA_WIDTH as u32, DEFAULT_CAMERA_HEIGHT as u32),
        })
    }

    pub fn next_frame(&mut self) -> Result<&mut RgbImage> {
        self.camera
            .write_frame_to_buffer::<RgbFormat>(self.frame.as_mut())
            .context("Sum fucked up with getting a frame")?;

        Ok(&mut self.frame)
    }
}
