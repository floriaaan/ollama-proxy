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

        let mut frames = Vec::new();
        let num_squares = 10;
        let num_frames = 20;

        for f in 0..num_frames {
            let t = f as f32 / num_frames as f32;
            // Easing: ease-in-out sine for smooth feel at the edges
            let eased_t = (1.0 - (t * std::f32::consts::PI).cos()) / 2.0;
            let active_pos = (eased_t * (num_squares - 1) as f32).round() as i32;

            let mut frame = String::new();
            for i in 0..num_squares {
                let i = i as i32;
                if i == active_pos {
                    frame.push_str(&console::style("■").cyan().bright().to_string());
                } else if i == active_pos - 1 {
                    frame.push_str(&console::style("▪").cyan().to_string());
                } else if i == active_pos - 2 {
                    frame.push_str(&console::style("▪").color256(242).to_string()); // Dimmer
                } else if i == active_pos - 3 {
                    frame.push_str(&console::style("▪").color256(238).to_string()); // Even dimmer
                } else {
                    frame.push_str(&console::style("▪").color256(235).to_string()); // Inactive
                }
                if i < (num_squares - 1) as i32 {
                    frame.push(' ');
                }
            }
            frames.push(frame);
        }

        let frames_refs: Vec<&str> = frames.iter().map(|s| s.as_str()).collect();

        pb.set_style(
            ProgressStyle::with_template("{prefix}  {spinner}")
                .unwrap()
                .tick_strings(&frames_refs),
        );
        pb.set_prefix(format!("{} {}", method, path));
        pb.enable_steady_tick(Duration::from_millis(80));

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
