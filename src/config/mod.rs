pub mod simulation;
pub mod display;
pub mod controls;
pub mod validation;

use ratatui::style::Color;
use serde::{Deserialize, Serialize};
use std::{error::Error, fs, path::PathBuf};

pub use simulation::SimulationConfig;
pub use display::{DisplayConfig, CharData};
pub use controls::ControlsConfig;

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
        SimulationConfig::generate_random_rule()
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
        validation::validate_config(self)
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
        validation::parse_color(color_str)
    }

    // Forward method to SimulationConfig
    pub fn generate_random_rule() -> String {
        SimulationConfig::generate_random_rule()
    }
}
