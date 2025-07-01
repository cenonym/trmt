use serde::{Deserialize, Serialize};

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
    pub randomize_seed: String,
    #[serde(default = "rule_key")]
    pub randomize_rule: String,
    #[serde(default = "randomize_key")]
    pub randomize: String,
}

// Default functions
fn quit_key() -> String { "q".to_string() }
fn toggle_key() -> String { " ".to_string() }
fn reset_key() -> String { "r".to_string() }
fn faster_key() -> String { "+".to_string() }
fn slower_key() -> String { "-".to_string() }
fn config_key() -> String { "c".to_string() }
fn help_key() -> String { "h".to_string() }
fn statusbar_key() -> String { "b".to_string() }
fn seed_key() -> String { "s".to_string() }
fn rule_key() -> String { "n".to_string() }
fn randomize_key() -> String { "R".to_string() }

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
            randomize_seed: seed_key(),
            randomize_rule: rule_key(),
            randomize: randomize_key(),
        }
    }
}
