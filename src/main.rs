mod window;

mod windows_manager;
use windows_manager::*;

mod camera;
use camera::*;

mod config;
mod geometry;
mod transform;

fn main() {
    env_logger::init();

    let mut camera = match Cam::new() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Cam oopsie: {e}");
            return;
        }
    };

    let mut wins_man = match WindowsManager::new() {
        Ok(w) => w,
        Err(e) => {
            eprintln!("Windows oopsie: {e}");
            return;
        }
    };

    while wins_man.is_alive() {
        let mut next_frame_buf = match camera.next_frame() {
            Ok(b) => b,
            Err(e) => {
                eprintln!("Camera oopsie: {e}");
                return;
            }
        };

        wins_man.step_wins(&mut next_frame_buf);
    }
}
