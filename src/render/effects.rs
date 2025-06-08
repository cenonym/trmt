use ratatui::style::Color;

pub fn fade_color_to_target(original: Color, target: Color, fade_factor: f32) -> Color {
    match (original, target) {
        (Color::Rgb(or, og, ob), Color::Rgb(tr, tg, tb)) => {
            let r = (or as f32 * (1.0 - fade_factor) + tr as f32 * fade_factor) as u8;
            let g = (og as f32 * (1.0 - fade_factor) + tg as f32 * fade_factor) as u8;
            let b = (ob as f32 * (1.0 - fade_factor) + tb as f32 * fade_factor) as u8;
            Color::Rgb(r, g, b)
        }
        _ => original,
    }
}