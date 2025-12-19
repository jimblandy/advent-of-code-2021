/*! An iterator over [Band]s. */

use super::{Edge, Point, is_vertical};
use binary_heap_plus::BinaryHeap;
use std::cmp::{self, max, min};
use std::ops::{Range, RangeInclusive};

/// A range of rows within which the ranges of columns that lie within the shape
/// do not change, and within which red tiles appear only in the top row.
///
/// Since bands don't overlap, every red tile appears in exactly one band.
pub struct Band {
    pub rows: RangeInclusive<u64>,
    pub runs: Vec<Range<u64>>,

    /// Columns where there are red tiles. These all fall in the first row of
    /// `rows`.
    pub reds: Vec<u64>,
}

pub struct BandIter {
    /// The starting row of the next band, if any.
    next_row: u64,

    /// All edges that touch `next_row`.
    ///
    /// These are in a heap ordered by lower end (`edge_bottom(e)`).
    ///
    /// Since our first band starts at the top of the shape, and the shape we're
    /// dealing with is connected, this should only be empty when iteration is
    /// over.
    next: EdgesByBottom,

    /// Edges that we have not yet reached in our top-to-bottom
    /// traversal.
    unreached: EdgesByTop,
}

struct EdgeByTop;
struct EdgeByBottom;
type EdgesByTop = BinaryHeap<Edge, EdgeByTop>;
type EdgesByBottom = BinaryHeap<Edge, EdgeByBottom>;

impl compare::Compare<Edge> for EdgeByTop {
    fn compare(&self, l: &Edge, r: &Edge) -> cmp::Ordering {
        edge_top(l).cmp(&edge_top(r))
            .then(edge_left(l).cmp(&edge_left(r)))
    }
}

impl compare::Compare<Edge> for EdgeByBottom {
    fn compare(&self, l: &Edge, r: &Edge) -> cmp::Ordering {
        edge_bottom(l).cmp(&edge_bottom(r))
            .then(edge_left(l).cmp(&edge_left(r)))
    }
}

impl BandIter {
    fn from_edges(edges: impl IntoIterator<Item = Edge>) -> Self {
        let edges = edges.into_iter();

        // Build a vector from the iterator, and assert that the edges actually
        // form a connected loop.
        let mut prev_end = None; 
        let edges: Vec<Edges> = edges
            .inspect(|edge| {
                if let Some(prev_end) = prev_end {
                    assert_eq!(prev_end, edge.start());
                }
                prev_end = Some(edge.end());
            })
            .collect();
        assert!(edges.len() > 1);
        assert_eq!(Some(edges.first().unwrap().start()), prev_end);

        // To start with, add all the edges to the `unreached` heap, from which
        // we can draw them in order of their top rows.
        let mut unreached = EdgesByTop::from_vec_cmp(edges, EdgeByTop);

        // Find an edge along the top of the shape.
        let top_edge = unreached.peek().unwrap();

        // Since we need to start iteration at the top of the shape, initialize
        // `next` by drawing all edges that touch the top of the shape out of
        // `unreached`.
        let mut next = EdgesByBottom::from_vec_cmp(vec![], EdgeByBottom);
        let next_row = edge_top(top_edge);
        while let Some(v) = unreached.peek_mut().filter(|e| edge_top(e) == next_row) {
            next.push(v.pop());
        }

        BandIter { next_row, next, unreached }
    }
}

impl Iterator for BandIter {
    /// Non-overlapping runs of columns, sorted by column.
    type Item = Band;

    fn next(&mut self) -> Option<Band> {
        let mut runs = vec![];
        let mut reds = vec![];

        let band_top = self.next_row;

        // Drop all edges from `next` whose bottoms are at `next_row`, and note
        // the columns of their red squares.
        while let Some(maybe_done) = self.next.peek_mut() {
            let maybe_done_bottom = edge_bottom(&*maybe_done);
            assert!(band_top <= maybe_done_bottom);
            if band_top < maybe_done_bottom {
                break;
            }
            let done = maybe_done.pop();
            if done.start.0 == band_top() {
                reds.push(done.start.1);
            }
            if done.end.0 == band_top() {
                reds.push(done.end.1);
            }
            if done.start.0 == done.end.0 {
                runs.push(done.start.1 .. done.end.1 + 1);
            } else {
                runs.push(done.start.1 .. done.end.1 + 1);
            }
        }

        // Since `next` contains all edges that touch `next_row`, and the shape
        // is connected, if `next` is empty now, then we're at the end of the
        // whole iteration.
        let Some(next) = self.next.peek() else {
            assert!(self.unreached.is_empty());
            let rows = band_top ..= band_top;

        };
        
        // At this point, if `next` contains any edges at all, they all extend
        // below `next_row`. `next` may be empty.

        // Choose an ending row for this band. Extend as far down we can without
        // enclosing any more red squares.
        let mut band_bottom = None;

        // Consider the next edge in `next`, if any.
        if let Some(next) = self.next.peek() {
            band_bottom = Some(edge_bottom(&next) - 1);
        }

        // Consider the next edges in `unreached`, if any.
        if let Some(next_unreached) = self.unreached.peek() {
            // The next edge in `unreached` had better be starting below whatever was in `next`.
            let next_unreached_top = edge_top(next_unreached.end());
            assert!(rows.start < next_unreached_top);
            rows.end = min(rows.end, next_unreached_top - 1);
        }
        

        let e = self.next.pop()?;
        let mut last_row = edge_bottom(&e);
        
        // If there's another edge coming up, that might limit the size of this band.

        // 
    }
}

fn intersection<T: cmp::Ord + Copy>(a: Range<T>, b: Range<T>) -> Option<Range<T>> {
    let candidate = max(a.start, b.start) .. min(a.end, b.end);
    (candidate.start < candidate.end).then_some(candidate)
}

fn edge_top(e: &Edge) -> u64 {
    min(e.start().0, e.end().0)
}

fn edge_left(e: &Edge) -> u64 {
    min(e.start().1, e.end().1)
}

fn edge_bottom(e: &Edge) -> u64 {
    min(e.start().0, e.end().0)
}

fn edge_right(e: &Edge) -> u64 {
    min(e.start().1, e.end().1)
}

#[cfg(test)]
mod test {
    use super::{Band, BandIter, Direction, Vertical};

    #[test]
    fn empty() {
        let mut bands = BandIter::from_edges([]);
        assert(bands.next().is_none());
    }

    #[test]
    fn square() {
        let mut bands = BandIter::from_edges([
            (12,10) ..= (12,20),
            (12,20) ..= (22,20),
            (22,20) ..= (22,10),
            (22,10) ..= (12,10),
        ]);

        assert_eq!(bands.next(), Some(Band {
            rows: 12..=21,
            runs: vec![10..21],
            reds: vec![10, 20],
        }));
        assert_eq!(bands.next(), Some(Band {
            rows: 22..=22,
            runs: vec![10..21],
            reds: vec![10, 20],
        }));
        assert_eq!(bands.next(), None);
    }

    #[test]
    fn reversed_square() {
        let mut bands = BandIter::from_edges([
            (22,20) ..= (12,20),
            (12,20) ..= (12,10),
            (12,10) ..= (22,10),
            (22,10) ..= (22,20),
        ]);

        assert_eq!(bands.next(), Some(Band {
            rows: 12..=21,
            runs: vec![10..21],
            reds: vec![10, 20],
        }));
        assert_eq!(bands.next(), Some(Band {
            rows: 22..=22,
            runs: vec![10..21],
            reds: vec![10, 20],
        }));
        assert_eq!(bands.next(), None);
    }

    #[test]
    fn horizontal_line() {
        let mut bands = BandIter::from_edges([
            (12,10) ..= (12,20),
            (12,20) ..= (12,10),
        ]);

        assert_eq!(bands.next(), Some(Band {
            rows: 12..=12,
            runs: vec![10..21],
            reds: vec![10, 20],
        }));
        assert_eq!(bands.next(), None);
    }

    #[test]
    fn vertical_line() {
        let mut bands = BandIter::from_edges([
            (12,10) ..= (22,10),
            (22,10) ..= (12,10),
        ]);

        assert_eq!(bands.next(), Some(Band {
            rows: 12..=21,
            runs: vec![10..11],
            reds: vec![10],
        }));
        assert_eq!(bands.next(), Some(Band {
            rows: 22..=22,
            runs: vec![10..11],
            reds: vec![10],
        }));
        assert_eq!(bands.next(), None);
    }

    #[test]
    fn four_point_vertical_line() {
        let mut bands = BandIter::from_edges([
            (12,10) ..= (22,10),
            (22,10) ..= (42,10),
            (42,10) ..= (32,10),
            (32,10) ..= (12,10),
        ]);

        assert_eq!(bands.next(), Some(Band {
            rows: 12..=21,
            runs: vec![10..11],
            reds: vec![10],
        }));
        assert_eq!(bands.next(), Some(Band {
            rows: 22..=31,
            runs: vec![10..11],
            reds: vec![10],
        }));
        assert_eq!(bands.next(), Some(Band {
            rows: 32..=41,
            runs: vec![10..11],
            reds: vec![10],
        }));
        assert_eq!(bands.next(), Some(Band {
            rows: 42..=42,
            runs: vec![10..11],
            reds: vec![10],
        }));
        assert_eq!(bands.next(), None);
    }

    #[test]
    fn u_shape() {
        let mut bands = BandIter::from_edges([
            (10,10) ..= (10,20),
            (10,20) ..= (20,20),
            (20,20) ..= (20,30),
            (20,30) ..= (10,30),
            (10,30) ..= (10,40),
            (10,40) ..= (30,40),
            (30,40) ..= (30,10),
            (30,10) ..= (10,10),
        ]);

        assert_eq!(bands.next(), Some(Band {
            rows: 10..=19,
            runs: vec![10..11, 20..31],
            reds: vec![10, 20, 30, 40],
        }));
        assert_eq!(bands.next(), Some(Band {
            rows: 20..=29,
            runs: vec![10..31],
            reds: vec![20, 30],
        }));
        assert_eq!(bands.next(), Some(Band {
            rows: 30..=30,
            runs: vec![10..31],
            reds: vec![10, 40],
        }));
        assert_eq!(bands.next(), None);
    }

    #[test]
    fn j_shape() {
        let mut bands = BandIter::from_edges([
            (10,10) ..= (10,20),
            (10,20) ..= (30,20),
            (30,20) ..= (30,30),
            (30,30) ..= (20,30),
            (20,30) ..= (20,40),
            (20,40) ..= (40,40),
            (40,40) ..= (40,10),
            (40,10) ..= (10,10),
        ]);

        assert_eq!(bands.next(), Some(Band {
            rows: 10..=19,
            runs: vec![10..21],
            reds: vec![10, 20],
        }));
        assert_eq!(bands.next(), Some(Band {
            rows: 20..=29,
            runs: vec![10..21, 30..41],
            reds: vec![30, 40],
        }));
        assert_eq!(bands.next(), Some(Band {
            rows: 30..=39,
            runs: vec![10..41],
            reds: vec![20, 30],
        }));
        assert_eq!(bands.next(), Some(Band {
            rows: 40..=40,
            runs: vec![10..41],
            reds: vec![10, 40],
        }));
        assert_eq!(bands.next(), None);
    }

    #[test]
    fn lower_case_r_shape() {
        let mut bands = BandIter::from_edges([
            (10,10) ..= (10,40),
            (10,40) ..= (30,40),
            (30,40) ..= (30,30),
            (30,30) ..= (20,30),
            (20,30) ..= (20,10),
        ]);

        assert_eq!(bands.next(), Some(Band {
            rows: 10..=19,
            runs: vec![10..41],
            reds: vec![10, 40],
        }));
        // The row with the two inner corners is still
        // a single run, the full width of the figure.
        assert_eq!(bands.next(), Some(Band {
            rows: 20..=20,
            runs: vec![10..41],
            reds: vec![20, 30],
        }));
        // The band with two disjoin runs begins below
        // the row with the inner corners.
        assert_eq!(bands.next(), Some(Band {
            rows: 21..=29,
            runs: vec![10..21, 30..41],
            reds: vec![],
        }));
        // The row the ends the r's right side also introduces a one-row band.
        assert_eq!(bands.next(), Some(Band {
            rows: 30..=30,
            runs: vec![10..21, 30..41],
            reds: vec![30, 40],
        }));
        assert_eq!(bands.next(), Some(Band {
            rows: 31..=39,
            runs: vec![10..21],
            reds: vec![],
        }));
        assert_eq!(bands.next(), Some(Band {
            rows: 40..=40,
            runs: vec![10..21],
            reds: vec![10, 20],
        }));
        assert_eq!(bands.next(), None);
    }
}
