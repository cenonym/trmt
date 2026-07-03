use rustc_hash::FxHashMap;
use super::grid::Grid;
use super::heads::Head;
use super::rules::Direction;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DetectionStatus {
    Running,
    Stalled { at_step: u64 },
    Cycle { at_step: u64, period: u64 },
}

// splitmix64 finalizer
#[inline]
fn mix64(mut z: u64) -> u64 {
    z = z.wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

#[inline]
fn dir_code(d: Direction) -> u64 {
    match d {
        Direction::Up => 0, Direction::Down => 1,
        Direction::Left => 2, Direction::Right => 3,
        Direction::UpLeft => 4, Direction::UpRight => 5,
        Direction::DownLeft => 6, Direction::DownRight => 7,
    }
}

// Empty cells contribute nothing, stored or absent
#[inline]
fn cell_contrib(x: i32, y: i32, c: char) -> u64 {
    if c == Grid::EMPTY {
        return 0;
    }
    mix64(((x as u64 & 0xFFFF) << 48) | ((y as u64 & 0xFFFF) << 32) | c as u64)
}

#[inline]
fn head_contrib(index: usize, x: i32, y: i32, dir: Direction, istate: usize) -> u64 {
    let word = ((index as u64) << 56)
        | ((x as u64 & 0xFFFF) << 40)
        | ((y as u64 & 0xFFFF) << 24)
        | (dir_code(dir) << 21)
        | (istate as u64 & 0x1F_FFFF);
    mix64(word ^ 0xDEAD_BEEF_CAFE_F00D)
}

#[derive(Debug, Clone, PartialEq)]
struct Snapshot {
    tape: FxHashMap<(i32, i32), char>,
    heads: Vec<(i32, i32, Direction, usize)>,
}

impl Snapshot {
    fn capture(grid: &Grid, heads: &[Head]) -> Self {
        Self {
            tape: grid.tape.iter()
                .filter(|&(_, &c)| c != Grid::EMPTY)
                .map(|(&pos, &c)| (pos, c))
                .collect(),
            heads: heads.iter()
                .map(|h| (h.x, h.y, h.direction, h.internal_state))
                .collect(),
        }
    }
}

fn full_hash(grid: &Grid, heads: &[Head]) -> u64 {
    let mut h = 0u64;
    for (&(x, y), &c) in &grid.tape {
        h ^= cell_contrib(x, y, c);
    }
    for (i, head) in heads.iter().enumerate() {
        h ^= head_contrib(i, head.x, head.y, head.direction, head.internal_state);
    }
    h
}

#[derive(Debug)]
pub struct CycleDetector {
    hash: u64,
    saved_hash: u64,
    snapshot: Snapshot,
    power: u64,
    lam: u64,
    status: DetectionStatus,
}

impl CycleDetector {
    pub fn new() -> Self {
        Self {
            hash: 0,
            saved_hash: 0,
            snapshot: Snapshot { tape: FxHashMap::default(), heads: Vec::new() },
            power: 1,
            lam: 0,
            status: DetectionStatus::Running,
        }
    }

    pub fn reset_with(&mut self, grid: &Grid, heads: &[Head]) {
        self.hash = full_hash(grid, heads);
        self.saved_hash = self.hash;
        self.snapshot = Snapshot::capture(grid, heads);
        self.power = 1;
        self.lam = 0;
        self.status = DetectionStatus::Running;
    }

    #[inline]
    pub fn cell_delta(&mut self, x: i32, y: i32, old: char, new: char) {
        self.hash ^= cell_contrib(x, y, old) ^ cell_contrib(x, y, new);
    }

    #[inline]
    pub fn head_delta(
        &mut self,
        index: usize,
        old: (i32, i32, Direction, usize),
        new: (i32, i32, Direction, usize),
    ) {
        self.hash ^= head_contrib(index, old.0, old.1, old.2, old.3)
            ^ head_contrib(index, new.0, new.1, new.2, new.3);
    }

    pub fn mark_stalled(&mut self, at_step: u64) {
        if self.status == DetectionStatus::Running {
            self.status = DetectionStatus::Stalled { at_step };
        }
    }

    pub fn on_step_end(&mut self, grid: &Grid, heads: &[Head], steps: u64) {
        if self.status != DetectionStatus::Running {
            return;
        }
        self.lam += 1;
        // Only a structural match proves a cycle
        if self.hash == self.saved_hash && self.snapshot == Snapshot::capture(grid, heads) {
            self.status = DetectionStatus::Cycle { at_step: steps, period: self.lam };
            return;
        }
        if self.lam == self.power {
            self.power <<= 1;
            self.lam = 0;
            self.saved_hash = self.hash;
            self.snapshot = Snapshot::capture(grid, heads);
        }
    }

    pub fn status(&self) -> DetectionStatus {
        self.status
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::style::Color;

    fn make_head(x: i32, y: i32) -> Head {
        Head::new(x, y, Color::White)
    }

    #[test]
    fn incremental_hash_matches_full_recompute() {
        let mut grid = Grid::new();
        let mut heads = vec![make_head(3, 4)];
        let mut det = CycleDetector::new();
        det.reset_with(&grid, &heads);

        let writes = [(1, 2, 'B'), (5, 5, 'C'), (1, 2, 'D'), (5, 5, 'A')];
        for &(x, y, new) in &writes {
            let old = grid.get_cell(x, y);
            grid.set_cell(x, y, new, Color::White, None, false);
            det.cell_delta(x, y, old, new);
        }
        let old = (heads[0].x, heads[0].y, heads[0].direction, heads[0].internal_state);
        heads[0].x = 7;
        heads[0].direction = Direction::Left;
        heads[0].internal_state = 2;
        let new = (7, 4, Direction::Left, 2);
        det.head_delta(0, old, new);

        assert_eq!(det.hash, full_hash(&grid, &heads));
    }

    #[test]
    fn detects_cycle_with_exact_period() {
        let grid = Grid::new();
        let mut heads = vec![make_head(0, 0)];
        let mut det = CycleDetector::new();
        det.reset_with(&grid, &heads);

        let orbit = [
            (1, 0, Direction::Right),
            (1, 1, Direction::Down),
            (0, 1, Direction::Left),
            (0, 0, Direction::Up),
        ];
        for step in 1..=40u64 {
            let i = ((step - 1) % 4) as usize;
            let old = (heads[0].x, heads[0].y, heads[0].direction, heads[0].internal_state);
            heads[0].x = orbit[i].0;
            heads[0].y = orbit[i].1;
            heads[0].direction = orbit[i].2;
            det.head_delta(0, old, (orbit[i].0, orbit[i].1, orbit[i].2, 0));
            det.on_step_end(&grid, &heads, step);
            if det.status() != DetectionStatus::Running {
                break;
            }
        }
        match det.status() {
            DetectionStatus::Cycle { period, .. } => assert_eq!(period, 4),
            other => panic!("expected cycle, got {:?}", other),
        }
    }

    #[test]
    fn hash_collision_without_structural_match_is_rejected() {
        let grid = Grid::new();
        let heads = vec![make_head(0, 0)];
        let mut det = CycleDetector::new();
        det.reset_with(&grid, &heads);
        // Forge a collision, same hash but different structure
        det.snapshot.heads[0].0 = 99;
        det.on_step_end(&grid, &heads, 1);
        assert_eq!(det.status(), DetectionStatus::Running);
    }

    #[test]
    fn reset_clears_status_and_rebuilds_hash() {
        let mut grid = Grid::new();
        let heads = vec![make_head(2, 2)];
        let mut det = CycleDetector::new();
        det.mark_stalled(10);
        grid.set_cell(1, 1, 'B', Color::White, None, false);
        det.reset_with(&grid, &heads);
        assert_eq!(det.status(), DetectionStatus::Running);
        assert_eq!(det.hash, full_hash(&grid, &heads));
    }

    #[test]
    fn stall_is_latched_and_not_overwritten() {
        let grid = Grid::new();
        let heads = vec![make_head(0, 0)];
        let mut det = CycleDetector::new();
        det.reset_with(&grid, &heads);
        det.mark_stalled(5);
        det.on_step_end(&grid, &heads, 6);
        assert_eq!(det.status(), DetectionStatus::Stalled { at_step: 5 });
    }
}
