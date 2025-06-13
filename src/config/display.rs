use ratatui::style::Color;
use serde::{Deserialize, Serialize};
use super::validation;
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DisplayConfig {
    #[serde(default = "keycast")]
    pub keycast: bool,
    #[serde(default = "colors")]
    pub colors: Vec<String>,
    #[serde(default = "fade_trail_color")]
    pub fade_trail_color: String,
    #[serde(default = "state_based_colors")]
    pub state_based_colors: bool,
    #[serde(default = "live_colors")]
    pub live_colors: bool,
    #[serde(default = "randomize_heads")]
    pub randomize_heads: bool,
    #[serde(default = "randomize_trails")]
    pub randomize_trails: bool,
    #[serde(default = "head_char")]
    pub head_char: Vec<String>,
    #[serde(default = "trail_char")]
    pub trail_char: Vec<String>,
    #[serde(default = "cell_char")]
    pub cell_char: String,
    
    // Cached character data
    #[serde(skip)]
    pub head_char_data: Vec<CharData>,
    #[serde(skip)]
    pub trail_char_data: Vec<CharData>,
    #[serde(skip)]
    pub cell_char_data: CharData,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct CharData {
    pub chars: Vec<char>,
    pub is_single_char: bool,
}

impl CharData {
    fn new(s: &str) -> Self {
        let chars: Vec<char> = s.chars().collect();
        Self {
            is_single_char: chars.len() == 1,
            chars,
        }
    }
}

// Default functions
fn keycast() -> bool { false }
fn colors() -> Vec<String> {
    vec![
        "rgb(241, 113, 54)".to_string(),
        "rgb(255,204,153)".to_string(),
        "#FFB3D1".to_string(),
        "#B3FFB3".to_string(),
        "225".to_string(),
        "194".to_string(),
    ]
}
fn state_based_colors() -> bool { false }
fn live_colors() -> bool { false }
fn head_char() -> Vec<String> { 
    vec!["██".to_string()] 
}
fn trail_char() -> Vec<String> { 
    vec!["▓▓".to_string()] 
}
fn cell_char() -> String { "░░".to_string() }
fn randomize_heads() -> bool { false }
fn randomize_trails() -> bool { false }
fn fade_trail_color() -> String { String::new() }

impl Default for DisplayConfig {
    fn default() -> Self {
        let mut config = Self {
            keycast: keycast(),
            colors: colors(),
            fade_trail_color: fade_trail_color(),
            state_based_colors: state_based_colors(),
            live_colors: live_colors(),
            head_char: head_char(),
            trail_char: trail_char(),
            cell_char: cell_char(),
            randomize_heads: randomize_heads(),
            randomize_trails: randomize_trails(),
            head_char_data: Vec::new(),
            trail_char_data: Vec::new(),
            cell_char_data: CharData::new(""),
        };
        config.cache_char_data();
        config
    }
}

impl DisplayConfig {
    pub fn cache_char_data(&mut self) {
        self.head_char_data = self.head_char.iter()
            .map(|s| CharData::new(s))
            .collect();
        
        self.trail_char_data = self.trail_char.iter()
            .map(|s| CharData::new(s))
            .collect();
            
        self.cell_char_data = CharData::new(&self.cell_char);
    }

    pub fn get_cell_color(&self, cell_state: char, head_index: usize) -> Color {
        if self.state_based_colors {
            // Map colors to states
            let cell_index = (cell_state as u8).saturating_sub(b'A') as usize;
            if !self.colors.is_empty() {
                parse_color(&self.colors[cell_index % self.colors.len()])
            } else {
                Color::White
            }
        } else {
            // Map colors to heads
            if !self.colors.is_empty() {
                parse_color(&self.colors[head_index % self.colors.len()])
            } else {
                Color::White
            }
        }
    }

    pub fn should_render_cell(&self, cell_state: char) -> bool {
        self.state_based_colors || cell_state != 'A'
    }

    pub fn get_head_color(&self, head_index: usize) -> Color {
        if self.state_based_colors && !self.live_colors {
            // Cycle through colors by head index
            if !self.colors.is_empty() {
                parse_color(&self.colors[head_index % self.colors.len()])
            } else {
                Color::White
            }
        } else if self.state_based_colors && self.live_colors {
            if !self.colors.is_empty() {
                parse_color(&self.colors[0])
            } else {
                Color::White
            }
        } else {
            // Head-based
            if !self.colors.is_empty() {
                parse_color(&self.colors[head_index % self.colors.len()])
            } else {
                Color::White
            }
        }
    }
}

pub fn parse_color(color_str: &str) -> Color {
    validation::parse_color(color_str)
}
