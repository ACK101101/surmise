mod window;

mod windows_orchestrator;
use windows_orchestrator::*;

mod camera;
use camera::*;

mod config;
use crate::config::FRAME_SAMPLING_WINDOW;

mod geometry;
mod transform;

fn main() {
    env_logger::init();

    let cam = match Cam::new() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Cam oopsie: {e}");
            return;
        }
    };

    let mut wins_orc = match WindowsOrchestrator::new(cam) {
        Ok(w) => w,
        Err(e) => {
            eprintln!("Windows oopsie: {e}");
            return;
        }
    };

    let mut frames_processed: u64 = 0;
    let mut frame_times = std::time::Duration::new(0, 0);
    while wins_orc.is_alive() {
        let start = std::time::Instant::now();

        wins_orc.step_wins();

        frames_processed += 1;
        frame_times += start.elapsed();
        if frames_processed % FRAME_SAMPLING_WINDOW == 0 {
            eprint!(
                "\rFrame {}: {:.3}ms / frame ({} wins)",
                frames_processed,
                (frame_times.as_secs_f64() / FRAME_SAMPLING_WINDOW as f64) * 1000.0,
                wins_orc.num_open()
            );
            frame_times = std::time::Duration::new(0, 0);
        }
    }

    eprintln!("")
}
