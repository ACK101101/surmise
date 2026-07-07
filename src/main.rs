mod window;

mod windows_orchestrator;
use windows_orchestrator::*;

mod frame_manager;
use frame_manager::*;

mod config;
use crate::config::FRAME_SAMPLING_WINDOW;

mod geometry;
mod transform;

fn main() {
    env_logger::init();

    let frame_man = match FrameManager::spawn() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Cam oopsie: {e}");
            return;
        }
    };

    let mut wins_orc = match WindowsOrchestrator::new(frame_man) {
        Ok(w) => w,
        Err(e) => {
            eprintln!("Windows oopsie: {e}");
            return;
        }
    };

    let mut frames_processed: u64 = 0;
    let start = std::time::Instant::now();
    let mut last_frame_elapsed = std::time::Duration::new(0, 0);
    
    while wins_orc.is_alive() {

        wins_orc.step_wins();

        frames_processed += 1;
        if frames_processed % FRAME_SAMPLING_WINDOW == 0 {
            let time_since_start = start.elapsed();
            eprintln!(
                "Frame {}: {:.3} ms/frame \t{:.1} fps \t({} wins)",
                frames_processed,
                ((time_since_start - last_frame_elapsed).as_secs_f64() / FRAME_SAMPLING_WINDOW as f64) * 1000.0,
                frames_processed as f64 / time_since_start.as_secs_f64(),
                wins_orc.num_open(),
            );
            last_frame_elapsed = time_since_start;
        }
    }

    eprintln!("")
}
