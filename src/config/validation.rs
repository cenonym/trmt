use ratatui::style::Color;
use crate::config::Config;

pub fn validate_config(config: &Config) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();

    // Validate rule string
    if let Err(e) = validate_rule_string(&config.simulation.rule) {
        errors.push(format!("simulation.rule: {}", e));
    }

    // Validate char modes
    let head_modes = [
        config.display.direction_based_chars,
        config.display.randomize_heads,
    ].iter().filter(|&&x| x).count();
    
    let trail_modes = [
        config.display.direction_based_chars,
        config.display.randomize_trails,
    ].iter().filter(|&&x| x).count();
    
    if head_modes > 1 {
        errors.push("display: only one of direction_based_chars or randomize_heads can be true".to_string());
    }
    
    if trail_modes > 1 {
        errors.push("display: only one of direction_based_chars or randomize_trails can be true".to_string());
    }

    // Validate colors
    for (i, color) in config.display.colors.iter().enumerate() {
        if let Err(e) = validate_color(color) {
            errors.push(format!("display.colors[{}]: {}", i, e));
        }
    }

    // Validate numeric ranges
    if config.simulation.heads == 0 || config.simulation.heads > 256 {
        errors.push("simulation.heads: must be between 1 and 256".to_string());
    }

    if config.simulation.speed_ms <= 0.0 {
        errors.push("simulation.speed_ms: must be positive".to_string());
    }

    // Validate display characters
    if config.display.head_char.is_empty() || config.display.head_char.iter().any(|s| s.is_empty()) {
        errors.push("display.head_char: cannot be empty or contain empty strings".to_string());
    }
    if config.display.trail_char.is_empty() || config.display.trail_char.iter().any(|s| s.is_empty()) {
        errors.push("display.trail_char: cannot be empty or contain empty strings".to_string());
    }
    if config.display.cell_char.is_empty() {
        errors.push("display.cell_char: cannot be empty".to_string());
    }

    // Validate control keys
    let controls = [
        ("quit", &config.controls.quit),
        ("toggle", &config.controls.toggle),
        ("reset", &config.controls.reset),
        ("faster", &config.controls.faster),
        ("slower", &config.controls.slower),
        ("config_reload", &config.controls.config_reload),
        ("help", &config.controls.help),
        ("statusbar", &config.controls.statusbar),
        ("randomize_seed", &config.controls.randomize_seed),
        ("randomize_rule", &config.controls.randomize_rule),
    ];

    for (name, key) in &controls {
        if key.is_empty() {
            errors.push(format!("controls.{}: cannot be empty", name));
        }
    }

    // Check for duplicate key bindings
    let mut seen_keys = std::collections::HashSet::new();
    for (name, key) in &controls {
        if !seen_keys.insert(key) {
            errors.push(format!("controls.{}: duplicate key binding '{}'", name, key));
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn validate_rule_string(rule: &str) -> Result<(), String> {
    if rule.is_empty() {
        return Ok(());
    }

    // Handle standard notation
    if rule.trim().starts_with('{') {
        return validate_standard_notation(rule);
    }

    // Handle explicit state rules
    if rule.contains(',') {
        let combinations: Vec<&str> = rule.split(',').collect();
        for combo in combinations {
            if combo.is_empty() {
                return Err("rule combination cannot be empty".to_string());
            }
            
            let state_parts: Vec<&str> = combo.split(':').collect();
            for state_part in state_parts {
                validate_direction_string(state_part)?;
            }
        }
        return Ok(());
    }

    // Split by colon for multi-state rules
    let state_rules: Vec<&str> = rule.split(':').collect();
    
    for state_rule in state_rules {
        if state_rule.is_empty() {
            return Err("state rule cannot be empty".to_string());
        }
        validate_direction_string(state_rule)?;
    }

    Ok(())
}

fn validate_standard_notation(rule: &str) -> Result<(), String> {
    let cleaned = rule.replace(" ", "").replace("\n", "");
    
    if !cleaned.starts_with('{') || !cleaned.ends_with('}') {
        return Err("standard notation must start and end with braces".to_string());
    }
    
    // Basic brace balance check
    let mut brace_count = 0;
    for ch in cleaned.chars() {
        match ch {
            '{' => brace_count += 1,
            '}' => {
                brace_count -= 1;
                if brace_count < 0 {
                    return Err("unmatched closing brace".to_string());
                }
            },
            _ => {}
        }
    }
    
    if brace_count != 0 {
        return Err("unmatched braces".to_string());
    }
    
    // Check for valid triplet patterns
    let mut i = 0;
    let chars: Vec<char> = cleaned.chars().collect();
    
    while i < chars.len() {
        if i + 2 < chars.len() && chars[i] == '{' && chars[i+1] != '{' {
            let mut j = i + 1;
            let mut triplet_content = String::new();
            let mut brace_depth = 1;
            
            while j < chars.len() && brace_depth > 0 {
                match chars[j] {
                    '{' => brace_depth += 1,
                    '}' => brace_depth -= 1,
                    _ => {}
                }
                
                if brace_depth > 0 {
                    triplet_content.push(chars[j]);
                }
                j += 1;
            }
            
            if triplet_content.matches(',').count() == 2 {
                validate_triplet(&triplet_content)?;
            }
            
            i = j;
        } else {
            i += 1;
        }
    }
    
    Ok(())
}

fn validate_triplet(triplet: &str) -> Result<(), String> {
    let values: Vec<&str> = triplet.split(',').collect();
    if values.len() != 3 {
        return Ok(());
    }
    
    // Validate cell state
    if let Ok(cell_state) = values[0].trim().parse::<usize>() {
        if cell_state > 255 {
            return Err(format!("cell state {} is out of range (0-255)", cell_state));
        }
    } else {
        return Err(format!("invalid cell state: {}", values[0]));
    }
    
    // Validate turn direction
    if let Ok(turn_dir) = values[1].trim().parse::<usize>() {
        if ![1, 2, 4, 8].contains(&turn_dir) {
            return Err(format!("invalid turn direction: {}. Must be 1 (no turn), 2 (right), 4 (u-turn), or 8 (left)", turn_dir));
        }
    } else {
        return Err(format!("invalid turn direction: {}", values[1]));
    }
    
    // Validate internal state
    if values[2].trim().parse::<usize>().is_err() {
        return Err(format!("invalid internal state: {}", values[2]));
    }
    
    Ok(())
}

fn validate_direction_string(rule: &str) -> Result<(), String> {
    // Check if rule has state transition indicator
    let directions = if let Some(transition_pos) = rule.find('>') {
        let next_state_str = &rule[transition_pos + 1..];
        // Validate state number
        if !next_state_str.chars().all(|c| c.is_ascii_digit()) {
            return Err(format!("invalid state number '{}' in rule '{}'", next_state_str, rule));
        }
        &rule[..transition_pos] // Validate only the directions
    } else {
        rule
    };
    
    let mut i = 0;
    while i < directions.len() {
        let remaining = &directions[i..];
        
        if remaining.starts_with("NW") || remaining.starts_with("NE") ||
        remaining.starts_with("SW") || remaining.starts_with("SE") {
            i += 2;
        } else if let Some(c) = remaining.chars().next() {
            match c {
                'L' | 'R' | 'U' | 'D' | 'N' | 'S' | 'E' | 'W' | '0'..='9' => i += 1,
                _ => return Err(format!("invalid character '{}' in rule '{}'", c, rule)),
            }
        } else {
            break;
        }
    }
    Ok(())
}

fn validate_color(color_str: &str) -> Result<(), String> {
    // Validate hex
    if color_str.starts_with('#') && color_str.len() == 7 {
        if color_str[1..].chars().all(|c| c.is_ascii_hexdigit()) {
            return Ok(());
        } else {
            return Err(format!("invalid hex color format '{}'", color_str));
        }
    }
    
    // Validate rgb
    if color_str.starts_with("rgb(") && color_str.ends_with(')') {
        let inner = &color_str[4..color_str.len()-1];
        let parts: Vec<&str> = inner.split(',').map(|s| s.trim()).collect();
        if parts.len() != 3 {
            return Err(format!("RGB format must have 3 components: '{}'", color_str));
        }
        for part in parts {
            if part.parse::<u8>().is_err() {
                return Err(format!("invalid RGB component '{}' in '{}'", part, color_str));
            }
        }
        return Ok(());
    }

    // Validate 256-color
    if let Ok(_index) = color_str.parse::<u8>() {
        return Ok(());
    }

    Err(format!("invalid color format '{}'. Supported formats: #RRGGBB, rgb(r,g,b), or 0-255", color_str))
}

pub fn parse_color(color_str: &str) -> Color {
    // Parse hex colors
    if color_str.starts_with('#') && color_str.len() == 7 {
        if let (Ok(r), Ok(g), Ok(b)) = (
            u8::from_str_radix(&color_str[1..3], 16),
            u8::from_str_radix(&color_str[3..5], 16),
            u8::from_str_radix(&color_str[5..7], 16),
        ) {
            return Color::Rgb(r, g, b);
        }
    }
    
    // Parse rgb colors
    if color_str.starts_with("rgb(") && color_str.ends_with(')') {
        let inner = &color_str[4..color_str.len()-1];
        let parts: Vec<&str> = inner.split(',').map(|s| s.trim()).collect();
        if parts.len() == 3 {
            if let (Ok(r), Ok(g), Ok(b)) = (
                parts[0].parse::<u8>(),
                parts[1].parse::<u8>(),
                parts[2].parse::<u8>(),
            ) {
                return Color::Rgb(r, g, b);
            }
        }
    }

    // Parse 256-colors
    if let Ok(index) = color_str.parse::<u8>() {
        return Color::Indexed(index);
    }

    // Fallback to white for invalid colors
    Color::White
}
