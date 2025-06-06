use ratatui::{
    layout::{Constraint, Direction as LayoutDirection, Layout, Rect, Alignment},
    style::{Color, Style, Modifier},
    widgets::{Block, Borders, Paragraph, Clear, Wrap},
    text::{Line, Span},
    symbols::border,
};
use std::time::Duration;
use crate::{machine::TuringMachine, config::Config};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PopupPosition {
    Center,
    Bottom,
}

#[derive(Debug, Clone)]
pub struct PopupConfig {
    pub title: String,
    pub title_style: Style,
    pub border_style: Style,
    pub content_style: Style,
    pub background_style: Style,
    pub max_width_percent: u16,
    pub max_height_percent: Option<u16>,
    pub alignment: Alignment,
    pub wrap_text: bool,
    pub position: PopupPosition,
    pub padding: u16,
}

impl Default for PopupConfig {
    fn default() -> Self {
        Self {
            title: "Popup".to_string(),
            title_style: Style::default().add_modifier(Modifier::BOLD),
            border_style: Style::default().fg(Color::Gray),
            content_style: Style::default().fg(Color::White),
            background_style: Style::default().bg(Color::Rgb(24, 24, 32)),
            max_width_percent: 70,
            max_height_percent: Some(80),
            alignment: Alignment::Left,
            wrap_text: true,
            position: PopupPosition::Center,
            padding: 1,
        }
    }
}

impl PopupConfig {
    pub fn error() -> Self {
        Self {
            title: "Error".to_string(),
            title_style: Style::default().add_modifier(Modifier::BOLD).fg(Color::Rgb(255, 99, 99)),
            border_style: Style::default().fg(Color::Rgb(255, 99, 99)),
            background_style: Style::default().bg(Color::Rgb(32, 24, 24)),
            content_style: Style::default().fg(Color::Rgb(255, 220, 220)),
            max_width_percent: 50,
            max_height_percent: None,
            ..Default::default()
        }
    }

    pub fn help() -> Self {
        Self {
            title: "Help".to_string(),
            title_style: Style::default().add_modifier(Modifier::BOLD).fg(Color::Rgb(100, 200, 255)),
            border_style: Style::default().fg(Color::Rgb(100, 200, 255)),
            background_style: Style::default().bg(Color::Rgb(24, 28, 32)),
            content_style: Style::default().fg(Color::Rgb(220, 235, 255)),
            max_width_percent: 60,
            max_height_percent: Some(50),
            ..Default::default()
        }
    }

    pub fn statusbar() -> Self {
        Self {
            title: "Status".to_string(),
            title_style: Style::default().add_modifier(Modifier::BOLD).fg(Color::Rgb(150, 200, 100)),
            border_style: Style::default().fg(Color::Rgb(100, 150, 80)),
            background_style: Style::default().bg(Color::Rgb(24, 28, 24)),
            content_style: Style::default().fg(Color::Rgb(200, 220, 200)),
            max_width_percent: 100,
            max_height_percent: None,
            position: PopupPosition::Bottom,
            wrap_text: false,
            padding: 0,
            ..Default::default()
        }
    }
}

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
                config.simulation.default_heads,
                &config.simulation.default_rule,
                &config
            ),
            last_step: std::time::Instant::now(),
            step_interval: Duration::from_nanos((config.simulation.default_speed_ms * 1_000_000.0) as u64),
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

pub fn ui(f: &mut ratatui::Frame, app: &mut App) {
    // Use full area for simulation
    render_pixel_grid(f, app, f.area());

    // Render overlays (error has highest priority)
    if let Some(ref error) = app.error_message {
        render_error_overlay(f, app, error);
    } else if app.show_statusbar {
        render_statusbar_overlay(f, app);
    } else if app.show_help {
        render_help_overlay(f, app);
    }
    
    app.machine.clear_dirty_cells();
}

pub fn render_enhanced_popup(
    f: &mut ratatui::Frame,
    content: Vec<Line>,
    config: PopupConfig,
) {
    let area = f.area();
    
    // Calculate dimensions accounting for horizontal padding (left + right)
    let max_line_width = content.iter()
        .map(|line| line.width())
        .max()
        .unwrap_or(0) as u16;
    
    let border_width = 2 + 2;
    let total_width = max_line_width + border_width;
    let max_width = (area.width * config.max_width_percent) / 100;
    let popup_width = total_width.min(max_width);
    
    let content_width = popup_width.saturating_sub(border_width);
    let wrapped_lines: u16 = if config.wrap_text {
        content.iter()
            .map(|line| {
                let line_width = line.width() as u16;
                if line_width == 0 {
                    1
                } else {
                    (line_width + content_width - 1) / content_width
                }
            })
            .sum()
    } else {
        content.len() as u16
    };
    
    // Height: border + top padding + bottom padding
    let border_height = 2 + config.padding;
    let total_height = wrapped_lines + border_height;
    
    let popup_height = if let Some(max_height_percent) = config.max_height_percent {
        let max_height = (area.height * max_height_percent) / 100;
        total_height.min(max_height)
    } else {
        total_height
    };

    let popup_area = match config.position {
        PopupPosition::Center => centered_rect_fixed_size(popup_width, popup_height, area),
        PopupPosition::Bottom => bottom_rect_fixed_size(popup_width, popup_height, area),
    };
    
    f.render_widget(Clear, popup_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_set(border::ROUNDED)
        .title(format!(" {} ", config.title)) // Add padding to title
        .title_alignment(Alignment::Left)
        .title_style(config.title_style)
        .border_style(config.border_style)
        .style(config.background_style);

    // Content area: 1 padding on sides, config.padding on top and bottom
    let content_area = Rect {
        x: popup_area.x + 2, // border + left padding
        y: popup_area.y + 1 + config.padding, // border + top padding
        width: popup_area.width.saturating_sub(4), // subtract left+right padding
        height: popup_area.height.saturating_sub(2 + config.padding), // subtract borders + top padding
    };

    // Render background block
    f.render_widget(block, popup_area);

    // Render content with enhanced styling
    let enhanced_content: Vec<Line> = content.into_iter().map(|line| {
        let spans: Vec<Span> = line.spans.into_iter().map(|span| {
            if span.style == Style::default() {
                Span::styled(span.content, config.content_style)
            } else {
                span
            }
        }).collect();
        Line::from(spans)
    }).collect();

    let mut paragraph = Paragraph::new(enhanced_content)
        .alignment(config.alignment);
    
    if config.wrap_text {
        paragraph = paragraph.wrap(Wrap { trim: true });
    }
    
    f.render_widget(paragraph, content_area);
}

fn centered_rect_fixed_size(width: u16, height: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(LayoutDirection::Vertical)
        .constraints([
            Constraint::Length((r.height.saturating_sub(height)) / 2),
            Constraint::Length(height),
            Constraint::Min(0),
        ])
        .split(r);

    Layout::default()
        .direction(LayoutDirection::Horizontal)
        .constraints([
            Constraint::Length((r.width.saturating_sub(width)) / 2),
            Constraint::Length(width),
            Constraint::Min(0),
        ])
        .split(popup_layout[1])[1]
}

fn bottom_rect_fixed_size(width: u16, height: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(LayoutDirection::Vertical)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(height),
        ])
        .split(r);

    Layout::default()
        .direction(LayoutDirection::Horizontal)
        .constraints([
            Constraint::Length((r.width.saturating_sub(width)) / 2),
            Constraint::Length(width),
            Constraint::Min(0),
        ])
        .split(popup_layout[1])[1]
}

fn render_error_overlay(f: &mut ratatui::Frame, _app: &App, error_message: &str) {
    let mut error_text = vec![];
    
    for line in error_message.lines() {
        if line.trim().is_empty() {
            error_text.push(Line::from(""));
        } else if line.starts_with("Config") {
            error_text.push(Line::from(vec![Span::styled(line, Style::default().add_modifier(Modifier::BOLD))]));
        } else {
            error_text.push(Line::from(format!("• {}", line.trim())));
        }
    }
    
    error_text.push(Line::from(""));
    error_text.push(Line::from(vec![Span::styled("Press 'x' to close", Style::default().add_modifier(Modifier::BOLD))]));
    
    render_enhanced_popup(f, error_text, PopupConfig::error());
}

fn render_help_overlay(f: &mut ratatui::Frame, app: &App) {
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
        Line::from(vec![Span::styled("Press 'x' to close overlays", Style::default().add_modifier(Modifier::BOLD))]),
    ];
    
    render_enhanced_popup(f, help_text, PopupConfig::help());
}

fn render_statusbar_overlay(f: &mut ratatui::Frame, app: &App) {
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
    
    let status_text = format!(
        "Heads: {} | Steps: {} | Speed: {} | Rule: {} | Seed: {} | {} | {} for help | x to close",
        app.machine.num_heads,
        app.machine.steps,
        current_speed,
        app.machine.rule_string,
        app.machine.current_seed,
        running_text,
        app.config.controls.help.to_uppercase()
    );

    let content = vec![Line::from(status_text)];
    render_enhanced_popup(f, content, PopupConfig::statusbar());
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