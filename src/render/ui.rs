use ratatui::{
    layout::{Constraint, Direction as LayoutDirection, Layout, Rect, Alignment},
    style::{Color, Style, Modifier},
    widgets::{Block, Borders, Paragraph, Clear, Wrap},
    text::{Line, Span},
    symbols::border,
    Frame,
};
use super::App;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PopupPosition {
    Center,
    Bottom,
    BottomLeft,
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
            max_height_percent: None,
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

    pub fn keycast() -> Self {
        Self {
            title: "".to_string(),
            title_style: Style::default(),
            border_style: Style::default().fg(Color::Rgb(100, 150, 80)),
            background_style: Style::default().bg(Color::Rgb(24, 28, 24)),
            content_style: Style::default().fg(Color::Rgb(200, 220, 200)).add_modifier(Modifier::BOLD),
            max_width_percent: 20,
            max_height_percent: None,
            alignment: Alignment::Center,
            wrap_text: false,
            position: PopupPosition::BottomLeft,
            padding: 0,
        }
    }
}

pub fn render_popup(f: &mut Frame, content: Vec<Line>, config: PopupConfig) {
    let area = f.area();
    
    // Calculate dimensions
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
                    line_width.div_ceil(content_width)
                }
            })
            .sum()
    } else {
        content.len() as u16
    };
    
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
        PopupPosition::BottomLeft => bottom_left_rect_fixed_size(popup_width, popup_height, area),
    };
    
    f.render_widget(Clear, popup_area);

    let block = if config.title.is_empty() {
        Block::default()
            .borders(Borders::ALL)
            .border_set(border::ROUNDED)
            .border_style(config.border_style)
            .style(config.background_style)
    } else {
        Block::default()
            .borders(Borders::ALL)
            .border_set(border::ROUNDED)
            .title(format!(" {} ", config.title))
            .title_alignment(Alignment::Left)
            .title_style(config.title_style)
            .border_style(config.border_style)
            .style(config.background_style)
    };

    // Content area
    let content_area = Rect {
        x: popup_area.x + 2,
        y: popup_area.y + 1 + config.padding,
        width: popup_area.width.saturating_sub(4),
        height: popup_area.height.saturating_sub(2 + config.padding),
    };

    // Render background
    f.render_widget(block, popup_area);

    // Render content
    let formatted_content: Vec<Line> = content.into_iter().map(|line| {
        let spans: Vec<Span> = line.spans.into_iter().map(|span| {
            if span.style == Style::default() {
                Span::styled(span.content, config.content_style)
            } else {
                span
            }
        }).collect();
        Line::from(spans)
    }).collect();

    let mut paragraph = Paragraph::new(formatted_content)
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

fn bottom_left_rect_fixed_size(width: u16, height: u16, r: Rect) -> Rect {
    Rect {
        x: r.x + 1,
        y: r.y + r.height.saturating_sub(height),
        width,
        height,
    }
}

pub fn render_error_overlay(f: &mut Frame, _app: &App, error_message: &str) {
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
    
    render_popup(f, error_text, PopupConfig::error());
}

pub fn render_help_overlay(f: &mut Frame, app: &App) {
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
        Line::from(format!("{}: Random seed", app.config.controls.randomize_seed)),
        Line::from(format!("{}: Random rule", app.config.controls.randomize_rule)),
        Line::from("R: Random seed & rule"),
        Line::from(""),
        Line::from(vec![Span::styled("Head Count", Style::default().add_modifier(Modifier::BOLD))]),
        Line::from(""),
        Line::from("1: 1 head     2: 2 heads    3: 4 heads"),
        Line::from("4: 8 heads    5: 16 heads   6: 32 heads"),
        Line::from("7: 64 heads   8: 128 heads  9: 256 heads"),
        Line::from(""),
        Line::from(vec![Span::styled("Press 'x' to close overlays", Style::default().add_modifier(Modifier::BOLD))]),
    ];
    
    render_popup(f, help_text, PopupConfig::help());
}

pub fn render_statusbar_overlay(f: &mut Frame, app: &App) {
    let speed_ms = if app.step_interval >= std::time::Duration::from_millis(1) {
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
        "{} | Heads: {} | Steps: {} | Speed: {} | Rule: {} | Seed: {}",
        running_text,
        app.machine.num_heads,
        app.machine.steps,
        current_speed,
        app.machine.rule_string,
        app.machine.current_seed
    );

    let content = vec![Line::from(status_text)];
    render_popup(f, content, PopupConfig::statusbar());
}

pub fn render_keycast_overlay(f: &mut Frame, app: &App) {
    if let Some(ref keypress) = app.last_keypress {
        let content = vec![Line::from(keypress.clone())];
        render_popup(f, content, PopupConfig::keycast());
    }
}