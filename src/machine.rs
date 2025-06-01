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
}

#[derive(Debug, Clone)]
pub struct Head {
    pub x: i32,
    pub y: i32,
    pub direction: Direction,
    pub color: Color,
    pub trail: VecDeque<(i32, i32)>,
}

impl Head {
    pub fn new(x: i32, y: i32, color: Color) -> Self {
        Self {
            x,
            y,
            direction: Direction::Up,
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

#[derive(Debug)]
pub struct TuringMachine {
    pub tape: FxHashMap<(i32, i32), char>,
    pub tape_colors: FxHashMap<(i32, i32), Color>,
    pub heads: Vec<Head>,
    pub rule_string: String,
    pub rules: BTreeMap<char, (char, Direction)>,
    pub num_heads: usize,
    pub running: bool,
    pub steps: u64,
    pub current_seed: String,
    colors: Vec<Color>,
    cached_parsed_colors: FxHashMap<String, Color>,
    updates_buffer: Vec<(usize, char, Direction, i32, i32)>,
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
        
        // Generate or use provided seed
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
        
        // Create RNG from seed
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
        
        let states = ['A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z'];
        let mut state_index = 0;
        let mut i = 0;
        
        while i < rule_string.len() && state_index < states.len() {
            let remaining = &rule_string[i..];
            
            if remaining.starts_with("NW") {
                self.rules.insert(states[state_index], ('N', Direction::UpLeft));
                i += 2;
            } else if remaining.starts_with("NE") {
                self.rules.insert(states[state_index], ('N', Direction::UpRight));
                i += 2;
            } else if remaining.starts_with("SW") {
                self.rules.insert(states[state_index], ('S', Direction::DownLeft));
                i += 2;
            } else if remaining.starts_with("SE") {
                self.rules.insert(states[state_index], ('S', Direction::DownRight));
                i += 2;
            } else if let Some(c) = remaining.chars().next() {
                let direction = match c {
                    'L' => Direction::Left,
                    'R' => Direction::Right,
                    'N' => Direction::Up,
                    'S' => Direction::Down,
                    'E' => Direction::Right,
                    'W' => Direction::Left,
                    _ => Direction::Right,
                };
                self.rules.insert(states[state_index], (c, direction));
                i += 1;
            }
            state_index += 1;
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
            let current_state = self.get_cell(head.x, head.y);
            
            if let Some(&(turn_char, stored_direction)) = self.rules.get(&current_state) {
                let new_direction = match turn_char {
                    'L' => head.direction.turn_left(),
                    'R' => head.direction.turn_right(),
                    _ => stored_direction,
                };
                
                let states: Vec<char> = self.rules.keys().copied().collect();
                let current_idx = states.iter().position(|&s| s == current_state).unwrap_or(0);
                let next_state = states.get((current_idx + 1) % states.len()).copied().unwrap_or('A');
                
                let (new_x, new_y) = new_direction.apply(head.x, head.y);
                let wrapped_x = ((new_x % width) + width) % width;
                let wrapped_y = ((new_y % height) + height) % height;
                
                self.updates_buffer.push((i, next_state, new_direction, wrapped_x, wrapped_y));
                self.batch_updates.push(((head.x, head.y), next_state, head.color));
            }
        }

        self.batch_set_cells();

        let updates = self.updates_buffer.clone();
        for (i, _, direction, x, y) in updates {
            let head = &mut self.heads[i];
            head.direction = direction;
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