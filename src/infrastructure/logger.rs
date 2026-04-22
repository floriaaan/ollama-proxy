use std::time::Duration;

use console::Term;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};

use crate::domain::LogLevel;

pub struct RequestLogger {
    multi_progress: MultiProgress,
    is_tty: bool,
}

impl RequestLogger {
    pub fn new() -> Self {
        Self {
            multi_progress: MultiProgress::new(),
            is_tty: Term::stdout().is_term(),
        }
    }

    pub fn start_loader(&self, method: &str, path: &str) -> Option<ProgressBar> {
        if !self.is_tty {
            return None;
        }

        let pb = self.multi_progress.add(ProgressBar::new_spinner());
        pb.set_draw_target(indicatif::ProgressDrawTarget::hidden());
        pb.set_style(
            ProgressStyle::with_template("{prefix}  {spinner}")
                .unwrap()
                .tick_strings(&[
                    "        ",
                    "·       ",
                    "··      ",
                    "···     ",
                    "····    ",
                    "·····   ",
                    "······  ",
                    "······· ",
                    "········",
                    " ·······",
                    "  ······",
                    "   ·····",
                    "    ····",
                    "     ···",
                    "      ··",
                    "       ·",
                ]),
        );
        pb.set_prefix(format!("{} {}", method, path));
        pb.enable_steady_tick(Duration::from_millis(100));

        let pb_clone = pb.clone();
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(50)).await;
            pb_clone.set_draw_target(indicatif::ProgressDrawTarget::stderr());
        });

        Some(pb)
    }

    pub fn finish_loader(
        &self,
        pb: Option<ProgressBar>,
        ip: &str,
        method: &str,
        path: &str,
        status: u16,
        latency_ms: u128,
        log_level: LogLevel,
        upstream: Option<&str>,
        body_size: usize,
    ) {
        let log_line = if log_level == LogLevel::Debug {
            format!(
                "{} \"{} {}\" {} {}ms bytes={} upstream={}",
                ip,
                method,
                path,
                status,
                latency_ms,
                body_size,
                upstream.unwrap_or("-")
            )
        } else {
            format!("{} \"{} {}\" {} {}ms", ip, method, path, status, latency_ms)
        };

        if let Some(pb) = pb {
            pb.finish_and_clear();
            self.multi_progress
                .println(&log_line)
                .unwrap_or_else(|_| println!("{}", log_line));
        } else {
            println!("{}", log_line);
        }
    }
}

impl Default for RequestLogger {
    fn default() -> Self {
        Self::new()
    }
}
