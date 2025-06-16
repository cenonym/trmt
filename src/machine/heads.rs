use ratatui::style::Color;
use std::collections::VecDeque;
use super::rules::Direction;

#[derive(Debug, Clone)]
pub struct Head {
    pub x: i32,
    pub y: i32,
    pub direction: Direction,
    pub internal_state: usize,
    pub previous_direction: Option<Direction>,
    pub color: Color,
    pub trail: VecDeque<(i32, i32)>,
}

impl Head {
    pub fn new(x: i32, y: i32, color: Color) -> Self {
        Self {
            x,
            y,
            direction: Direction::Up,
            previous_direction: None,
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

    pub fn set_direction(&mut self, new_direction: Direction) {
        self.previous_direction = Some(self.direction);
        self.direction = new_direction;
    }
}