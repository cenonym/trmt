use ratatui::{Frame, layout::Rect, style::Color};
use super::{App, effects};

#[inline(always)]
fn wrap_coords(x: i32, y: i32, width: i32, height: i32) -> (i32, i32) {
    (((x % width) + width) % width, ((y % height) + height) % height)
}

pub fn render_pixel_grid(f: &mut Frame, app: &App, area: Rect) {
    let width = area.width as i32 / 2;
    let height = area.height as i32;

    render_tape_cells(f, app, area, width, height);
    render_trails(f, app, area, width, height);
    render_heads(f, app, area, width, height);
}

fn render_tape_cells(f: &mut Frame, app: &App, area: Rect, width: i32, height: i32) {
    if !app.config.simulation.color_cells {
        return;
    }

    let cell_char_data = &app.config.display.cell_char_data;

    for (&(x, y), &state) in app.machine.tape() {
        if app.config.display.should_render_cell(state) {
            let (grid_x, grid_y) = wrap_coords(x, y, width, height);
            let buffer_x = area.x + (grid_x * 2) as u16;
            let buffer_y = area.y + grid_y as u16;
            
            let color = app.machine.tape_colors().get(&(x, y)).copied().unwrap_or(Color::White);
            
            for (i, &ch) in cell_char_data.chars.iter().enumerate() {
                let char_x = buffer_x + i as u16;
                if char_x < area.x + area.width && buffer_y < area.y + area.height {
                    f.buffer_mut()[(char_x, buffer_y)].set_char(ch).set_fg(color);
                }
            }
        }
    }
}

fn render_trails(f: &mut Frame, app: &App, area: Rect, width: i32, height: i32) {
    for (head_index, head) in app.machine.heads.iter().enumerate() {
        for (trail_index, &(trail_x, trail_y)) in head.trail.iter().rev().enumerate() {
            let (grid_x, grid_y) = wrap_coords(trail_x, trail_y, width, height);
            let buffer_x = area.x + (grid_x * 2) as u16;
            let buffer_y = area.y + grid_y as u16;
            
            let char_index = if app.config.display.randomize_trails {
                let random_index = app.machine.get_trail_char_index(head_index, trail_index);
                random_index % app.config.display.trail_char_data.len()
            } else if trail_index < app.config.display.trail_char_data.len() {
                trail_index
            } else {
                app.config.display.trail_char_data.len() - 1
            };
            
            let trail_char_data = &app.config.display.trail_char_data[char_index];
            
            let color = if !app.config.display.fade_trail_color.is_empty() {
                let fade_factor = trail_index as f32 / app.config.simulation.trail_length as f32;
                let target_color = app.config.parse_color(&app.config.display.fade_trail_color);
                effects::fade_color_to_target(head.color, target_color, fade_factor)
            } else {
                head.color
            };

            render_character_at_position(f, trail_char_data, buffer_x, buffer_y, area, color);
        }
    }
}

fn render_heads(f: &mut Frame, app: &App, area: Rect, width: i32, height: i32) {
    for (head_index, head) in app.machine.heads.iter().enumerate() {
        let (grid_x, grid_y) = wrap_coords(head.x, head.y, width, height);
        let buffer_x = area.x + (grid_x * 2) as u16;
        let buffer_y = area.y + grid_y as u16;
        
        // Force clear both positions
        for i in 0..2 {
            let char_x = buffer_x + i as u16;
            if char_x < area.x + area.width && buffer_y < area.y + area.height {
                f.buffer_mut()[(char_x, buffer_y)].set_char(' ');
            }
        }
        
        let char_index = if app.config.display.randomize_heads {
            let random_index = app.machine.get_head_char_index(head_index);
            random_index % app.config.display.head_char_data.len()
        } else {
            (app.machine.steps as usize) % app.config.display.head_char_data.len()
        };
        
        let head_char_data = &app.config.display.head_char_data[char_index];
        render_character_at_position(f, head_char_data, buffer_x, buffer_y, area, head.color);
    }
}

fn render_character_at_position(f: &mut Frame, char_data: &crate::config::CharData, buffer_x: u16, buffer_y: u16, area: Rect, color: Color) {
    if char_data.is_single_char {
        if buffer_x < area.x + area.width && buffer_y < area.y + area.height {
            f.buffer_mut()[(buffer_x, buffer_y)].set_char(' ');
        }
        let char_x = buffer_x + 1;
        if char_x < area.x + area.width && buffer_y < area.y + area.height {
            f.buffer_mut()[(char_x, buffer_y)].set_char(' ');
        }
        if buffer_x < area.x + area.width && buffer_y < area.y + area.height {
            f.buffer_mut()[(buffer_x, buffer_y)].set_char(char_data.chars[0]).set_fg(color);
        }
    } else {
        for (i, &ch) in char_data.chars.iter().enumerate() {
            let char_x = buffer_x + i as u16;
            if char_x < area.x + area.width && buffer_y < area.y + area.height {
                f.buffer_mut()[(char_x, buffer_y)].set_char(ch).set_fg(color);
            }
        }
    }
}