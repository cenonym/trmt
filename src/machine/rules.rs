use std::collections::BTreeMap;

#[inline]
fn state_char(index: usize) -> char {
    if index < 26 {
        (b'A' + index as u8) as char
    } else if index < 52 {
        (b'a' + (index - 26) as u8) as char
    } else {
        // Fallback for out-of-range indices
        '?'
    }
}

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
    None,                    // D / 1
    Right,                   // R / 2
    UTurn,                   // U / 4
    Left,                    // L / 8
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
    
    // Check for standard notation
    if rule_string.trim().starts_with('{') {
        if parse_brace_notation(rule_string, &mut rules).is_err() {
            parse_fallback_rules(rule_string, &mut rules);
        }
    } else if rule_string.contains('>') || rule_string.contains(':') {
        parse_state_transition_rules(rule_string, &mut rules);
    } else {
        parse_string_rules(rule_string, &mut rules);
    }
    
    rules
}

fn parse_brace_notation(rule_string: &str, rules: &mut BTreeMap<(usize, char), StateTransition>) -> Result<(), String> {
    let cleaned = rule_string.chars().filter(|&c| !c.is_whitespace()).collect::<String>();
    
    if !cleaned.starts_with('{') || !cleaned.ends_with('}') {
        return Err("Invalid format: must start and end with braces".to_string());
    }
    
    let content = &cleaned[1..cleaned.len()-1];
    let state_parts = split_by_top_level_comma(content)?;
    
    for (state_idx, state_part) in state_parts.iter().enumerate() {
        if !state_part.starts_with('{') || !state_part.ends_with('}') {
            return Err(format!("State {} is not properly wrapped in braces", state_idx));
        }
        
        let state_content = &state_part[1..state_part.len()-1];
        let cell_parts = split_by_top_level_comma(state_content)?;
        
        for (cell_idx, cell_part) in cell_parts.iter().enumerate() {
            if !cell_part.starts_with('{') || !cell_part.ends_with('}') {
                return Err(format!("Cell rule in state {} is not properly wrapped in braces", state_idx));
            }
            
            let cell_content = &cell_part[1..cell_part.len()-1];
            let values: Vec<&str> = cell_content.split(',').collect();
            
            if values.len() != 3 {
                return Err(format!("Cell rule must have exactly 3 values, got {}", values.len()));
            }
            
            let new_cell_state_idx: usize = values[0].trim().parse()
                .map_err(|_| format!("Invalid cell state: {}", values[0]))?;
            let turn_direction_flag: usize = values[1].trim().parse()
                .map_err(|_| format!("Invalid turn direction: {}", values[1]))?;
            let new_internal_state: usize = values[2].trim().parse()
                .map_err(|_| format!("Invalid internal state: {}", values[2]))?;
            
            let turn_direction = match turn_direction_flag {
                1 => TurnDirection::None,
                2 => TurnDirection::Right,
                4 => TurnDirection::UTurn,
                8 => TurnDirection::Left,
                _ => return Err(format!("Invalid turn direction flag: {}. Must be 1, 2, 4, or 8", turn_direction_flag)),
            };
            
            rules.insert((state_idx, state_char(cell_idx)), StateTransition {
                new_cell_state: state_char(new_cell_state_idx),
                turn_direction,
                new_internal_state,
            });
        }
    }
    
    Ok(())
}

fn split_by_top_level_comma(s: &str) -> Result<Vec<String>, String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut brace_depth = 0;
    let chars: Vec<char> = s.chars().collect();
    
    for ch in chars {
        match ch {
            '{' => {
                brace_depth += 1;
                current.push(ch);
            },
            '}' => {
                brace_depth -= 1;
                current.push(ch);
                if brace_depth < 0 {
                    return Err("Unmatched closing brace".to_string());
                }
            },
            ',' if brace_depth == 0 => {
                if !current.trim().is_empty() {
                    parts.push(current.trim().to_string());
                }
                current.clear();
            },
            _ => {
                current.push(ch);
            }
        }
    }
    
    if brace_depth != 0 {
        return Err("Unmatched braces".to_string());
    }
    
    if !current.trim().is_empty() {
        parts.push(current.trim().to_string());
    }
    
    Ok(parts)
}

fn parse_fallback_rules(rule_string: &str, rules: &mut BTreeMap<(usize, char), StateTransition>) {
    if rule_string.contains('>') || rule_string.contains(':') {
        parse_state_transition_rules(rule_string, rules);
    } else {
        parse_string_rules(rule_string, rules);
    }
}

fn parse_state_transition_rules(rule_string: &str, rules: &mut BTreeMap<(usize, char), StateTransition>) {
    let state_rules: Vec<&str> = rule_string.split(':').collect();
    
    for (state_idx, state_rule) in state_rules.iter().enumerate() {
        parse_state_rule(state_idx, state_rule, &state_rules, rules);
    }
}

fn parse_string_rules(rule_string: &str, rules: &mut BTreeMap<(usize, char), StateTransition>) {
    if rule_string.is_empty() {
        return; // Don't parse empty rules
    }

    let mut state_index = 0;
    
    loop {
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
        
        rules.insert((0, state_char(state_index)), StateTransition {
            new_cell_state: state_char((state_index + 1) % rule_string.len()),
            turn_direction,
            new_internal_state: 0,
        });
        
        state_index += 1;
        
        if state_index >= 256 {
            break;
        }
    }
}

fn parse_state_rule(
    state_idx: usize, 
    rule: &str, 
    all_state_rules: &[&str], 
    rules: &mut BTreeMap<(usize, char), StateTransition>
) {
    // Handle internal multi-state
    if rule.contains(',') {
        let transitions: Vec<&str> = rule.split(',').collect();
        for (cell_idx, transition) in transitions.iter().enumerate() {
            // Parse direction and cell specification
            let (directions, next_state) = if let Some(transition_pos) = transition.find('>') {
                let directions = &transition[..transition_pos];
                let next_state_str = &transition[transition_pos + 1..];
                let next_state = next_state_str.parse::<usize>().unwrap_or(state_idx);
                (directions, next_state)
            } else {
                (*transition, state_idx)
            };
            
            let current_cell = state_char(cell_idx);
            
            // Check if direction string ends with a cell specifier
            let (direction_part, next_cell) = if let Some(last_char) = directions.chars().last() {
                if last_char.is_ascii_digit() {
                    let cell_idx = last_char.to_digit(10).unwrap_or(0) as usize;
                    let next_cell = state_char(cell_idx);
                    let direction_part = &directions[..directions.len() - 1];
                    (direction_part, next_cell)
                } else {
                    (directions, state_char((cell_idx + 1) % 2))
                }
            } else {
                (directions, state_char((cell_idx + 1) % 2))
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
        // Auto-cycle for simple multi-state format
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
    
    while i < directions.len() && cell_state_idx < 256 {
        let remaining = &directions[i..];
        let current_cell = state_char(cell_state_idx);
        let next_cell = state_char((cell_state_idx + 1) % directions.len());
        
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