mod config;
mod machine;
mod render;

use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{
    error::Error,
    io,
    time::Duration,
};

use config::{Config, ConfigLoadResult};
use render::{App, ui};

fn main() -> Result<(), Box<dyn Error>> {
    let (config, error_message) = match Config::load() {
        ConfigLoadResult::Success(config) => (config, None),
        ConfigLoadResult::ValidationErrors(config, errors) => {
            (config, Some(format!("Config validation failed:\n{}", errors.join("\n"))))
        },
        ConfigLoadResult::ParseError(config, error) => {
            (config, Some(format!("Config parse error: {}", error)))
        },
        ConfigLoadResult::IoError(config, error) => {
            (config, Some(format!("Config I/O error: {}", error)))
        },
    };
    
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(config);
    
    // Show error if config loading failed
    if let Some(error) = error_message {
        app.show_error(error);
    }
    
    let res = run_app(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err);
    }

    Ok(())
}

fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> io::Result<()> {
    loop {
        let area = terminal.draw(|f| ui(f, app))?.area;
        
        if event::poll(Duration::from_millis(16))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char(ch) => {
                        let ch_str = ch.to_string();
                        match ch_str.as_str() {
                            s if s == app.config.controls.quit => return Ok(()),
                            s if s == app.config.controls.toggle => app.machine.toggle_running(),
                            s if s == app.config.controls.reset => app.machine.reset(&app.config),
                            s if s == app.config.controls.faster => {
                                if app.step_interval > Duration::from_millis(100) {
                                    app.step_interval = app.step_interval.saturating_sub(Duration::from_millis(50));
                                } else if app.step_interval > Duration::from_millis(10) {
                                    app.step_interval = app.step_interval.saturating_sub(Duration::from_millis(10));
                                } else if app.step_interval > Duration::from_millis(1) {
                                    app.step_interval = app.step_interval.saturating_sub(Duration::from_millis(1));
                                } else {
                                    app.step_interval = app.step_interval.saturating_sub(Duration::from_nanos(100_000));
                                    if app.step_interval < Duration::from_nanos(100_000) {
                                        app.step_interval = Duration::from_nanos(100_000);
                                    }
                                }
                            },
                            s if s == app.config.controls.slower => {
                                if app.step_interval < Duration::from_nanos(100_000) {
                                    app.step_interval = Duration::from_nanos(100_000);
                                } else if app.step_interval < Duration::from_millis(1) {
                                    app.step_interval = app.step_interval.saturating_add(Duration::from_nanos(100_000));
                                } else if app.step_interval < Duration::from_millis(10) {
                                    app.step_interval = app.step_interval.saturating_add(Duration::from_millis(1));
                                } else if app.step_interval < Duration::from_millis(100) {
                                    app.step_interval = app.step_interval.saturating_add(Duration::from_millis(10));
                                } else {
                                    app.step_interval = app.step_interval.saturating_add(Duration::from_millis(50));
                                }
                            },
                            s if s == app.config.controls.config_reload => {
                                match Config::load() {
                                    ConfigLoadResult::Success(config) => {
                                        app.config = config;
                                        app.config.display.cache_char_data();
                                        app.machine.set_head_count(app.config.simulation.heads, &app.config);
                                        app.step_interval = Duration::from_nanos((app.config.simulation.speed_ms * 1_000_000.0) as u64);
                                        app.machine.parse_rules(&app.config.simulation.rule);
                                        app.machine.rule_string = app.config.simulation.rule.clone();
                                        app.machine.update_colors(&app.config);
                                        app.machine.reset(&app.config);
                                        app.error_message = None; // Clear any existing errors
                                    },
                                    ConfigLoadResult::ValidationErrors(config, errors) => {
                                        app.config = config;
                                        app.show_error(format!("Config validation failed:\n{}", errors.join("\n")));
                                    },
                                    ConfigLoadResult::ParseError(config, error) => {
                                        app.config = config;
                                        app.show_error(format!("Config parse error: {}", error));
                                    },
                                    ConfigLoadResult::IoError(config, error) => {
                                        app.config = config;
                                        app.show_error(format!("Config I/O error: {}", error));
                                    },
                                }
                            },
                            "1" => app.machine.set_head_count(1, &app.config),
                            "2" => app.machine.set_head_count(2, &app.config),
                            "3" => app.machine.set_head_count(4, &app.config),
                            "4" => app.machine.set_head_count(8, &app.config),
                            "5" => app.machine.set_head_count(16, &app.config),
                            "6" => app.machine.set_head_count(32, &app.config),
                            "7" => app.machine.set_head_count(64, &app.config),
                            "8" => app.machine.set_head_count(128, &app.config),
                            "9" => app.machine.set_head_count(256, &app.config),
                            s if s == app.config.controls.help => app.show_help = !app.show_help,
                            s if s == app.config.controls.statusbar => app.show_statusbar = !app.show_statusbar,
                            s if s == app.config.controls.seed_toggle => {
                                let _ = app.config.toggle_seed(&app.machine.current_seed);
                            },
                            "x" => app.clear_overlays(),
                            _ => {}
                        }
                    },
                    _ => {}
                }
            }
        }

        app.update(area.width as i32 / 2, area.height as i32);
    }
}