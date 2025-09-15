use std::io::{self, Write};
use std::time::Instant;

pub struct ProgressTracker {
    total_frames: usize,
    start_time: Instant,
    last_update: Instant,
    verbose: bool,
}

impl ProgressTracker {
    pub fn new(total_frames: usize, verbose: bool) -> Self {
        let now = Instant::now();
        ProgressTracker {
            total_frames,
            start_time: now,
            last_update: now,
            verbose,
        }
    }

    pub fn update(&mut self, completed_frames: usize) {
        let now = Instant::now();

        // Only update progress every 100ms to avoid spam
        if now.duration_since(self.last_update).as_millis() < 100
            && completed_frames < self.total_frames
        {
            return;
        }

        self.last_update = now;

        let elapsed = now.duration_since(self.start_time).as_secs_f64();
        let progress = completed_frames as f64 / self.total_frames as f64;
        let fps = completed_frames as f64 / elapsed;
        let eta = if fps > 0.0 {
            (self.total_frames - completed_frames) as f64 / fps
        } else {
            0.0
        };

        if self.verbose {
            eprint!(
                "\rFrame {} of {} ({}%) {:.2} fps, eta {:.0}s",
                completed_frames,
                self.total_frames,
                (progress * 100.0) as u32,
                fps,
                eta
            );

            io::stderr().flush().unwrap();
        }
    }

    pub fn finish(&mut self) {
        let elapsed = self.start_time.elapsed().as_secs_f64();
        let fps = self.total_frames as f64 / elapsed;

        eprintln!(
            "\rProcessed {} frames in {:.2}s ({:.2} fps)",
            self.total_frames, elapsed, fps
        );
    }
}
