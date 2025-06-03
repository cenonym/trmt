use ratatui::style::Color;
use rustc_hash::{FxHashMap, FxHashSet};
use std::collections::{BTreeMap, VecDeque};
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;
use crate::config::Config;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
    UpLeft,
    UpRight,
    DownLeft,
    DownRight,
}

impl Direction {
    #[inline(always)]
    pub fn apply(&self, x: i32, y: i32) -> (i32, i32) {
        match self {
            Direction::Up => (x, y - 1),
            Direction::Down => (x, y + 1),
            Direction::Left => (x - 1, y),
            Direction::Right => (x + 1, y),
            Direction::UpLeft => (x - 1, y - 1),
            Direction::UpRight => (x + 1, y - 1),
            Direction::DownLeft => (x - 1, y + 1),
            Direction::DownRight => (x + 1, y + 1),
        }
    }

    #[inline]
    pub fn turn_left(self) -> Self {
        match self {
            Direction::Up => Direction::Left,
            Direction::Left => Direction::Down,
            Direction::Down => Direction::Right,
            Direction::Right => Direction::Up,
            Direction::UpLeft => Direction::DownLeft,
            Direction::DownLeft => Direction::DownRight,
            Direction::DownRight => Direction::UpRight,
            Direction::UpRight => Direction::UpLeft,
        }
    }

    #[inline]
    pub fn turn_right(self) -> Self {
        match self {
            Direction::Up => Direction::Right,
            Direction::Right => Direction::Down,
            Direction::Down => Direction::Left,
            Direction::Left => Direction::Up,
            Direction::UpLeft => Direction::UpRight,
            Direction::UpRight => Direction::DownRight,
            Direction::DownRight => Direction::DownLeft,
            Direction::DownLeft => Direction::UpLeft,
        }
    }

    #[inline]
    pub fn u_turn(self) -> Self {
        match self {
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
            Direction::UpLeft => Direction::DownRight,
            Direction::UpRight => Direction::DownLeft,
            Direction::DownLeft => Direction::UpRight,
            Direction::DownRight => Direction::UpLeft,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Head {
    pub x: i32,
    pub y: i32,
    pub direction: Direction,
    pub internal_state: usize,
    pub color: Color,
    pub trail: VecDeque<(i32, i32)>,
}

impl Head {
    pub fn new(x: i32, y: i32, color: Color) -> Self {
        Self {
            x,
            y,
            direction: Direction::Up,
            internal_state: 0,
            color,
            trail: VecDeque::with_capacity(20),
        }
    }

    #[inline]
    pub fn move_to(&mut self, x: i32, y: i32, trail_length: usize) {
        self.trail.push_back((self.x, self.y));
        if self.trail.len() > trail_length {
            self.trail.pop_front();
        }
        self.x = x;
        self.y = y;
    }
}

#[derive(Debug, Clone)]
pub struct StateTransition {
    pub new_cell_state: char,
    pub turn_direction: TurnDirection,
    pub new_internal_state: usize,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TurnDirection {
    None,                    // D
    Right,                   // R
    UTurn,                   // U
    Left,                    // L
    Absolute(Direction),     // N/S/E/W/NW/NE/SW/SE
}

impl TurnDirection {
    pub fn apply(&self, current_direction: Direction) -> Direction {
        match self {
            TurnDirection::None => current_direction,
            TurnDirection::Right => current_direction.turn_right(),
            TurnDirection::UTurn => current_direction.u_turn(),
            TurnDirection::Left => current_direction.turn_left(),
            TurnDirection::Absolute(dir) => *dir,
        }
    }
}

#[derive(Debug)]
pub struct TuringMachine {
    pub tape: FxHashMap<(i32, i32), char>,
    pub tape_colors: FxHashMap<(i32, i32), Color>,
    pub heads: Vec<Head>,
    pub rule_string: String,
    pub rules: BTreeMap<(usize, char), StateTransition>, // (internal_state, cell_state) -> transition
    pub num_heads: usize,
    pub running: bool,
    pub steps: u64,
    pub current_seed: String,
    colors: Vec<Color>,
    cached_parsed_colors: FxHashMap<String, Color>,
    updates_buffer: Vec<(usize, char, TurnDirection, usize, i32, i32)>, // head_idx, new_cell, turn, new_state, x, y
    batch_updates: Vec<((i32, i32), char, Color)>,
    pub dirty_cells: FxHashSet<(i32, i32)>,
}

impl TuringMachine {
    pub fn new(num_heads: usize, rule_string: &str, config: &Config) -> Self {
        let mut machine = Self {
            tape: FxHashMap::with_capacity_and_hasher(8192, Default::default()),
            tape_colors: FxHashMap::with_capacity_and_hasher(8192, Default::default()),
            heads: Vec::with_capacity(num_heads.min(256)),
            rule_string: rule_string.to_string(),
            rules: BTreeMap::new(),
            num_heads: num_heads.min(256),
            running: false,
            steps: 0,
            current_seed: String::new(),
            colors: Vec::new(),
            cached_parsed_colors: FxHashMap::default(),
            updates_buffer: Vec::with_capacity(256),
            batch_updates: Vec::with_capacity(256),
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
            let mut head = Head::new(x, y, color);
            
            if config.simulation.random_head_direction {
                head.direction = match rng.gen_range(0..8) {
                    0 => Direction::Up,
                    1 => Direction::Down,
                    2 => Direction::Left,
                    3 => Direction::Right,
                    4 => Direction::UpLeft,
                    5 => Direction::UpRight,
                    6 => Direction::DownLeft,
                    7 => Direction::DownRight,
                    _ => Direction::Up,
                };
            }
            
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
        self.rules.clear();
        
        if rule_string.contains('>') || rule_string.contains(':') {
            // Enhanced or legacy multi-state format
            self.parse_multi_state_rules(rule_string);
        } else {
            // Simple sequential
            self.parse_single_state_rules(rule_string);
        }
    }

    fn parse_multi_state_rules(&mut self, rule_string: &str) {
        let state_rules: Vec<&str> = rule_string.split(':').collect();
        
        for (state_idx, state_rule) in state_rules.iter().enumerate() {
            self.parse_enhanced_state_rule(state_idx, state_rule, &state_rules);
        }
    }

    fn parse_single_state_rules(&mut self, rule_string: &str) {
        self.parse_legacy_rules(rule_string);
    }

    fn parse_legacy_rules(&mut self, rule_string: &str) {
        let states = ['A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z'];
        
        for state_index in 0..states.len() {
            let rule_char_index = state_index % rule_string.len();
            let rule_chars: Vec<char> = rule_string.chars().collect();
            let c = rule_chars[rule_char_index];
            
            let remaining = &rule_string[rule_char_index..];
            
            let turn_direction = if remaining.starts_with("NW") {
                TurnDirection::Absolute(Direction::UpLeft)
            } else if remaining.starts_with("NE") {
                TurnDirection::Absolute(Direction::UpRight)
            } else if remaining.starts_with("SW") {
                TurnDirection::Absolute(Direction::DownLeft)
            } else if remaining.starts_with("SE") {
                TurnDirection::Absolute(Direction::DownRight)
            } else {
                match c {
                    'L' => TurnDirection::Left,
                    'R' => TurnDirection::Right,
                    'U' => TurnDirection::UTurn,
                    'D' => TurnDirection::None,
                    'N' => TurnDirection::Absolute(Direction::Up),
                    'S' => TurnDirection::Absolute(Direction::Down),
                    'E' => TurnDirection::Absolute(Direction::Right),
                    'W' => TurnDirection::Absolute(Direction::Left),
                    _ => TurnDirection::Right,
                }
            };
            
            self.rules.insert((0, states[state_index]), StateTransition {
                new_cell_state: states[(state_index + 1) % rule_string.len()],
                turn_direction,
                new_internal_state: 0,
            });
        }
    }

    fn parse_enhanced_state_rule(&mut self, state_idx: usize, rule: &str, all_state_rules: &[&str]) {
        let states = ['A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z'];
        
        // Handle comma-separated transitions first
        if rule.contains(',') {
            let transitions: Vec<&str> = rule.split(',').collect();
            for (cell_idx, transition) in transitions.iter().enumerate() {
                if cell_idx >= states.len() { break; }
                
                // Parse direction and cell specification (e.g., "L1>1" or "R0>0")
                let (directions, next_state) = if let Some(transition_pos) = transition.find('>') {
                    let directions = &transition[..transition_pos];
                    let next_state_str = &transition[transition_pos + 1..];
                    let next_state = next_state_str.parse::<usize>().unwrap_or(state_idx);
                    (directions, next_state)
                } else {
                    (*transition, state_idx)
                };
                
                let current_cell = states[cell_idx];
                
                // Check if direction string ends with a cell specifier (0 or 1)
                let (direction_part, next_cell) = if let Some(last_char) = directions.chars().last() {
                    if last_char.is_ascii_digit() {
                        let cell_idx = last_char.to_digit(10).unwrap_or(0) as usize;
                        let next_cell = states[cell_idx.min(states.len() - 1)];
                        let direction_part = &directions[..directions.len() - 1];
                        (direction_part, next_cell)
                    } else {
                        // Default A↔B cycling for compatibility
                        (directions, states[(cell_idx + 1) % 2])
                    }
                } else {
                    (directions, states[(cell_idx + 1) % 2])
                };
                
                if let Some(direction_char) = direction_part.chars().next() {
                    let turn_direction = match direction_char {
                        'L' => TurnDirection::Left,
                        'R' => TurnDirection::Right,
                        'U' => TurnDirection::UTurn,
                        'D' => TurnDirection::None,
                        'N' => TurnDirection::Absolute(Direction::Up),
                        'S' => TurnDirection::Absolute(Direction::Down),
                        'E' => TurnDirection::Absolute(Direction::Right),
                        'W' => TurnDirection::Absolute(Direction::Left),
                        _ => TurnDirection::Right,
                    };
                    
                    self.rules.insert((state_idx, current_cell), StateTransition {
                        new_cell_state: next_cell,
                        turn_direction,
                        new_internal_state: next_state,
                    });
                }
            }
            return;
        }

        // Check if rule has state transition indicator
        let (directions, next_state) = if let Some(transition_pos) = rule.find('>') {
            let directions = &rule[..transition_pos];
            let next_state_str = &rule[transition_pos + 1..];
            let next_state = next_state_str.parse::<usize>().unwrap_or(state_idx);
            (directions, next_state)
        } else {
            // Auto-cycle for legacy multi-state format (no > transitions)
            let total_states = all_state_rules.len();
            let next_state = if total_states > 1 {
                (state_idx + 1) % total_states
            } else {
                state_idx
            };
            (rule, next_state)
        };
        
        let mut i = 0;
        let mut cell_state_idx = 0;
        
        while i < directions.len() && cell_state_idx < states.len() {
            let remaining = &directions[i..];
            let current_cell = states[cell_state_idx];
            let next_cell = states[(cell_state_idx + 1) % directions.len()];
            
            // Parse direction/turn
            let (turn_direction, chars_consumed) = if remaining.starts_with("NW") {
                (TurnDirection::Absolute(Direction::UpLeft), 2)
            } else if remaining.starts_with("NE") {
                (TurnDirection::Absolute(Direction::UpRight), 2)
            } else if remaining.starts_with("SW") {
                (TurnDirection::Absolute(Direction::DownLeft), 2)
            } else if remaining.starts_with("SE") {
                (TurnDirection::Absolute(Direction::DownRight), 2)
            } else if let Some(c) = remaining.chars().next() {
                let turn = match c {
                    'L' => TurnDirection::Left,
                    'R' => TurnDirection::Right,
                    'U' => TurnDirection::UTurn,
                    'D' => TurnDirection::None,
                    'N' => TurnDirection::Absolute(Direction::Up),
                    'S' => TurnDirection::Absolute(Direction::Down),
                    'E' => TurnDirection::Absolute(Direction::Right),
                    'W' => TurnDirection::Absolute(Direction::Left),
                    _ => TurnDirection::Right,
                };
                (turn, 1)
            } else {
                break;
            };
            
            self.rules.insert((state_idx, current_cell), StateTransition {
                new_cell_state: next_cell,
                turn_direction,
                new_internal_state: next_state,
            });
            
            i += chars_consumed;
            cell_state_idx += 1;
        }
    }

    #[inline(always)]
    pub fn get_cell(&self, x: i32, y: i32) -> char {
        self.tape.get(&(x, y)).copied().unwrap_or('A')
    }

    fn batch_set_cells(&mut self) {
        for ((x, y), state, color) in self.batch_updates.drain(..) {
            self.dirty_cells.insert((x, y));
            
            if state == 'A' {
                self.tape.remove(&(x, y));
                self.tape_colors.remove(&(x, y));
            } else {
                self.tape.insert((x, y), state);
                self.tape_colors.insert((x, y), color);
            }
        }
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
        self.batch_updates.clear();
        self.updates_buffer.reserve(self.heads.len());
        self.batch_updates.reserve(self.heads.len());

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
                
                self.batch_updates.push(((head.x, head.y), transition.new_cell_state, head.color));
            }
        }

        self.batch_set_cells();

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
        self.tape.clear();
        self.tape_colors.clear();
        self.dirty_cells.clear();
        self.spawn_heads(config);
    }

    pub fn set_head_count(&mut self, count: usize, config: &Config) {
        self.num_heads = count.min(256);
        self.spawn_heads(config);
    }
}