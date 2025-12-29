use std::cmp::{max, min};
use std::ops::RangeInclusive;

pub type Point = (u64, u64); // row, col
pub type Edge = RangeInclusive<Point>;

pub fn is_horizontal(e: &Edge) -> bool {
    e.start().0 == e.end().0
}

pub fn is_vertical(e: &Edge) -> bool {
    e.start().1 == e.end().1
}

pub fn goes_down(e: &Edge) -> bool {
    e.start().0 < e.end().0
}

pub fn are_connected(a: &Edge, b: &Edge) -> bool {
    a.start() == b.end() || b.start() == a.end()
}

pub fn edge_top(e: &Edge) -> u64 {
    min(e.start().0, e.end().0)
}

pub fn edge_left(e: &Edge) -> u64 {
    min(e.start().1, e.end().1)
}

pub fn edge_bottom(e: &Edge) -> u64 {
    max(e.start().0, e.end().0)
}

pub fn edge_right(e: &Edge) -> u64 {
    max(e.start().1, e.end().1)
}
