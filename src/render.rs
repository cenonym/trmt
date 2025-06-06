use ratatui::{
    layout::{Constraint, Direction as LayoutDirection, Layout, Rect, Alignment},
    style::{Color, Style, Modifier},
    widgets::{Block, Borders, Paragraph, Clear},
    text::{Line, Span},
};
use std::time::Duration;
use crate::{machine::TuringMachine, config::Config};

pub struct App {
    pub machine: TuringMachine,
    pub last_step: std::time::Instant,
    pub step_interval: Duration,
    pub config: Config,
    pub show_help: bool,
    pub show_statusbar: bool,
}

impl App {
    pub fn new(config: Config) -> Self {
        Self {
            machine: TuringMachine::new(
                config.simulation.default_heads,
                &config.simulation.default_rule,
                &config
            ),
            last_step: std::time::Instant::now(),
            step_interval: Duration::from_nanos((config.simulation.default_speed_ms * 1_000_000.0) as u64),
            config,
            show_help: false,
            show_statusbar: false,
        }
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

pub fn ui(f: &mut ratatui::Frame, app: &mut App) {
    // Use full area for simulation
    render_pixel_grid(f, app, f.area());

    // Render statusbar overlay
    if app.show_statusbar {
        render_statusbar_overlay(f, app);
    }

    // Render help overlay
    if app.show_help {
        render_help_overlay(f, app);
    }
    
    app.machine.clear_dirty_cells();
}

fn render_statusbar_overlay(f: &mut ratatui::Frame, app: &App) {
    let area = f.area();
    let statusbar_area = Rect {
        x: area.x,
        y: area.height.saturating_sub(3),
        width: area.width,
        height: 3,
    };
    
    // Clear the area behind the statusbar
    f.render_widget(Clear, statusbar_area);
    
    // Build status text using pre-allocated buffer
    let mut status_text = String::with_capacity(256);
    
    // Speed displays without repeated allocations
    let speed_ms = if app.step_interval >= Duration::from_millis(1) {
        app.step_interval.as_millis() as f64
    } else {
        app.step_interval.as_nanos() as f64 / 1_000_000.0
    };

    let current_speed = if speed_ms <= 0.1 {
        "fast af".to_string()
    } else {
        format!("{}ms", speed_ms)
    };
    
    let running_text = if app.machine.running { "Running" } else { "Paused" };
    
    use std::fmt::Write;
    let _ = write!(
        &mut status_text,
        "Heads: {} | Steps: {} | Speed: {} | Rule: {} | Seed: {} | {} | {} for help",
        app.machine.num_heads,
        app.machine.steps,
        current_speed,
        app.machine.rule_string,
        app.machine.current_seed,
        running_text,
        app.config.controls.help.to_uppercase()
    );

    let status = Paragraph::new(status_text)
        .block(Block::default().borders(Borders::ALL).title("Status"))
        .style(Style::default().fg(Color::White).bg(Color::Black));
    f.render_widget(status, statusbar_area);
}

fn render_help_overlay(f: &mut ratatui::Frame, app: &App) {
    let area = f.area();
    let popup_area = centered_rect(60, 50, area);
    
    // Clear the area behind the help popup
    f.render_widget(Clear, popup_area);
    
    // Pre-build help text without repeated allocations
    let help_text = vec![
        Line::from(vec![Span::styled("Controls", Style::default().add_modifier(Modifier::BOLD))]),
        Line::from(""),
        Line::from(format!("{}: Quit", app.config.controls.quit)),
        Line::from(format!("{}: Toggle simulation", app.config.controls.toggle)),
        Line::from(format!("{}: Reset simulation", app.config.controls.reset)),
        Line::from(format!("{}: Increase speed", app.config.controls.faster)),
        Line::from(format!("{}: Decrease speed", app.config.controls.slower)),
        Line::from(format!("{}: Reload config", app.config.controls.config_reload)),
        Line::from(format!("{}: Toggle help", app.config.controls.help)),
        Line::from(format!("{}: Toggle statusbar", app.config.controls.statusbar)),
        Line::from(format!("{}: Toggle seed", app.config.controls.seed_toggle)),
        Line::from(""),
        Line::from(vec![Span::styled("Head Count", Style::default().add_modifier(Modifier::BOLD))]),
        Line::from(""),
        Line::from("1: 1 head     2: 2 heads    3: 4 heads"),
        Line::from("4: 8 heads    5: 16 heads   6: 32 heads"),
        Line::from("7: 64 heads   8: 128 heads  9: 256 heads"),
        Line::from(""),
        Line::from(vec![Span::styled(
            format!("{} to close help", app.config.controls.help.to_uppercase()),
            Style::default().add_modifier(Modifier::BOLD)
        )]),
    ];
    
    let help_paragraph = Paragraph::new(help_text)
        .block(Block::default()
            .borders(Borders::ALL)
            .title("Help")
            .title_alignment(Alignment::Center))
        .style(Style::default().fg(Color::White).bg(Color::Black))
        .alignment(Alignment::Left);
    
    f.render_widget(help_paragraph, popup_area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(LayoutDirection::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(LayoutDirection::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

#[inline(always)]
fn wrap_coords(x: i32, y: i32, width: i32, height: i32) -> (i32, i32) {
    (((x % width) + width) % width, ((y % height) + height) % height)
}

fn render_pixel_grid(f: &mut ratatui::Frame, app: &App, area: Rect) {
    let width = area.width as i32 / 2;
    let height = area.height as i32;

    // Get cached character data from config
    let cell_char_data = &app.config.display.cell_char_data;

    // Render tape cells
    if app.config.simulation.infinite_trail {
        for (&(x, y), &state) in app.machine.tape() {
            if state != 'A' { // Only render non-default states
                let (grid_x, grid_y) = wrap_coords(x, y, width, height);
                let buffer_x = area.x + (grid_x * 2) as u16;
                let buffer_y = area.y + grid_y as u16;
                
                let color = app.machine.tape_colors().get(&(x, y)).copied().unwrap_or(Color::White);
                
                // Use cached character data
                for (i, &ch) in cell_char_data.chars.iter().enumerate() {
                    let char_x = buffer_x + i as u16;
                    if char_x < area.x + area.width && buffer_y < area.y + area.height {
                        f.buffer_mut()[(char_x, buffer_y)].set_char(ch).set_fg(color);
                    }
                }
            }
        }
    }

    // Render head trails
    for head in &app.machine.heads {
        for (trail_index, &(trail_x, trail_y)) in head.trail.iter().rev().enumerate() {
            let (grid_x, grid_y) = wrap_coords(trail_x, trail_y, width, height);
            let buffer_x = area.x + (grid_x * 2) as u16;
            let buffer_y = area.y + grid_y as u16;
            
            // Map trail position to character array
            let char_index = if trail_index < app.config.display.trail_char_data.len() {
                trail_index
            } else {
                app.config.display.trail_char_data.len() - 1 // Repeat last character
            };
            let trail_char_data = &app.config.display.trail_char_data[char_index];
            
            // Use cached character data
            if trail_char_data.is_single_char {
                if buffer_x < area.x + area.width && buffer_y < area.y + area.height {
                    f.buffer_mut()[(buffer_x, buffer_y)].set_char(' ');
                }
                let char_x = buffer_x + 1;
                if char_x < area.x + area.width && buffer_y < area.y + area.height {
                    f.buffer_mut()[(char_x, buffer_y)].set_char(' ');
                }
                // Place character at position 0
                if buffer_x < area.x + area.width && buffer_y < area.y + area.height {
                    f.buffer_mut()[(buffer_x, buffer_y)].set_char(trail_char_data.chars[0]).set_fg(head.color);
                }
            } else {
                // Render all characters for multi-char strings
                for (i, &ch) in trail_char_data.chars.iter().enumerate() {
                    let char_x = buffer_x + i as u16;
                    if char_x < area.x + area.width && buffer_y < area.y + area.height {
                        f.buffer_mut()[(char_x, buffer_y)].set_char(ch).set_fg(head.color);
                    }
                }
            }
        }
    }

    // Render head positions
    for head in &app.machine.heads {
        let (grid_x, grid_y) = wrap_coords(head.x, head.y, width, height);
        let buffer_x = area.x + (grid_x * 2) as u16;
        let buffer_y = area.y + grid_y as u16;
        
        // Cycle through head characters per step
        let char_index = (app.machine.steps as usize) % app.config.display.head_char_data.len();
        let head_char_data = &app.config.display.head_char_data[char_index];
        
        // Use cached character data
        if head_char_data.is_single_char {
            if buffer_x < area.x + area.width && buffer_y < area.y + area.height {
                f.buffer_mut()[(buffer_x, buffer_y)].set_char(' ');
            }
            let char_x = buffer_x + 1;
            if char_x < area.x + area.width && buffer_y < area.y + area.height {
                f.buffer_mut()[(char_x, buffer_y)].set_char(' ');
            }
            // Place character at position 0
            if buffer_x < area.x + area.width && buffer_y < area.y + area.height {
                f.buffer_mut()[(buffer_x, buffer_y)].set_char(head_char_data.chars[0]).set_fg(head.color);
            }
        } else {
            // Render all characters for multi-char strings
            for (i, &ch) in head_char_data.chars.iter().enumerate() {
                let char_x = buffer_x + i as u16;
                if char_x < area.x + area.width && buffer_y < area.y + area.height {
                    f.buffer_mut()[(char_x, buffer_y)].set_char(ch).set_fg(head.color);
                }
            }
        }
    }
}