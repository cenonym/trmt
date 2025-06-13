use serde::{Deserialize, Serialize};
use rand::Rng;

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

// Default functions
fn autoplay() -> bool { true }
fn heads() -> usize { 6 }
fn rule() -> String { "RL".to_string() }
fn speed() -> f64 { 50.0 }
fn trail_length() -> usize { 16 }
fn color_cells() -> bool { true }
fn seed() -> Option<String> { Some(String::new()) }

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

impl SimulationConfig {
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
}
