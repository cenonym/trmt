use ratatui::style::Color;
use rustc_hash::FxHashMap;

#[derive(Debug, Clone)]
pub struct Grid {
    pub tape: FxHashMap<(i32, i32), char>,
    pub tape_colors: FxHashMap<(i32, i32), Color>,
    pub tape_chars: FxHashMap<(i32, i32), String>,
}

impl Grid {
    pub fn new() -> Self {
        Self {
            tape: FxHashMap::with_capacity_and_hasher(8192, Default::default()),
            tape_colors: FxHashMap::with_capacity_and_hasher(8192, Default::default()),
            tape_chars: FxHashMap::with_capacity_and_hasher(8192, Default::default()),
        }
    }

    #[inline(always)]
    pub fn get_cell(&self, x: i32, y: i32) -> char {
        self.tape.get(&(x, y)).copied().unwrap_or('A')
    }

    pub fn set_cell(&mut self, x: i32, y: i32, state: char, color: Color, display_char: Option<String>, state_based_colors: bool) {
        if state == 'A' && !state_based_colors {
            self.tape.remove(&(x, y));
            self.tape_colors.remove(&(x, y));
            self.tape_chars.remove(&(x, y));
        } else {
            self.tape.insert((x, y), state);
            self.tape_colors.insert((x, y), color);
            
            if let Some(char) = display_char {
                self.tape_chars.insert((x, y), char);
            }
        }
    }

    pub fn clear(&mut self) {
        self.tape.clear();
        self.tape_colors.clear();
        self.tape_chars.clear();
    }
}