use std::time::Duration;

use indicatif::{ProgressBar, ProgressStyle};

fn create_progress(total_steps: u64) -> ProgressBar {
    let bar = ProgressBar::new(total_steps);
    // Progress Bar needs to be updated in a different thread.
    {
        let bar = bar.downgrade();
        std::thread::spawn(move || {
            while let Some(bar) = bar.upgrade() {
                bar.tick();
                std::thread::sleep(Duration::from_millis(100));
            }
        });
    }

    bar.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} {prefix} [{elapsed_precise}] [{bar:20}]")
            .expect("failed to parse template")
            // .tick_chars("-\\|/")
            .progress_chars("=>-"),
    );

    bar
}

pub(crate) struct ServeProgress {
    inner: ProgressBar,
}

impl ServeProgress {
    pub fn new() -> Self {
        Self {
            inner: create_progress(20),
        }
    }

    pub fn step_build_frontend(&self) {
        self.inner.set_prefix("Building (frontend) ");
        self.inner.set_position(2);
    }

    pub fn step_build_backend(&self) {
        self.inner.set_prefix("Building (backend)  ");
        self.inner.set_position(10);
    }

    pub fn step_starting(&self) {
        self.inner.set_prefix("Starting            ");
        self.inner.set_position(17);
    }

    pub fn hide(self) {
        self.inner.finish_and_clear()
    }
}
