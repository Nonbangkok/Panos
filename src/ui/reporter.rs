//! Progress reporter trait

use indicatif::{ProgressBar, ProgressStyle};
use std::sync::Mutex;

pub trait ProgressReporter: Send + Sync {
    fn start(&self, total: Option<u64>, message: String);

    fn update(&self, delta: u64, message: String);

    fn finish(&self, message: String);
}

pub struct NoopReporter;

impl ProgressReporter for NoopReporter {
    fn start(&self, _total: Option<u64>, _message: String) {}
    fn update(&self, _delta: u64, _message: String) {}
    fn finish(&self, _message: String) {}
}

pub struct IndicatifReporter {
    pb: Mutex<Option<ProgressBar>>,
}

impl IndicatifReporter {
    pub fn new() -> Self {
        Self {
            pb: Mutex::new(None),
        }
    }
}

impl ProgressReporter for IndicatifReporter {
    fn start(&self, total: Option<u64>, message: String) {
        let pb = match total {
            Some(t) => {
                let p = ProgressBar::new(t);
                p.set_style(
                    ProgressStyle::default_bar()
                        .template("{spinner:.green} {msg} [{bar:40.cyan/blue}] {pos}/{len} ({eta})")
                        .unwrap(),
                );
                p
            }
            None => {
                let p = ProgressBar::new_spinner();
                p.set_style(
                    ProgressStyle::default_spinner()
                        .template("{spinner:.green} {msg} [{elapsed_precise}]")
                        .unwrap(),
                );
                p
            }
        };
        pb.set_message(message);
        let mut guard = self.pb.lock().unwrap();
        *guard = Some(pb);
    }
    fn update(&self, delta: u64, message: String) {
        let guard = self.pb.lock().unwrap();
        if let Some(pb) = guard.as_ref() {
            if delta > 0 {
                pb.inc(delta);
            }
            if !message.is_empty() {
                pb.set_message(message);
            } else {
                pb.tick();
            }
        }
    }
    fn finish(&self, message: String) {
        let guard = self.pb.lock().unwrap();
        if let Some(pb) = guard.as_ref() {
            pb.finish_with_message(message);
        }
    }
}
