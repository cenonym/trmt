use ratatui::style::Color;
use rustc_hash::FxHashMap;

#[derive(Debug, Clone)]
pub struct Grid {
    pub tape: FxHashMap<(i32, i32), char>,
    pub tape_colors: FxHashMap<(i32, i32), Color>,
}

impl Grid {
    pub fn new() -> Self {
        Self {
            tape: FxHashMap::with_capacity_and_hasher(8192, Default::default()),
            tape_colors: FxHashMap::with_capacity_and_hasher(8192, Default::default()),
        }
    }

    #[inline(always)]
    pub fn get_cell(&self, x: i32, y: i32) -> char {
        self.tape.get(&(x, y)).copied().unwrap_or('A')
    }

    pub fn set_cell(&mut self, x: i32, y: i32, state: char, color: Color, state_based_colors: bool) {
        if state == 'A' && !state_based_colors {
            self.tape.remove(&(x, y));
            self.tape_colors.remove(&(x, y));
        } else {
            self.tape.insert((x, y), state);
            self.tape_colors.insert((x, y), color);
        }
    }

    pub fn clear(&mut self) {
        self.tape.clear();
        self.tape_colors.clear();
    }
}