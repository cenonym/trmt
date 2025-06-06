pub mod rules;
pub mod grid;
pub mod heads;

use ratatui::style::Color;
use rustc_hash::{FxHashMap, FxHashSet};
use std::collections::BTreeMap;
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;
use crate::config::Config;

pub use rules::{StateTransition, TurnDirection};
pub use heads::Head;
pub use grid::Grid;

#[derive(Debug)]
pub struct TuringMachine {
    pub grid: Grid,
    pub heads: Vec<Head>,
    pub rule_string: String,
    pub rules: BTreeMap<(usize, char), StateTransition>,
    pub num_heads: usize,
    pub running: bool,
    pub steps: u64,
    pub current_seed: String,
    colors: Vec<Color>,
    cached_parsed_colors: FxHashMap<String, Color>,
    updates_buffer: Vec<(usize, char, TurnDirection, usize, i32, i32)>,
    pub dirty_cells: FxHashSet<(i32, i32)>,
}

impl TuringMachine {
    pub fn new(num_heads: usize, rule_string: &str, config: &Config) -> Self {
        let mut machine = Self {
            grid: Grid::new(),
            heads: Vec::with_capacity(num_heads.min(256)),
            rule_string: rule_string.to_string(),
            rules: BTreeMap::new(),
            num_heads: num_heads.min(256),
            running: true,
            steps: 0,
            current_seed: String::new(),
            colors: Vec::new(),
            cached_parsed_colors: FxHashMap::default(),
            updates_buffer: Vec::with_capacity(256),
            dirty_cells: FxHashSet::with_capacity_and_hasher(1024, Default::default()),
        };

        machine.update_colors(config);
        machine.parse_rules(rule_string);
        machine.spawn_heads(config);
        machine
    }

    pub fn update_colors(&mut self, config: &Config) {
        self.colors = config.display.colors.iter()
            .map(|c| self.parse_color_cached(c, config))
            .collect();
        
        for (i, head) in self.heads.iter_mut().enumerate() {
            head.color = self.colors[i % self.colors.len()];
        }
    }

    fn parse_color_cached(&mut self, color_str: &str, config: &Config) -> Color {
        if let Some(&cached_color) = self.cached_parsed_colors.get(color_str) {
            return cached_color;
        }
        
        let color = config.parse_color(color_str);
        self.cached_parsed_colors.insert(color_str.to_string(), color);
        color
    }

    fn spawn_heads(&mut self, config: &Config) {
        self.heads.clear();
        self.heads.reserve(self.num_heads);
        
        let seed = if let Some(config_seed) = &config.simulation.seed {
            if !config_seed.is_empty() {
                config_seed.clone()
            } else {
                self.generate_random_seed()
            }
        } else {
            self.generate_random_seed()
        };
        
        self.current_seed = seed.clone();
        let seed_hash = self.hash_seed(&seed);
        let mut rng = StdRng::seed_from_u64(seed_hash);

        for i in 0..self.num_heads {
            let x = rng.gen_range(0..100);
            let y = rng.gen_range(0..100);
            let color = self.colors[i % self.colors.len()];
            let head = Head::new(x, y, color);
            
            self.heads.push(head);
        }
    }

    fn generate_random_seed(&self) -> String {
        use rand::distributions::Alphanumeric;
        let mut rng = rand::thread_rng();
        (0..8)
            .map(|_| rng.sample(Alphanumeric) as char)
            .collect::<String>()
            .to_lowercase()
    }

    fn hash_seed(&self, seed: &str) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        seed.hash(&mut hasher);
        self.num_heads.hash(&mut hasher);
        hasher.finish()
    }

    pub fn parse_rules(&mut self, rule_string: &str) {
        self.rules = rules::parse_rules(rule_string);
    }

    #[inline(always)]
    pub fn get_cell(&self, x: i32, y: i32) -> char {
        self.grid.get_cell(x, y)
    }

    pub fn mark_trail_dirty(&mut self) {
        for head in &self.heads {
            self.dirty_cells.insert((head.x, head.y));
            for &(trail_x, trail_y) in &head.trail {
                self.dirty_cells.insert((trail_x, trail_y));
            }
        }
    }

    pub fn clear_dirty_cells(&mut self) {
        self.dirty_cells.clear();
    }

    pub fn step(&mut self, width: i32, height: i32, config: &Config) {
        if !self.running {
            return;
        }

        self.updates_buffer.clear();
        self.updates_buffer.reserve(self.heads.len());

        for (i, head) in self.heads.iter().enumerate() {
            let current_cell = self.get_cell(head.x, head.y);
            
            if let Some(transition) = self.rules.get(&(head.internal_state, current_cell)) {
                let new_direction = transition.turn_direction.apply(head.direction);
                let (new_x, new_y) = new_direction.apply(head.x, head.y);
                let wrapped_x = ((new_x % width) + width) % width;
                let wrapped_y = ((new_y % height) + height) % height;
                
                self.updates_buffer.push((
                    i,
                    transition.new_cell_state,
                    transition.turn_direction,
                    transition.new_internal_state,
                    wrapped_x,
                    wrapped_y,
                ));
                
                self.grid.set_cell(head.x, head.y, transition.new_cell_state, head.color);
                self.dirty_cells.insert((head.x, head.y));
            }
        }

        let updates = self.updates_buffer.clone();
        for (i, _, turn_direction, new_internal_state, x, y) in updates {
            let head = &mut self.heads[i];
            head.direction = turn_direction.apply(head.direction);
            head.internal_state = new_internal_state;
            head.move_to(x, y, config.simulation.trail_length);
        }
        
        self.steps += 1;
    }

    pub fn toggle_running(&mut self) {
        self.running = !self.running;
    }

    pub fn reset(&mut self, config: &Config) {
        self.running = false;
        self.steps = 0;
        self.grid.clear();
        self.dirty_cells.clear();
        self.spawn_heads(config);
    }

    pub fn set_head_count(&mut self, count: usize, config: &Config) {
        self.num_heads = count.min(256);
        self.spawn_heads(config);
    }

    // Accessor methods for render module
    pub fn tape(&self) -> &FxHashMap<(i32, i32), char> {
        &self.grid.tape
    }

    pub fn tape_colors(&self) -> &FxHashMap<(i32, i32), Color> {
        &self.grid.tape_colors
    }
}