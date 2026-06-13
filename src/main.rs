mod window;
use window::*;

mod camera;
use camera::*;

mod config;

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

    // let wins: Vec<Win> = Vec::new();
    let mut win = match Win::new() {
        Ok(w) => w,
        Err(e) => {
            eprintln!("Win oopsie: {e}");
            return;
        }
    };
    // wins.push(win);

    // while wins.len() > 0 {

    // }

    loop {
        let new_camera_buffer = match camera.next() {
            Ok(b) => b,
            Err(e) => {
                eprintln!("Camera oopsie: {e}");
                return;
            }
        };

        let ok = win.step(new_camera_buffer);
        if !ok {
            break;
        }
    }
}
