use std::collections::BTreeMap;

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

pub fn parse_rules(rule_string: &str) -> BTreeMap<(usize, char), StateTransition> {
    let mut rules = BTreeMap::new();
    
    if rule_string.contains('>') || rule_string.contains(':') {
        // Enhanced or legacy multi-state format
        parse_multi_state_rules(rule_string, &mut rules);
    } else {
        // Simple sequential
        parse_single_state_rules(rule_string, &mut rules);
    }
    
    rules
}

fn parse_multi_state_rules(rule_string: &str, rules: &mut BTreeMap<(usize, char), StateTransition>) {
    let state_rules: Vec<&str> = rule_string.split(':').collect();
    
    for (state_idx, state_rule) in state_rules.iter().enumerate() {
        parse_enhanced_state_rule(state_idx, state_rule, &state_rules, rules);
    }
}

fn parse_single_state_rules(rule_string: &str, rules: &mut BTreeMap<(usize, char), StateTransition>) {
    parse_legacy_rules(rule_string, rules);
}

fn parse_legacy_rules(rule_string: &str, rules: &mut BTreeMap<(usize, char), StateTransition>) {
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
        
        rules.insert((0, states[state_index]), StateTransition {
            new_cell_state: states[(state_index + 1) % rule_string.len()],
            turn_direction,
            new_internal_state: 0,
        });
    }
}

fn parse_enhanced_state_rule(
    state_idx: usize, 
    rule: &str, 
    all_state_rules: &[&str], 
    rules: &mut BTreeMap<(usize, char), StateTransition>
) {
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
                
                rules.insert((state_idx, current_cell), StateTransition {
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
        
        rules.insert((state_idx, current_cell), StateTransition {
            new_cell_state: next_cell,
            turn_direction,
            new_internal_state: next_state,
        });
        
        i += chars_consumed;
        cell_state_idx += 1;
    }
}