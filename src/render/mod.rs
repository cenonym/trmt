pub mod grid;
pub mod effects;
pub mod ui;

use ratatui::Frame;
use crate::{machine::TuringMachine, config::Config};
use std::time::Duration;

pub struct App {
    pub machine: TuringMachine,
    pub last_step: std::time::Instant,
    pub step_interval: Duration,
    pub config: Config,
    pub show_help: bool,
    pub show_statusbar: bool,
    pub error_message: Option<String>,
}

impl App {
    pub fn new(config: Config) -> Self {
        Self {
            machine: TuringMachine::new(
                config.simulation.heads,
                &config.simulation.rule,
                &config
            ),
            last_step: std::time::Instant::now(),
            step_interval: Duration::from_nanos((config.simulation.speed_ms * 1_000_000.0) as u64),
            config,
            show_help: false,
            show_statusbar: false,
            error_message: None,
        }
    }

    pub fn show_error(&mut self, message: String) {
        self.error_message = Some(message);
    }

    pub fn clear_overlays(&mut self) {
        self.show_help = false;
        self.show_statusbar = false;
        self.error_message = None;
    }

    pub fn update(&mut self, width: i32, height: i32) {
        if self.machine.running && self.last_step.elapsed() >= self.step_interval {
            let steps_per_frame = if self.step_interval < Duration::from_millis(16) {
                (Duration::from_millis(16).as_nanos() / self.step_interval.as_nanos().max(1)) as usize
            } else {
                1
            };
            
            for _ in 0..steps_per_frame.min(100) {
                self.machine.step(width, height, &self.config);
            }
            
            self.machine.mark_trail_dirty();
            self.last_step = std::time::Instant::now();
        }
    }
}

pub fn ui(f: &mut Frame, app: &mut App) {
    grid::render_pixel_grid(f, app, f.area());

    // Render overlays
    if let Some(ref error) = app.error_message {
        ui::render_error_overlay(f, app, error);
    } else if app.show_statusbar {
        ui::render_statusbar_overlay(f, app);
    } else if app.show_help {
        ui::render_help_overlay(f, app);
    }
    
    app.machine.clear_dirty_cells();
}