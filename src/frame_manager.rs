use anyhow::{Context, Result};
use arc_swap::ArcSwap;
use image::RgbImage;
use nokhwa::{Camera, pixel_format::RgbFormat, utils::*};
use std::sync::Arc;
use std::thread;

use crate::config::{DEFAULT_CAMERA_HEIGHT, DEFAULT_CAMERA_WIDTH};

pub struct FrameManager {
    frame: Arc<ArcSwap<RgbImage>>,
}

impl FrameManager {
    pub fn spawn() -> Result<FrameManager> {
        let frame = Arc::new(ArcSwap::from_pointee(RgbImage::new(
            DEFAULT_CAMERA_WIDTH,
            DEFAULT_CAMERA_HEIGHT,
        )));
        let frame_for_thread = Arc::clone(&frame);

        thread::spawn(move || -> Result<()> {
            let camera_index = CameraIndex::Index(0);
            let requested_format =
                RequestedFormat::new::<RgbFormat>(RequestedFormatType::AbsoluteHighestFrameRate);

            // my camera is 1920 x 1080, 30fps
            let mut camera = Camera::new(camera_index, requested_format)?;

            // tries to open camera stream
            camera.open_stream()?;

            let mut scratch =
                RgbImage::new(DEFAULT_CAMERA_WIDTH, DEFAULT_CAMERA_HEIGHT );
            loop {
                camera
                    .write_frame_to_buffer::<RgbFormat>(scratch.as_mut())
                    .context("Sum fucked up with getting a frame")?;

                frame_for_thread.store(Arc::new(scratch.clone()));
            }
        });

        Ok(FrameManager { frame })
    }

    pub fn get_frame(&self) -> Arc<RgbImage> {
        self.frame.load_full()
    }
}
