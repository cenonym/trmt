pub mod grid;
pub mod effects;
pub mod ui;

use ratatui::Frame;
use crate::{machine::TuringMachine, config::{Config, SimMode}};
use std::time::Duration;

pub struct App {
    pub machine: TuringMachine,
    pub last_step: std::time::Instant,
    pub step_interval: Duration,
    pub config: Config,
    pub show_help: bool,
    pub show_statusbar: bool,
    pub error_message: Option<String>,
    pub last_keypress: Option<String>,
    pub keypress_time: Option<std::time::Instant>
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
            last_keypress: None,
            keypress_time: None,
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

    pub fn register_keypress(&mut self, key: String) {
        if self.config.display.keycast {
            self.last_keypress = Some(key);
            self.keypress_time = Some(std::time::Instant::now());
        }
    }

    pub fn should_show_keycast(&self) -> bool {
        if !self.config.display.keycast {
            return false;
        }
        
        if let (Some(_), Some(time)) = (&self.last_keypress, &self.keypress_time) {
            // Show keycast for 2 seconds
            time.elapsed() < Duration::from_millis(2000)
        } else {
            false
        }
    }

    pub fn update(&mut self, width: i32, height: i32) {
        // Update grid dimensions
        self.machine.update_grid_dimensions(width, height);
        
        if self.machine.running && self.last_step.elapsed() >= self.step_interval {
            let steps_per_frame = if self.step_interval < Duration::from_millis(16) {
                (Duration::from_millis(16).as_nanos() / self.step_interval.as_nanos().max(1)) as usize
            } else {
                1
            };
            
            for _ in 0..steps_per_frame.min(100) {
                self.machine.step(width, height, &self.config);
                if self.machine.detection_pending() {
                    break;
                }
            }
            
            self.machine.mark_trail_dirty();
            self.last_step = std::time::Instant::now();
        }

        if self.machine.running {
            self.apply_mode_reaction();
        }
    }

    fn apply_mode_reaction(&mut self) {
        if !self.machine.detection_pending() {
            return;
        }
        match self.config.simulation.mode {
            SimMode::Loop => self.machine.restart_replay(&self.config),
            SimMode::Halt => self.machine.auto_halt(),
        }
    }
}

pub fn ui(f: &mut Frame, app: &mut App) {
    grid::render_pixel_grid(f, app, f.area());

    if app.should_show_keycast() {
        ui::render_keycast_overlay(f, app);
    }

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::machine::DetectionStatus;

    #[test]
    fn no_reaction_while_detector_running() {
        let mut config = Config::default();
        config.simulation.mode = SimMode::Loop;
        let mut app = App::new(config);
        app.machine.running = true;
        app.apply_mode_reaction();
        assert!(app.machine.running);
        assert!(!app.machine.auto_halted);
    }

    // has_looped pre-set keeps the test out of ~/.local/state
    #[test]
    fn proven_loop_restart_keeps_running_despite_autoplay_off() {
        let mut config = Config::default();
        config.simulation.mode = SimMode::Loop;
        config.simulation.autoplay = false;
        let mut app = App::new(config);
        app.machine.running = true;
        app.machine.has_looped = true;
        app.machine.detector.mark_stalled(10);
        app.apply_mode_reaction();
        assert_eq!(app.machine.steps, 0, "restart must reset the run");
        assert!(app.machine.running, "machine-initiated restarts keep running");
        assert!(app.machine.has_looped);
        assert_eq!(app.machine.detector.status(), DetectionStatus::Running);
    }

    #[test]
    fn halt_mode_stops_the_machine_on_detection() {
        let mut config = Config::default();
        config.simulation.mode = SimMode::Halt;
        let mut app = App::new(config);
        app.machine.running = true;
        app.machine.detector.mark_stalled(10);
        app.apply_mode_reaction();
        assert!(!app.machine.running);
        assert!(app.machine.auto_halted);
        assert_eq!(app.machine.detector.status(), DetectionStatus::Stalled { at_step: 10 });
    }
}