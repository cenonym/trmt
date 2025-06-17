pub mod rules;
pub mod grid;
pub mod heads;

use ratatui::style::Color;
use rustc_hash::{FxHashMap, FxHashSet};
use std::collections::BTreeMap;
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;
use crate::config::Config;
use crate::machine::rules::Direction;

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
    pub grid_width: i32,
    pub grid_height: i32,
    colors: Vec<Color>,
    cached_parsed_colors: FxHashMap<String, Color>,
    updates_buffer: Vec<(usize, char, TurnDirection, usize, i32, i32, Color)>,
    pub dirty_cells: FxHashSet<(i32, i32)>,
    head_char_sequence: Vec<usize>,
    trail_char_sequence: Vec<usize>,
    sequence_length: usize,
}

impl TuringMachine {
    pub fn new(num_heads: usize, rule_string: &str, config: &Config) -> Self {
        let sequence_length = 10000;
        
        let mut machine = Self {
            grid: Grid::new(),
            heads: Vec::with_capacity(num_heads.min(256)),
            rule_string: rule_string.to_string(),
            rules: BTreeMap::new(),
            num_heads: num_heads.min(256),
            running: config.simulation.autoplay,
            steps: 0,
            current_seed: String::new(),
            grid_width: 100,
            grid_height: 100,
            colors: Vec::new(),
            cached_parsed_colors: FxHashMap::default(),
            updates_buffer: Vec::with_capacity(256),
            dirty_cells: FxHashSet::with_capacity_and_hasher(1024, Default::default()),
            head_char_sequence: Vec::with_capacity(sequence_length),
            trail_char_sequence: Vec::with_capacity(sequence_length),
            sequence_length,
        };

        machine.update_colors(config);
        machine.parse_rules(rule_string);
        machine.spawn_heads(config);
        machine
    }

    // Pre-generate char sequences
    fn generate_random_sequences(&mut self, config: &Config) {
        let seed = if let Some(config_seed) = &config.simulation.seed {
            if !config_seed.is_empty() {
                config_seed.clone()
            } else {
                self.generate_random_seed()
            }
        } else {
            self.generate_random_seed()
        };
        
        let seed_hash = self.hash_seed(&seed);
        let mut rng = StdRng::seed_from_u64(seed_hash.wrapping_add(12345));
        
        self.head_char_sequence = (0..self.sequence_length)
            .map(|_| rng.gen_range(0..usize::MAX))
            .collect();
            
        self.trail_char_sequence = (0..self.sequence_length)
            .map(|_| rng.gen_range(0..usize::MAX))
            .collect();
    }

    fn spawn_heads(&mut self, config: &Config) {
        self.heads.clear();
        self.heads.reserve(self.num_heads);
        
        let seed = if let Some(effective_seed) = config.get_effective_seed() {
            if !effective_seed.is_empty() {
                effective_seed
            } else {
                self.generate_random_seed()
            }
        } else {
            self.generate_random_seed()
        };
        
        self.current_seed = seed.clone();
        
        let effective_rule = config.get_effective_rule();
        self.parse_rules(&effective_rule);
        self.rule_string = effective_rule;
        
        let seed_hash = self.hash_seed(&seed);
        let mut rng = StdRng::seed_from_u64(seed_hash);

        for i in 0..self.num_heads {
            let x = rng.gen_range(0..self.grid_width.max(1));
            let y = rng.gen_range(0..self.grid_height.max(1));
            let mut head = Head::new(x, y, Color::White);
            head.color = config.display.get_head_color(i);
            self.heads.push(head);
        }
        
        self.generate_random_sequences(config);
    }

    // Calculate char based on direction
    fn get_head_char(&self, head: &Head, new_direction: Direction, config: &Config) -> Option<String> {
        if config.display.direction_based_chars {
            let char_index = config.display.get_direction_char_index(new_direction, Some(head.direction));
            let index = char_index % config.display.head_char.len();
            Some(config.display.head_char[index].clone())
        } else {
            None
        }
    }

    pub fn get_head_char_index(&self, head_index: usize, config: &Config) -> usize {
        if config.display.randomize_heads {
            let sequence_index = (self.steps.wrapping_add(head_index as u64)) as usize % self.sequence_length;
            self.head_char_sequence[sequence_index] % config.display.head_char_data.len()
        } else {
            let head = &self.heads[head_index];
            config.display.get_head_char_index(
                head_index, 
                head.direction, 
                head.previous_direction
            )
        }
    }
    
    pub fn get_trail_char_index(&self, head_index: usize, trail_index: usize) -> usize {
        let sequence_index = (self.steps.wrapping_add(head_index as u64).wrapping_add(trail_index as u64 * 17)) as usize % self.sequence_length;
        self.trail_char_sequence[sequence_index]
    }

    pub fn update_colors(&mut self, config: &Config) {
        self.colors = config.display.colors.iter()
            .map(|c| self.parse_color_cached(c, config))
            .collect();
        
        for (i, head) in self.heads.iter_mut().enumerate() {
            head.color = config.display.get_head_color(i);
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

    pub fn generate_random_seed(&self) -> String {
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
                
                let live_colors_color = if config.display.state_based_colors && config.display.live_colors {
                    config.display.get_cell_color(transition.new_cell_state, i)
                } else {
                    config.display.get_head_color(i)
                };
                
                self.updates_buffer.push((
                    i,
                    transition.new_cell_state,
                    transition.turn_direction,
                    transition.new_internal_state,
                    wrapped_x,
                    wrapped_y,
                    live_colors_color,
                ));
                
                let display_char = if config.simulation.color_cells ||
                (config.display.direction_based_chars && config.simulation.trail_length > 0) {
                    self.get_head_char(head, new_direction, config)
                } else {
                    None
                };
        
                let cell_color = config.display.get_cell_color(transition.new_cell_state, i);
                self.grid.set_cell(
                    head.x, 
                    head.y, 
                    transition.new_cell_state, 
                    cell_color, 
                    display_char,
                    config.display.state_based_colors
                );
                self.dirty_cells.insert((head.x, head.y));
            }
        }

        let updates = self.updates_buffer.clone();
        for (i, _, turn_direction, new_internal_state, x, y, live_color) in updates {
            let head = &mut self.heads[i];
            let new_direction = turn_direction.apply(head.direction);
            head.set_direction(new_direction);
            head.internal_state = new_internal_state;
            head.color = live_color;
            head.move_to(x, y, config.simulation.trail_length);
        }
        
        self.steps += 1;
    }

    pub fn tape_chars(&self) -> &FxHashMap<(i32, i32), String> {
        &self.grid.tape_chars
    }

    pub fn toggle_running(&mut self) {
        self.running = !self.running;
    }

    // Save runtime state and reset
    pub fn reset(&mut self, config: &Config) {
        let _ = Config::save_current_seed(&self.current_seed);
        let _ = Config::save_current_rule(&self.rule_string);
        
        self.running = config.simulation.autoplay;
        self.steps = 0;
        self.grid.clear();
        self.dirty_cells.clear();
        self.spawn_heads(config);
    }

    pub fn reset_clean(&mut self, config: &Config) {
        self.running = config.simulation.autoplay;
        self.steps = 0;
        self.grid.clear();
        self.dirty_cells.clear();
        self.spawn_heads(config);
    }

    pub fn set_head_count(&mut self, count: usize, config: &Config) {
        self.num_heads = count.min(256);
        self.spawn_heads(config);
    }

    pub fn update_grid_dimensions(&mut self, width: i32, height: i32) {
        if self.grid_width != width || self.grid_height != height {
            // Clear existing cells when dimensions change
            self.grid.clear();
            self.dirty_cells.clear();
        }
        self.grid_width = width;
        self.grid_height = height;
    }

    pub fn tape(&self) -> &FxHashMap<(i32, i32), char> {
        &self.grid.tape
    }

    pub fn tape_colors(&self) -> &FxHashMap<(i32, i32), Color> {
        &self.grid.tape_colors
    }
}