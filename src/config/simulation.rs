use serde::{Deserialize, Serialize};
use rand::Rng;
use std::collections::{HashSet};

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
fn speed() -> f64 { 5.0 }
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
    const DIRECTIONS: &'static [&'static str] = &["L", "R", "U", "D", "N", "S", "W", "E"];

    // Random rule generation
    pub fn generate_random_rule() -> String {
        let mut rng = rand::thread_rng();
        
        // Generate multiple rules and pick the most promising
        let mut candidates = Vec::new();
        for _ in 0..5 {
            let rule = match rng.gen_range(0..10) {
                0..=6 => Self::generate_basic_rule(&mut rng),        // 70%
                7..=8 => Self::generate_multi_state_rule(&mut rng),  // 20%
                _ => Self::generate_explicit_rule(&mut rng),         // 10%
            };
            candidates.push(rule);
        }
        
        // Pick the best rule
        candidates.into_iter()
            .max_by_key(|rule| Self::score_rule_potential(rule))
            .unwrap_or_else(|| "RL".to_string())
    }
    
    fn generate_basic_rule(rng: &mut impl Rng) -> String {
        let length = rng.gen_range(2..=9);
        let mut rule = String::with_capacity(length);
        let mut left_count = 0;
        let mut right_count = 0;
        
        for _ in 0..length {
            let dir = Self::DIRECTIONS[rng.gen_range(0..Self::DIRECTIONS.len())];
            rule.push_str(dir);
            
            // Track L/R balance
            if dir == "L" { left_count += 1; }
            if dir == "R" { right_count += 1; }
        }
        
        // If severely imbalanced, add a balancing direction
        if (left_count as i32 - right_count as i32).abs() > length as i32 / 2 {
            let balance_dir = if left_count > right_count { "R" } else { "L" };
            rule.push_str(balance_dir);
        }
        
        rule
    }
    
    fn generate_multi_state_rule(rng: &mut impl Rng) -> String {
        let states = rng.gen_range(2..=3);
        let mut state_rules = Vec::<String>::with_capacity(states);
        
        for i in 0..states {
            let base_rule = if i == 0 {
                Self::generate_basic_rule(rng)
            } else {
                Self::generate_contrasting_rule(rng, &state_rules[0])
            };
            
            let rule_with_transition = if rng.gen_bool(0.5) && states > 1 {
                let target = (i + 1) % states;
                format!("{}>{}", base_rule, target)
            } else {
                base_rule
            };
            
            state_rules.push(rule_with_transition);
        }
        
        state_rules.join(":")
    }
    
    fn generate_contrasting_rule(rng: &mut impl Rng, base_rule: &str) -> String {
        let has_mostly_left = base_rule.matches('L').count() > base_rule.matches('R').count();
        let length = rng.gen_range(2..=4);
        let mut rule = String::with_capacity(length);
        
        // Filter directions
        let contrast_dirs: Vec<&str> = if has_mostly_left {
            Self::DIRECTIONS.iter().filter(|&&d| d != "L").copied().collect()
        } else {
            Self::DIRECTIONS.iter().filter(|&&d| d != "R").copied().collect()
        };
        
        for _ in 0..length {
            let dir = contrast_dirs[rng.gen_range(0..contrast_dirs.len())];
            rule.push_str(dir);
        }
        
        rule
    }
    
    fn generate_explicit_rule(rng: &mut impl Rng) -> String {
        let states = rng.gen_range(2..=3);
        let mut transitions = Vec::with_capacity(states * 2);
        
        let multi_transition_states = rng.gen_range(1..=2.min(states));
        let mut has_multi = HashSet::new();
        
        while has_multi.len() < multi_transition_states {
            has_multi.insert(rng.gen_range(0..states));
        }
        
        for i in 0..states {
            if has_multi.contains(&i) {
                for _ in 0..2 {
                    let dir = Self::DIRECTIONS[rng.gen_range(0..Self::DIRECTIONS.len())];
                    let next_state = if i == states - 1 { 
                        rng.gen_range(0..states) 
                    } else { 
                        (i + 1) % states 
                    };
                    transitions.push(format!("{}>{}", dir, next_state));
                }
            } else {
                let dir = Self::DIRECTIONS[rng.gen_range(0..Self::DIRECTIONS.len())];
                let next_state = (i + 1) % states;
                transitions.push(format!("{}>{}", dir, next_state));
            }
        }
        
        transitions.join(",")
    }
    
    // Score rules based on potential for interesting behavior
    fn score_rule_potential(rule: &str) -> i32 {
        let mut score = 0;
        
        // Penalize rules that are too short or too long
        let total_length: usize = rule.split(':').map(|s| s.len()).sum();
        if total_length >= 4 && total_length <= 12 {
            score += 10;
        }
        
        // Reward balanced L/R ratios (avoid spinning)
        let l_count = rule.matches('L').count() as i32;
        let r_count = rule.matches('R').count() as i32;
        let balance = 5 - (l_count - r_count).abs().min(5);
        score += balance * 2;
        
        // Reward variety in directions
        let mut unique_dirs = HashSet::new();
        for c in rule.chars() {
            if "LRUD".contains(c) {
                unique_dirs.insert(c);
            }
        }
        score += unique_dirs.len() as i32 * 3;
        
        score
    }
}