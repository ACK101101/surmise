use anyhow::{Result, anyhow};
use nokhwa::{Camera, pixel_format::RgbFormat, utils::*};
use image::{RgbImage};

pub struct Cam {
    camera: Camera
}

impl Cam {
    pub fn new() -> Result<Cam> {
        let camera_index = CameraIndex::Index(0);
        let requested_format = RequestedFormat::new::<RgbFormat>(
            RequestedFormatType::AbsoluteHighestFrameRate,
        );

        // my camera is 1920 x 1080, 30fps
        let mut camera = Camera::new(camera_index, requested_format)?;
        
        // tries to open camera stream
        camera.open_stream()?;

        Ok(Cam { camera })
    }

    pub fn next(&mut self) -> Result<RgbImage> {
        // get a frame
        let frame = match self.camera.frame() {
            Ok(f) => f,
            Err(e) => {
                return Err(anyhow!("Sum fucked up with getting a frame: {e}"));
            }
        };

        // decode into an ImageBuffer
        let decoded = match frame.decode_image::<RgbFormat>() {
            Ok(b) => b,
            Err(e) => {
                return Err(anyhow!("Sum fucked up with decoding buffer: {e}"));
            }
        };

        Ok(decoded)
    }
}
