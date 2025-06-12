use ratatui::style::Color;
use serde::{Deserialize, Serialize};
use std::{error::Error, fs, path::PathBuf};
use rand::Rng;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Default)]
pub struct Config {
    #[serde(default)]
    pub simulation: SimulationConfig,
    #[serde(default)]
    pub display: DisplayConfig,
    #[serde(default)]
    pub controls: ControlsConfig,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SimulationConfig {
    #[serde(default = "autoplay")]
    pub autoplay: bool,
    #[serde(default = "heads")]
    pub heads: usize,
    #[serde(default = "rule")]
    pub rule: String,
    #[serde(default = "speed")]
    pub speed_ms: f64,
    #[serde(default = "trail_length")]
    pub trail_length: usize,
    #[serde(default = "color_cells")]
    pub color_cells: bool,
    #[serde(default = "seed")]
    pub seed: Option<String>,
}

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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ControlsConfig {
    #[serde(default = "quit_key")]
    pub quit: String,
    #[serde(default = "toggle_key")]
    pub toggle: String,
    #[serde(default = "reset_key")]
    pub reset: String,
    #[serde(default = "faster_key")]
    pub faster: String,
    #[serde(default = "slower_key")]
    pub slower: String,
    #[serde(default = "config_key")]
    pub config_reload: String,
    #[serde(default = "help_key")]
    pub help: String,
    #[serde(default = "statusbar_key")]
    pub statusbar: String,
    #[serde(default = "seed_key")]
    pub seed_toggle: String,
    #[serde(default = "rule_key")]
    pub rule_toggle: String,
}

// Default config
fn autoplay() -> bool { true }
fn heads() -> usize { 6 }
fn rule() -> String { "RL".to_string() }
fn speed() -> f64 { 50.0 }
fn trail_length() -> usize { 16 }
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
fn quit_key() -> String { "q".to_string() }
fn toggle_key() -> String { " ".to_string() }
fn reset_key() -> String { "r".to_string() }
fn faster_key() -> String { "+".to_string() }
fn slower_key() -> String { "-".to_string() }
fn config_key() -> String { "c".to_string() }
fn help_key() -> String { "h".to_string() }
fn statusbar_key() -> String { "b".to_string() }
fn seed_key() -> String { "s".to_string() }
fn color_cells() -> bool { true }
fn seed() -> Option<String> { Some(String::new()) }
fn rule_key() -> String { "n".to_string() }

impl Default for SimulationConfig {
    fn default() -> Self {
        Self {
            autoplay: autoplay(),
            heads: heads(),
            rule: rule(),
            speed_ms: speed(),
            trail_length: trail_length(),
            color_cells: color_cells(),
            seed: seed(),
        }
    }
}

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

impl Default for ControlsConfig {
    fn default() -> Self {
        Self {
            quit: quit_key(),
            toggle: toggle_key(),
            reset: reset_key(),
            faster: faster_key(),
            slower: slower_key(),
            config_reload: config_key(),
            help: help_key(),
            statusbar: statusbar_key(),
            seed_toggle: seed_key(),
            rule_toggle: rule_key(),
        }
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

    pub fn get_cell_color(&self, cell_state: char, head_index: usize, config: &Config) -> Color {
        if self.state_based_colors {
            // Map colors to states
            let cell_index = (cell_state as u8).saturating_sub(b'A') as usize;
            if !self.colors.is_empty() {
                config.parse_color(&self.colors[cell_index % self.colors.len()])
            } else {
                Color::White
            }
        } else {
            // Map colors to heads
            if !self.colors.is_empty() {
                config.parse_color(&self.colors[head_index % self.colors.len()])
            } else {
                Color::White
            }
        }
    }

    pub fn should_render_cell(&self, cell_state: char) -> bool {
        self.state_based_colors || cell_state != 'A'
    }

    pub fn get_head_color(&self, head_index: usize, config: &Config) -> Color {
        if self.state_based_colors && !self.live_colors {
            // Cycle through colors by head index
            if !self.colors.is_empty() {
                config.parse_color(&self.colors[head_index % self.colors.len()])
            } else {
                Color::White
            }
        } else if self.state_based_colors && self.live_colors {
            if !self.colors.is_empty() {
                config.parse_color(&self.colors[0])
            } else {
                Color::White
            }
        } else {
            // Head-based
            if !self.colors.is_empty() {
                config.parse_color(&self.colors[head_index % self.colors.len()])
            } else {
                Color::White
            }
        }
    }
}

pub enum ConfigLoadResult {
    Success(Config),
    ValidationErrors(Config, Vec<String>),
    ParseError(Config, String),
    IoError(Config, String),
}

impl Config {
    pub fn load() -> ConfigLoadResult {
        let config_path = Self::config_dir().join("config.toml");
        
        if config_path.exists() {
            match fs::read_to_string(&config_path) {
                Ok(content) => match toml::from_str::<Config>(&content) {
                    Ok(mut config) => {
                        if let Err(errors) = config.validate() {
                            ConfigLoadResult::ValidationErrors(Config::default(), errors)
                        } else {
                            config.display.cache_char_data();
                            ConfigLoadResult::Success(config)
                        }
                    },
                    Err(e) => ConfigLoadResult::ParseError(Config::default(), e.to_string()),
                },
                Err(e) => ConfigLoadResult::IoError(Config::default(), e.to_string()),
            }
        } else {
            // Return default config and create example file
            let default_config = Self::default();
            let _ = default_config.create_example_config();
            ConfigLoadResult::Success(default_config)
        }
    }

    fn state_dir() -> PathBuf {
        if let Some(state_dir) = std::env::var_os("XDG_STATE_HOME") {
            PathBuf::from(state_dir).join("trmt")
        } else if let Some(home_dir) = dirs::home_dir() {
            home_dir.join(".local").join("state").join("trmt")
        } else {
            PathBuf::from(".local/state/trmt")
        }
    }

    pub fn get_effective_seed(&self) -> Option<String> {
        let state_path = Self::state_dir().join("current_seed");
        
        // State takes precedence when it exists
        if state_path.exists() {
            if let Ok(seed) = std::fs::read_to_string(&state_path) {
                let trimmed = seed.trim();
                if !trimmed.is_empty() {
                    return Some(trimmed.to_string());
                }
            }
        }
        
        // Fall back to config
        if let Some(ref config_seed) = self.simulation.seed {
            if !config_seed.is_empty() {
                return Some(config_seed.clone());
            }
        }
        
        None
    }

    pub fn save_current_seed(seed: &str) -> Result<(), Box<dyn Error>> {
        let state_dir = Self::state_dir();
        std::fs::create_dir_all(&state_dir)?;
        
        let state_path = state_dir.join("current_seed");
        std::fs::write(&state_path, seed)?;
        
        Ok(())
    }

    pub fn clear_current_seed() -> Result<(), Box<dyn Error>> {
        let state_path = Self::state_dir().join("current_seed");
        if state_path.exists() {
            std::fs::remove_file(&state_path)?;
        }
        Ok(())
    }

    // Random rule generation
    pub fn generate_random_rule() -> String {
        let mut rng = rand::thread_rng();
        
        match rng.gen_range(0..4) {
            0 => Self::generate_basic_rule(&mut rng),
            1 => Self::generate_multi_state_rule(&mut rng),
            2 => Self::generate_explicit_rule(&mut rng),
            _ => Self::generate_cell_specifier_rule(&mut rng),
        }
    }
    
    fn generate_basic_rule(rng: &mut impl Rng) -> String {
        let directions = ["L", "R", "U", "D", "N", "S", "E", "W", "NW", "NE", "SW", "SE"];
        let length = rng.gen_range(2..=8);
        
        let mut unique_dirs = std::collections::HashSet::new();
        let mut rule = Vec::new();
        
        while rule.len() < length {
            let dir = directions[rng.gen_range(0..directions.len())];
            rule.push(dir);
            unique_dirs.insert(dir);
            
            if rule.len() >= length - 1 && unique_dirs.len() < 2 {
                let different_dir = directions.iter()
                    .find(|&&d| !unique_dirs.contains(d))
                    .unwrap_or(&directions[0]);
                rule.push(different_dir);
                break;
            }
        }
        
        rule.into_iter().collect()
    }
    
    fn generate_multi_state_rule(rng: &mut impl Rng) -> String {
        let states = rng.gen_range(2..=4);
        let mut state_rules = Vec::new();
        
        // Generate base rules
        for _ in 0..states {
            state_rules.push(Self::generate_basic_rule(rng));
        }
        
        // Ensure state transitions exist by adding explicit transitions
        for state_rule in state_rules.iter_mut() {
            if rng.gen_bool(0.3) {
                let target_state = rng.gen_range(0..states);
                *state_rule = format!("{}>{}", state_rule, target_state);
            }
        }
        
        state_rules.join(":")
    }
    
    fn generate_explicit_rule(rng: &mut impl Rng) -> String {
        let directions = ["L", "R"];
        let combos = rng.gen_range(2..=4);
        (0..combos)
            .map(|i| {
                let dir = directions[rng.gen_range(0..directions.len())];
                let next_state = (i + 1) % combos; // Ensure state progression
                format!("{}>{}", dir, next_state)
            })
            .collect::<Vec<_>>()
            .join(",")
    }

    fn generate_cell_specifier_rule(rng: &mut impl Rng) -> String {
        let directions = ["L", "R", "U", "D"];
        let cell_state = rng.gen_range(0..2);
        let next_state = 1 - cell_state;
        format!("{}{}>{}", 
            directions[rng.gen_range(0..directions.len())],
            cell_state,
            next_state)
    }

    // Rule state management
    pub fn get_effective_rule(&self) -> String {
        let state_path = Self::state_dir().join("current_rule");
        
        // State takes precedence when it exists
        if state_path.exists() {
            if let Ok(rule) = std::fs::read_to_string(&state_path) {
                let trimmed = rule.trim();
                if !trimmed.is_empty() {
                    return trimmed.to_string();
                }
            }
        }
        
        // Fall back to config
        if !self.simulation.rule.is_empty() {
            return self.simulation.rule.clone();
        }
        
        // Generate random rule if both are empty
        Self::generate_random_rule()
    }

    pub fn save_current_rule(rule: &str) -> Result<(), Box<dyn Error>> {
        let state_dir = Self::state_dir();
        std::fs::create_dir_all(&state_dir)?;
        
        let state_path = state_dir.join("current_rule");
        std::fs::write(&state_path, rule)?;
        
        Ok(())
    }

    pub fn clear_current_rule() -> Result<(), Box<dyn Error>> {
        let state_path = Self::state_dir().join("current_rule");
        if state_path.exists() {
            std::fs::remove_file(&state_path)?;
        }
        Ok(())
    }

    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        // Validate rule string
        if let Err(e) = self.validate_rule_string(&self.simulation.rule) {
            errors.push(format!("simulation.rule: {}", e));
        }

        // Validate colors
        for (i, color) in self.display.colors.iter().enumerate() {
            if let Err(e) = self.validate_color(color) {
                errors.push(format!("display.colors[{}]: {}", i, e));
            }
        }

        // Validate numeric ranges
        if self.simulation.heads == 0 || self.simulation.heads > 256 {
            errors.push("simulation.heads: must be between 1 and 256".to_string());
        }

        if self.simulation.speed_ms <= 0.0 {
            errors.push("simulation.speed_ms: must be positive".to_string());
        }

        if self.simulation.trail_length == 0 {
            errors.push("simulation.trail_length: must be at least 1".to_string());
        }

        // Validate display characters
        if self.display.head_char.is_empty() || self.display.head_char.iter().any(|s| s.is_empty()) {
            errors.push("display.head_char: cannot be empty or contain empty strings".to_string());
        }
        if self.display.trail_char.is_empty() || self.display.trail_char.iter().any(|s| s.is_empty()) {
            errors.push("display.trail_char: cannot be empty or contain empty strings".to_string());
        }
        if self.display.cell_char.is_empty() {
            errors.push("display.cell_char: cannot be empty".to_string());
        }

        // Validate control keys
        let controls = [
            ("quit", &self.controls.quit),
            ("toggle", &self.controls.toggle),
            ("reset", &self.controls.reset),
            ("faster", &self.controls.faster),
            ("slower", &self.controls.slower),
            ("config_reload", &self.controls.config_reload),
            ("help", &self.controls.help),
            ("statusbar", &self.controls.statusbar),
            ("seed_toggle", &self.controls.seed_toggle),
            ("rule_toggle", &self.controls.rule_toggle),
        ];

        for (name, key) in &controls {
            if key.is_empty() {
                errors.push(format!("controls.{}: cannot be empty", name));
            }
        }

        // Check for duplicate key bindings
        let mut seen_keys = std::collections::HashSet::new();
        for (name, key) in &controls {
            if !seen_keys.insert(key) {
                errors.push(format!("controls.{}: duplicate key binding '{}'", name, key));
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    fn validate_rule_string(&self, rule: &str) -> Result<(), String> {
        if rule.is_empty() {
            return Ok(());
        }

        // Handle standard notation
        if rule.trim().starts_with('{') {
            return self.validate_standard_notation(rule);
        }

        // Handle explicit state rules
        if rule.contains(',') {
            let combinations: Vec<&str> = rule.split(',').collect();
            for combo in combinations {
                if combo.is_empty() {
                    return Err("rule combination cannot be empty".to_string());
                }
                
                let state_parts: Vec<&str> = combo.split(':').collect();
                for state_part in state_parts {
                    self.validate_direction_string(state_part)?;
                }
            }
            return Ok(());
        }

        // Split by colon for multi-state rules
        let state_rules: Vec<&str> = rule.split(':').collect();
        
        for state_rule in state_rules {
            if state_rule.is_empty() {
                return Err("state rule cannot be empty".to_string());
            }
            self.validate_direction_string(state_rule)?;
        }

        Ok(())
    }

    fn validate_standard_notation(&self, rule: &str) -> Result<(), String> {
        let cleaned = rule.replace(" ", "").replace("\n", "");
        
        if !cleaned.starts_with('{') || !cleaned.ends_with('}') {
            return Err("standard notation must start and end with braces".to_string());
        }
        
        // Basic brace balance check
        let mut brace_count = 0;
        for ch in cleaned.chars() {
            match ch {
                '{' => brace_count += 1,
                '}' => {
                    brace_count -= 1;
                    if brace_count < 0 {
                        return Err("unmatched closing brace".to_string());
                    }
                },
                _ => {}
            }
        }
        
        if brace_count != 0 {
            return Err("unmatched braces".to_string());
        }
        
        // Check for valid triplet patterns
        let mut i = 0;
        let chars: Vec<char> = cleaned.chars().collect();
        
        while i < chars.len() {
            if i + 2 < chars.len() && chars[i] == '{' && chars[i+1] != '{' {
                let mut j = i + 1;
                let mut triplet_content = String::new();
                let mut brace_depth = 1;
                
                while j < chars.len() && brace_depth > 0 {
                    match chars[j] {
                        '{' => brace_depth += 1,
                        '}' => brace_depth -= 1,
                        _ => {}
                    }
                    
                    if brace_depth > 0 {
                        triplet_content.push(chars[j]);
                    }
                    j += 1;
                }
                
                if triplet_content.matches(',').count() == 2 {
                    self.validate_triplet(&triplet_content)?;
                }
                
                i = j;
            } else {
                i += 1;
            }
        }
        
        Ok(())
    }

    fn validate_triplet(&self, triplet: &str) -> Result<(), String> {
        let values: Vec<&str> = triplet.split(',').collect();
        if values.len() != 3 {
            return Ok(());
        }
        
        // Validate cell state
        if let Ok(cell_state) = values[0].trim().parse::<usize>() {
            if cell_state > 255 {
                return Err(format!("cell state {} is out of range (0-255)", cell_state));
            }
        } else {
            return Err(format!("invalid cell state: {}", values[0]));
        }
        
        // Validate turn direction
        if let Ok(turn_dir) = values[1].trim().parse::<usize>() {
            if ![1, 2, 4, 8].contains(&turn_dir) {
                return Err(format!("invalid turn direction: {}. Must be 1 (no turn), 2 (right), 4 (u-turn), or 8 (left)", turn_dir));
            }
        } else {
            return Err(format!("invalid turn direction: {}", values[1]));
        }
        
        // Validate internal state
        if values[2].trim().parse::<usize>().is_err() {
            return Err(format!("invalid internal state: {}", values[2]));
        }
        
        Ok(())
    }

    fn validate_direction_string(&self, rule: &str) -> Result<(), String> {
        // Check if rule has state transition indicator
        let directions = if let Some(transition_pos) = rule.find('>') {
            let next_state_str = &rule[transition_pos + 1..];
            // Validate state number
            if !next_state_str.chars().all(|c| c.is_ascii_digit()) {
                return Err(format!("invalid state number '{}' in rule '{}'", next_state_str, rule));
            }
            &rule[..transition_pos] // Validate only the directions
        } else {
            rule
        };
        
        let mut i = 0;
        while i < directions.len() {
            let remaining = &directions[i..];
            
            if remaining.starts_with("NW") || remaining.starts_with("NE") ||
            remaining.starts_with("SW") || remaining.starts_with("SE") {
                i += 2;
            } else if let Some(c) = remaining.chars().next() {
                match c {
                    'L' | 'R' | 'U' | 'D' | 'N' | 'S' | 'E' | 'W' | '0'..='9' => i += 1,
                    _ => return Err(format!("invalid character '{}' in rule '{}'", c, rule)),
                }
            } else {
                break;
            }
        }
        Ok(())
    }

    fn validate_color(&self, color_str: &str) -> Result<(), String> {
        // Validate hex
        if color_str.starts_with('#') && color_str.len() == 7 {
            if color_str[1..].chars().all(|c| c.is_ascii_hexdigit()) {
                return Ok(());
            } else {
                return Err(format!("invalid hex color format '{}'", color_str));
            }
        }
        
        // Validate rgb
        if color_str.starts_with("rgb(") && color_str.ends_with(')') {
            let inner = &color_str[4..color_str.len()-1];
            let parts: Vec<&str> = inner.split(',').map(|s| s.trim()).collect();
            if parts.len() != 3 {
                return Err(format!("RGB format must have 3 components: '{}'", color_str));
            }
            for part in parts {
                if part.parse::<u8>().is_err() {
                    return Err(format!("invalid RGB component '{}' in '{}'", part, color_str));
                }
            }
            return Ok(());
        }

        // Validate 256-color
        if let Ok(_index) = color_str.parse::<u8>() {
            return Ok(());
        }

        Err(format!("invalid color format '{}'. Supported formats: #RRGGBB, rgb(r,g,b), or 0-255", color_str))
    }

    fn config_dir() -> PathBuf {
        if let Some(config_dir) = dirs::config_dir() {
            config_dir.join("trmt")
        } else {
            PathBuf::from(".config/trmt")
        }
    }

    fn create_example_config(&self) -> Result<(), Box<dyn Error>> {
        let config_dir = Self::config_dir();
        fs::create_dir_all(&config_dir)?;
        
        let config_path = config_dir.join("config.toml");
        if !config_path.exists() {
            let example_content = toml::to_string_pretty(self)?;
            fs::write(&config_path, example_content)?;
        }
        
        Ok(())
    }

    pub fn parse_color(&self, color_str: &str) -> Color {
        // Parse hex colors
        if color_str.starts_with('#') && color_str.len() == 7 {
            if let (Ok(r), Ok(g), Ok(b)) = (
                u8::from_str_radix(&color_str[1..3], 16),
                u8::from_str_radix(&color_str[3..5], 16),
                u8::from_str_radix(&color_str[5..7], 16),
            ) {
                return Color::Rgb(r, g, b);
            }
        }
        
        // Parse rgb colors
        if color_str.starts_with("rgb(") && color_str.ends_with(')') {
            let inner = &color_str[4..color_str.len()-1];
            let parts: Vec<&str> = inner.split(',').map(|s| s.trim()).collect();
            if parts.len() == 3 {
                if let (Ok(r), Ok(g), Ok(b)) = (
                    parts[0].parse::<u8>(),
                    parts[1].parse::<u8>(),
                    parts[2].parse::<u8>(),
                ) {
                    return Color::Rgb(r, g, b);
                }
            }
        }

        // Parse 256-colors
        if let Ok(index) = color_str.parse::<u8>() {
            return Color::Indexed(index);
        }

        // Fallback to white for invalid colors
        Color::White
    }
}