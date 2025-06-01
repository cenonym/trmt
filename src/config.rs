use ratatui::style::Color;
use serde::{Deserialize, Serialize};
use std::{error::Error, fs, path::PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    #[serde(default = "default_heads")]
    pub default_heads: usize,
    #[serde(default = "default_rule")]
    pub default_rule: String,
    #[serde(default = "default_speed")]
    pub default_speed_ms: f64,
    #[serde(default = "default_trail_length")]
    pub trail_length: usize,
    #[serde(default = "default_infinite_trail")]
    pub infinite_trail: bool,
    #[serde(default = "default_random_head_direction")]
    pub random_head_direction: bool,
    #[serde(default = "default_seed")]
    pub seed: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DisplayConfig {
    #[serde(default = "default_colors")]
    pub colors: Vec<String>,
    #[serde(default = "default_head_char")]
    pub head_char: Vec<String>,
    #[serde(default = "default_trail_char")]
    pub trail_char: Vec<String>,
    #[serde(default = "default_cell_char")]
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
    #[serde(default = "default_quit_key")]
    pub quit: String,
    #[serde(default = "default_toggle_key")]
    pub toggle: String,
    #[serde(default = "default_reset_key")]
    pub reset: String,
    #[serde(default = "default_faster_key")]
    pub faster: String,
    #[serde(default = "default_slower_key")]
    pub slower: String,
    #[serde(default = "default_config_key")]
    pub config_reload: String,
    #[serde(default = "default_help_key")]
    pub help: String,
    #[serde(default = "default_statusbar_key")]
    pub statusbar: String,
    #[serde(default = "default_seed_key")]
    pub seed_toggle: String,
}

// Default config
fn default_heads() -> usize { 4 }
fn default_rule() -> String { "RL".to_string() }
fn default_speed() -> f64 { 100.0 }
fn default_trail_length() -> usize { 20 }
fn default_colors() -> Vec<String> {
    vec![
        "#FF5500".to_string(),
        "#00FF88".to_string(),
        "#8844FF".to_string(),
        "#FFAA00".to_string(),
        "rgb(255,100,150)".to_string(),
        "rgb(100,255,200)".to_string(),
        "rgb(200,100,255)".to_string(),
        "88".to_string(),
        "28".to_string(),
        "129".to_string(),
        "208".to_string(),
        "39".to_string(),
    ]
}
fn default_head_char() -> Vec<String> { 
    vec!["██".to_string()] 
}
fn default_trail_char() -> Vec<String> { 
    vec!["▓▓".to_string()] 
}
fn default_cell_char() -> String { "░░".to_string() }
fn default_quit_key() -> String { "q".to_string() }
fn default_toggle_key() -> String { " ".to_string() }
fn default_reset_key() -> String { "r".to_string() }
fn default_faster_key() -> String { "+".to_string() }
fn default_slower_key() -> String { "-".to_string() }
fn default_config_key() -> String { "c".to_string() }
fn default_help_key() -> String { "h".to_string() }
fn default_statusbar_key() -> String { "b".to_string() }
fn default_seed_key() -> String { "s".to_string() }
fn default_random_head_direction() -> bool { false }
fn default_infinite_trail() -> bool { false }
fn default_seed() -> Option<String> { Some(String::new()) }

impl Default for Config {
    fn default() -> Self {
        Self {
            simulation: SimulationConfig::default(),
            display: DisplayConfig::default(),
            controls: ControlsConfig::default(),
        }
    }
}

impl Default for SimulationConfig {
    fn default() -> Self {
        Self {
            default_heads: default_heads(),
            default_rule: default_rule(),
            default_speed_ms: default_speed(),
            trail_length: default_trail_length(),
            infinite_trail: default_infinite_trail(),
            random_head_direction: default_random_head_direction(),
            seed: default_seed(),
        }
    }
}

impl Default for DisplayConfig {
    fn default() -> Self {
        let mut config = Self {
            colors: default_colors(),
            head_char: default_head_char(),
            trail_char: default_trail_char(),
            cell_char: default_cell_char(),
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
            quit: default_quit_key(),
            toggle: default_toggle_key(),
            reset: default_reset_key(),
            faster: default_faster_key(),
            slower: default_slower_key(),
            config_reload: default_config_key(),
            help: default_help_key(),
            statusbar: default_statusbar_key(),
            seed_toggle: default_seed_key(),
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
}

impl Config {
    pub fn load() -> Self {
        let config_path = Self::config_dir().join("config.toml");
        
        if config_path.exists() {
            match fs::read_to_string(&config_path) {
                Ok(content) => match toml::from_str::<Config>(&content) {
                    Ok(mut config) => {
                        if let Err(errors) = config.validate() {
                            eprintln!("Config validation failed:");
                            for error in errors {
                                eprintln!("  {}", error);
                            }
                            eprintln!("Using default config instead.");
                        } else {
                            config.display.cache_char_data();
                            return config;
                        }
                    },
                    Err(e) => eprintln!("Failed to parse config: {}", e),
                },
                Err(e) => eprintln!("Failed to read config: {}", e),
            }
        }
        
        // Return default config and create example file
        let default_config = Self::default();
        let _ = default_config.create_example_config();
        default_config
    }

    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        // Validate rule string
        if let Err(e) = self.validate_rule_string(&self.simulation.default_rule) {
            errors.push(format!("simulation.default_rule: {}", e));
        }

        // Validate colors
        for (i, color) in self.display.colors.iter().enumerate() {
            if let Err(e) = self.validate_color(color) {
                errors.push(format!("display.colors[{}]: {}", i, e));
            }
        }

        // Validate numeric ranges
        if self.simulation.default_heads == 0 || self.simulation.default_heads > 256 {
            errors.push("simulation.default_heads: must be between 1 and 256".to_string());
        }

        if self.simulation.default_speed_ms <= 0.0 {
            errors.push("simulation.default_speed_ms: must be positive".to_string());
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
            return Err("rule string cannot be empty".to_string());
        }

        // Handle explicit state rules (contains commas)
        if rule.contains(',') {
            let combinations: Vec<&str> = rule.split(',').collect();
            for combo in combinations {
                if combo.is_empty() {
                    return Err("rule combination cannot be empty".to_string());
                }
                
                if let Some(colon_pos) = combo.find(':') {
                    let full_action = &combo[..colon_pos];
                    let state_part = &combo[colon_pos + 1..];
                    
                    // Split state prefix from action (e.g., "0D" -> "D")
                    let action = if full_action.len() > 1 && full_action.chars().next().unwrap().is_ascii_digit() {
                        &full_action[1..]
                    } else {
                        full_action
                    };
                    
                    // Validate action part
                    self.validate_direction_string(action)?;
                    
                    // Validate state number
                    if !state_part.chars().all(|c| c.is_ascii_digit()) {
                        return Err(format!("invalid state number '{}' in '{}'", state_part, combo));
                    }
                } else {
                    self.validate_direction_string(combo)?;
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

    fn validate_direction_string(&self, rule: &str) -> Result<(), String> {
        let mut i = 0;
        while i < rule.len() {
            let remaining = &rule[i..];
            
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

    pub fn toggle_seed(&mut self, current_seed: &str) -> Result<(), Box<dyn Error>> {
        let config_path = Self::config_dir().join("config.toml");
        
        // Update seed in config
        if let Some(config_seed) = &self.simulation.seed {
            if config_seed == current_seed {
                self.simulation.seed = Some(String::new()); // Clear seed
            } else {
                self.simulation.seed = Some(current_seed.to_string()); // Set current seed
            }
        } else {
            self.simulation.seed = Some(current_seed.to_string()); // Set seed if None
        }
        
        // Write updated config
        let content = toml::to_string_pretty(self)?;
        fs::write(&config_path, content)?;
        
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