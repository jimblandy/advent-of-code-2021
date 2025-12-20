/*! An iterator over [Band]s.

We assert that there are no edges that traverse the same squares as other edges.
This simplifies things:

- The shape cannot be self-intersecting.

- A horizontal line through the shape can't exit it and re-enter it in the same
  square.

We could cope with all these cases, but it makes the code more complicated.

*/

use super::Edge;
use binary_heap_plus::{BinaryHeap, PeekMut};
use std::cmp::{self, Ordering, max, min};
use std::ops::{Range, RangeInclusive};

/// A range of rows within which the ranges of columns that lie within the shape
/// do not change, and within which red tiles appear only in the top row.
///
/// Since bands don't overlap, every red tile appears in exactly one band.
#[derive(Debug, Eq, PartialEq)]
pub struct Band {
    pub rows: RangeInclusive<u64>,
    pub runs: Vec<Range<u64>>,

    /// Columns in the top row of the band where there are red tiles.
    /// Red tiles appear only on the top row of the band.
    pub reds: Vec<u64>,
}

#[derive(Debug)]
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
    fn compare(&self, l: &Edge, r: &Edge) -> Ordering {
        edge_top(l)
            .cmp(&edge_top(r))
            .then(edge_left(l).cmp(&edge_left(r)))
            .reverse()
    }
}

impl compare::Compare<Edge> for EdgeByBottom {
    fn compare(&self, l: &Edge, r: &Edge) -> Ordering {
        edge_bottom(l)
            .cmp(&edge_bottom(r))
            .then(edge_left(l).cmp(&edge_left(r)))
            .reverse()
    }
}

impl BandIter {
    fn from_edges(edges: impl IntoIterator<Item = Edge>) -> Self {
        let edges = edges.into_iter();

        // Build a vector from the iterator, and assert that
        // - edges' endpoints are at different locations
        // - edges are either horizontal or vertical
        // - the edges actually form a connected loop
        let mut prev_end = None;
        let edges: Vec<Edge> = edges
            .inspect(|edge| {
                assert!(
                    edge.start() != edge.end(),
                    "edge's endpoints are not different"
                );
                assert!(
                    edge.start().0 == edge.end().0 || edge.start().1 == edge.end().1,
                    "edge is neither horizontal nor vertical"
                );
                if let Some(prev_end) = prev_end {
                    assert_eq!(prev_end, *edge.start());
                }
                prev_end = Some(*edge.end());
            })
            .collect();
        assert!(edges.len() > 1);
        assert_eq!(Some(*edges.first().unwrap().start()), prev_end);

        // Start with all the edges in the `unreached` heap. This will help us
        // find the top edges, which we must move to `next`.
        let mut unreached = EdgesByTop::from_vec_cmp(edges, EdgeByTop);

        // Pick a random edge along the top of the shape. This tells us the top
        // row of the first band iteration will produce.
        let top_edge = unreached.peek().unwrap();
        let next_row = edge_top(top_edge);

        // Since `next` must include all edges that contribute to `next_row`,
        // draw out all such edges from `unreached`.
        let mut next = EdgesByBottom::from_vec_cmp(vec![], EdgeByBottom);
        while let Some(v) = unreached.peek_mut().filter(|e| edge_top(e) == next_row) {
            next.push(PeekMut::pop(v));
        }

        BandIter {
            next_row,
            next,
            unreached,
        }
    }
}

/// Push `range` onto the end of `runs`, merging overlapping ranges.
fn push_range(runs: &mut Vec<Range<u64>>, range: Range<u64>) {
    match runs.last_mut() {
        // If this is the first range, just add it to the vector.
        None => {
            runs.push(range);
        }

        // If there is a gap between this range and the previous one, just add
        // the new range to the end.
        Some(last) if last.end < range.start => {
            runs.push(range);
        }

        // Otherwise, extend the prior range to enclose this one.
        Some(last) => {
            last.end = max(last.end, range.end);
        }
    }
}

impl Iterator for BandIter {
    /// Non-overlapping runs of columns, sorted by column.
    type Item = Band;

    fn next(&mut self) -> Option<Band> {
        eprintln!("BandIter::next:");
        eprintln!("    next_row:  {:?}", self.next_row);
        eprintln!("    next:      {:?}", self.next);
        eprintln!("    unreached: {:?}", self.unreached);

        // Since the shape is connected, if `next` is empty, then there are no
        // more bands to produce.
        if self.next.is_empty() {
            return None;
        }

        let band_top = self.next_row;

        // Since `next` contains all edges that touch `next_row`, it contains
        // all the information we need to build the band's `runs` and `reds`
        // vectors.

        // First, find all the red tiles in the band's top row.
        let mut reds: Vec<u64> = self
            .next
            .iter()
            .flat_map(|edge| {
                let mut a = None;
                let mut b = None;
                if edge.start().0 == edge.end().0 {
                    // It had better lie entirely in this band's top row.
                    assert_eq!(edge.start().0, band_top);
                    a = Some(edge.start().1);
                    b = Some(edge.end().1);
                } else if edge.start().0 == band_top || edge.end().0 == band_top {
                    // A vertical edge might contribute either its top or bottom red
                    // tile.
                    a = Some(edge.start().1);
                }

                a.into_iter().chain(b)
            })
            .collect();
        reds.sort(); // OMG we are sorting an array of integers

        // Every red tile is both the beginning of one edge, and the end of another,
        // and `next` is supposed to contain all relevant edges, so we should have
        // seen each red time mentioned twice. Drop duplicates.
        let with_dups = reds.len();
        reds.dedup();
        assert_eq!(reds.len() * 2, with_dups);

        // Build `runs` by traversing edges that touch the top of the band from
        // left to right.
        //
        // If we have verticals joined by a horizontal, ensure the verticals
        // appear before and after the horizontal.
        let mut edges: Vec<Edge> = self.next.iter().cloned().collect();
        edges.sort_by(|a, b| {
            let by_start = edge_left(a).cmp(&edge_left(b));
            let by_end = edge_right(a).cmp(&edge_right(b));
            by_start.then(by_end)
        });
        eprintln!("sorted edges: {edges:?}");
        let mut edges = edges.into_iter();

        // Track transitions into and out of the shape across rising and
        // descending edges. We can reject self-intersection and shared edges
        // here.
        enum State {
            Outside,
            InUp(Edge),
            InDown(Edge),
        }
        let mut state = State::Outside;
        let mut runs: Vec<Range<u64>> = vec![];
        while let Some(edge) = edges.next() {
            eprintln!("    considering edge {edge:?}");
            let orientation = edge.start().0.cmp(&edge.end().0);
            let next_state;
            match state {
                State::Outside => match orientation {
                    Ordering::Less => {
                        // Entering the shape via a downgoing edge.
                        next_state = State::InDown(edge);
                    }
                    Ordering::Equal => {
                        // We should never encounter a horizontal edge outside the shape; we should
                        // always see a vertical edge introducing it.
                        panic!("bare horizontal edge: {edge:?}");
                    }
                    Ordering::Greater => {
                        // Entering the shape via an upgoing edge.
                        next_state = State::InUp(edge);
                    }
                },
                State::InUp(entered) => match orientation {
                    Ordering::Less => {
                        // We entered the shape via an upgoing edge, and now
                        // we're exiting via a downgoing edge. But if the edge
                        // ends at the top of the band,
                        ... // then, what?
                        push_range(&mut runs, entered.start().1..edge.start().1 + 1);
                        next_state = State::Outside;
                    }
                    Ordering::Equal => {
                        // We entered the shape via an upgoing edge, and have
                        // encountered a horizontal edge reaching into the
                        // interior. We'll still be within the shape after this
                        // edge, and we'll create the full range when we finally
                        // exit the shape, so we don't need to record a range,
                        // or change state.
                        next_state = State::InUp(entered);
                    }
                    Ordering::Greater => {
                        // We entered the shape via an upgoing edge, but then
                        // encountered another upgoing edge? The shape must be
                        // self-intersecting.
                        panic!("Shape is self-intersecting: {entered:?}, then {edge:?}");
                    }
                },
                State::InDown(entered) => match orientation {
                    Ordering::Less => {
                        // We entered the shape via a downgoing edge, but then
                        // encountered another downgoing edge? The shape must be
                        // self-intersecting.
                        panic!("Shape is self-intersecting: {entered:?}, then {edge:?}");
                    }
                    Ordering::Equal => {
                        // We entered the shape via a downgoing edge, and have
                        // encountered a horizontal edge reaching into the
                        // interior. We'll still be within the shape after this
                        // edge, and we'll create the full range when we finally
                        // exit the shape, so we don't need to record a range,
                        // or change state.
                        next_state = State::InDown(entered);
                    }
                    Ordering::Greater => {
                        // We entered the shape via a downgoing edge, and now we
                        // are leaving it via an upgoing edge.
                        push_range(&mut runs, entered.start().1..edge.start().1 + 1);
                        next_state = State::Outside;
                    }
                },
            }
            state = next_state;
        }

        // Drop all edges from `next` that end at the top of this band.
        while let Some(maybe_done) = self.next.peek_mut() {
            let maybe_done_bottom = edge_bottom(&*maybe_done);
            assert!(band_top <= maybe_done_bottom);
            if band_top < maybe_done_bottom {
                break;
            }
            PeekMut::pop(maybe_done);
        }

        // The band extends up to (but not including) the next row with red
        // tiles.
        let band_bottom = match (self.next.peek(), self.unreached.peek()) {
            (None, None) => band_top,
            (Some(next), None) => edge_bottom(next) - 1,
            (None, Some(unreached)) => edge_top(unreached) - 1,
            (Some(next), Some(unreached)) => min(edge_bottom(next), edge_top(unreached)) - 1,
        };
        assert!(band_top <= band_bottom);

        // Choose the next row, and bring in any new relevant edges.
        self.next_row = band_bottom + 1;
        while let Some(v) = self
            .unreached
            .peek_mut()
            .filter(|e| edge_top(e) == self.next_row)
        {
            self.next.push(PeekMut::pop(v));
        }

        let rows = band_top..=band_bottom;

        Some(Band { rows, runs, reds })
    }
}

fn intersection<T: cmp::Ord + Copy>(a: Range<T>, b: Range<T>) -> Option<Range<T>> {
    let candidate = max(a.start, b.start)..min(a.end, b.end);
    (candidate.start < candidate.end).then_some(candidate)
}

fn is_vertical(e: &Edge) -> bool {
    e.start().1 == e.end().1
}

fn goes_down(e: &Edge) -> bool {
    e.start().0 < e.end().0
}

fn are_connected(a: &Edge, b: &Edge) -> bool {
    a.start() == b.end() || b.start() == a.end()
}

fn edge_top(e: &Edge) -> u64 {
    min(e.start().0, e.end().0)
}

fn edge_left(e: &Edge) -> u64 {
    min(e.start().1, e.end().1)
}

fn edge_bottom(e: &Edge) -> u64 {
    max(e.start().0, e.end().0)
}

fn edge_right(e: &Edge) -> u64 {
    max(e.start().1, e.end().1)
}

#[cfg(test)]
mod test {
    use super::{Band, BandIter};

    #[test]
    fn square() {
        let mut bands = BandIter::from_edges([
            (12, 10)..=(12, 20),
            (12, 20)..=(22, 20),
            (22, 20)..=(22, 10),
            (22, 10)..=(12, 10),
        ]);

        assert_eq!(
            bands.next(),
            Some(Band {
                rows: 12..=21,
                runs: vec![10..21],
                reds: vec![10, 20],
            })
        );
        assert_eq!(
            bands.next(),
            Some(Band {
                rows: 22..=22,
                runs: vec![10..21],
                reds: vec![10, 20],
            })
        );
        assert_eq!(bands.next(), None);
    }

    #[test]
    fn reversed_square() {
        let mut bands = BandIter::from_edges([
            (22, 20)..=(12, 20),
            (12, 20)..=(12, 10),
            (12, 10)..=(22, 10),
            (22, 10)..=(22, 20),
        ]);

        assert_eq!(
            bands.next(),
            Some(Band {
                rows: 12..=21,
                runs: vec![10..21],
                reds: vec![10, 20],
            })
        );
        assert_eq!(
            bands.next(),
            Some(Band {
                rows: 22..=22,
                runs: vec![10..21],
                reds: vec![10, 20],
            })
        );
        assert_eq!(bands.next(), None);
    }

    #[test]
    fn horizontal_line() {
        let mut bands = BandIter::from_edges([(12, 10)..=(12, 20), (12, 20)..=(12, 10)]);

        assert_eq!(
            bands.next(),
            Some(Band {
                rows: 12..=12,
                runs: vec![10..21],
                reds: vec![10, 20],
            })
        );
        assert_eq!(bands.next(), None);
    }

    #[test]
    fn vertical_line() {
        let mut bands = BandIter::from_edges([(12, 10)..=(22, 10), (22, 10)..=(12, 10)]);

        assert_eq!(
            bands.next(),
            Some(Band {
                rows: 12..=21,
                runs: vec![10..11],
                reds: vec![10],
            })
        );
        assert_eq!(
            bands.next(),
            Some(Band {
                rows: 22..=22,
                runs: vec![10..11],
                reds: vec![10],
            })
        );
        assert_eq!(bands.next(), None);
    }

    #[test]
    fn four_point_vertical_line() {
        let mut bands = BandIter::from_edges([
            (12, 10)..=(22, 10),
            (22, 10)..=(42, 10),
            (42, 10)..=(32, 10),
            (32, 10)..=(12, 10),
        ]);

        assert_eq!(
            bands.next(),
            Some(Band {
                rows: 12..=21,
                runs: vec![10..11],
                reds: vec![10],
            })
        );
        assert_eq!(
            bands.next(),
            Some(Band {
                rows: 22..=31,
                runs: vec![10..11],
                reds: vec![10],
            })
        );
        assert_eq!(
            bands.next(),
            Some(Band {
                rows: 32..=41,
                runs: vec![10..11],
                reds: vec![10],
            })
        );
        assert_eq!(
            bands.next(),
            Some(Band {
                rows: 42..=42,
                runs: vec![10..11],
                reds: vec![10],
            })
        );
        assert_eq!(bands.next(), None);
    }

    #[test]
    fn u_shape() {
        let mut bands = BandIter::from_edges([
            (10, 10)..=(10, 20),
            (10, 20)..=(20, 20),
            (20, 20)..=(20, 30),
            (20, 30)..=(10, 30),
            (10, 30)..=(10, 40),
            (10, 40)..=(30, 40),
            (30, 40)..=(30, 10),
            (30, 10)..=(10, 10),
        ]);

        assert_eq!(
            bands.next(),
            Some(Band {
                rows: 10..=19,
                runs: vec![10..21, 30..41],
                reds: vec![10, 20, 30, 40],
            })
        );
        assert_eq!(
            bands.next(),
            Some(Band {
                rows: 20..=29,
                runs: vec![10..41],
                reds: vec![20, 30],
            })
        );
        assert_eq!(
            bands.next(),
            Some(Band {
                rows: 30..=30,
                runs: vec![10..41],
                reds: vec![10, 40],
            })
        );
        assert_eq!(bands.next(), None);
    }

    #[test]
    fn j_shape() {
        let mut bands = BandIter::from_edges([
            (10, 10)..=(10, 20),
            (10, 20)..=(30, 20),
            (30, 20)..=(30, 30),
            (30, 30)..=(20, 30),
            (20, 30)..=(20, 40),
            (20, 40)..=(40, 40),
            (40, 40)..=(40, 10),
            (40, 10)..=(10, 10),
        ]);

        assert_eq!(
            bands.next(),
            Some(Band {
                rows: 10..=19,
                runs: vec![10..21],
                reds: vec![10, 20],
            })
        );
        assert_eq!(
            bands.next(),
            Some(Band {
                rows: 20..=29,
                runs: vec![10..21, 30..41],
                reds: vec![30, 40],
            })
        );
        assert_eq!(
            bands.next(),
            Some(Band {
                rows: 30..=39,
                runs: vec![10..41],
                reds: vec![20, 30],
            })
        );
        assert_eq!(
            bands.next(),
            Some(Band {
                rows: 40..=40,
                runs: vec![10..41],
                reds: vec![10, 40],
            })
        );
        assert_eq!(bands.next(), None);
    }

    #[test]
    fn lower_case_r_shape() {
        let mut bands = BandIter::from_edges([
            (10, 10) ..= (10, 40),
            (10, 40) ..= (30, 40),
            (30, 40) ..= (30, 30),
            (30, 30) ..= (20, 30),
            (20, 30) ..= (20, 20),
            (20, 20) ..= (40, 20),
            (40, 20) ..= (40, 10),
            (40, 10) ..= (10, 10),
        ]);

        assert_eq!(
            bands.next(),
            Some(Band {
                rows: 10..=19,
                runs: vec![10..41],
                reds: vec![10, 40],
            })
        );
        // The row with the two inner corners is still
        // a single run, the full width of the figure.
        assert_eq!(
            bands.next(),
            Some(Band {
                rows: 20..=20,
                runs: vec![10..41],
                reds: vec![20, 30],
            })
        );
        // The band with two disjoin runs begins below
        // the row with the inner corners.
        assert_eq!(
            bands.next(),
            Some(Band {
                rows: 21..=29,
                runs: vec![10..21, 30..41],
                reds: vec![],
            })
        );
        // The row the ends the r's right side also introduces a one-row band.
        assert_eq!(
            bands.next(),
            Some(Band {
                rows: 30..=30,
                runs: vec![10..21, 30..41],
                reds: vec![30, 40],
            })
        );
        assert_eq!(
            bands.next(),
            Some(Band {
                rows: 31..=39,
                runs: vec![10..21],
                reds: vec![],
            })
        );
        assert_eq!(
            bands.next(),
            Some(Band {
                rows: 40..=40,
                runs: vec![10..21],
                reds: vec![10, 20],
            })
        );
        assert_eq!(bands.next(), None);
    }

    #[test]
    #[ignore] // shared edges
    fn touching_c_shape() {
        let mut bands = BandIter::from_edges([
            (10, 10)..=(10, 40),
            (10, 40)..=(30, 40),
            (30, 40)..=(30, 30),
            (30, 30)..=(20, 30),
            (20, 30)..=(20, 20),
            (20, 20)..=(40, 20),
            (40, 20)..=(40, 30),
            (40, 30)..=(30, 30),
            (30, 30)..=(30, 40),
            (30, 40)..=(50, 40),
            (50, 40)..=(50, 10),
            (50, 10)..=(10, 10),
        ]);

        assert_eq!(
            bands.next(),
            Some(Band {
                rows: 10..=19,
                runs: vec![10..41],
                reds: vec![10, 40],
            })
        );
        assert_eq!(
            bands.next(),
            Some(Band {
                rows: 20..=20,
                runs: vec![10..41],
                reds: vec![20, 30],
            })
        );
        assert_eq!(
            bands.next(),
            Some(Band {
                rows: 21..=29,
                runs: vec![10..21, 30..41],
                reds: vec![],
            })
        );
        assert_eq!(
            bands.next(),
            Some(Band {
                rows: 30..=39,
                runs: vec![10..21, 30..41],
                reds: vec![30, 40],
            })
        );
        assert_eq!(
            bands.next(),
            Some(Band {
                rows: 40..=49,
                runs: vec![10..41],
                reds: vec![20, 30],
            })
        );
        assert_eq!(
            bands.next(),
            Some(Band {
                rows: 50..=50,
                runs: vec![10..41],
                reds: vec![10, 40],
            })
        );
        assert_eq!(bands.next(), None);
    }

    #[test]
    #[ignore] // shared edges
    fn touching_overbite_c_shape() {
        let mut bands = BandIter::from_edges([
            (10, 10)..=(10, 50),
            (10, 50)..=(30, 50),
            (30, 50)..=(30, 30),
            (30, 30)..=(20, 30),
            (20, 30)..=(20, 20),
            (20, 20)..=(40, 20),
            (40, 20)..=(40, 40),
            (40, 40)..=(30, 40),
            (30, 40)..=(30, 60),
            (30, 60)..=(50, 60),
            (50, 60)..=(50, 10),
            (50, 10)..=(10, 10),
        ]);

        assert_eq!(
            bands.next(),
            Some(Band {
                rows: 10..=19,
                runs: vec![10..51],
                reds: vec![10, 50],
            })
        );
        assert_eq!(
            bands.next(),
            Some(Band {
                rows: 20..=20,
                runs: vec![10..51],
                reds: vec![20, 30],
            })
        );
        assert_eq!(
            bands.next(),
            Some(Band {
                rows: 21..=29,
                runs: vec![10..21, 30..51],
                reds: vec![],
            })
        );
        assert_eq!(
            bands.next(),
            Some(Band {
                rows: 30..=30,
                runs: vec![10..21, 40..61],
                reds: vec![30, 40, 50, 60],
            })
        );
        assert_eq!(
            bands.next(),
            Some(Band {
                rows: 31..=39,
                runs: vec![10..21, 40..61],
                reds: vec![],
            })
        );
        assert_eq!(
            bands.next(),
            Some(Band {
                rows: 40..=49,
                runs: vec![10..61],
                reds: vec![20, 40],
            })
        );
        assert_eq!(
            bands.next(),
            Some(Band {
                rows: 50..=50,
                runs: vec![10..61],
                reds: vec![10, 60],
            })
        );
        assert_eq!(bands.next(), None);
    }

    #[test]
    #[ignore] // shared edges
    fn gladiator_mask() {
        // This shape has four edges that all come together at the same red
        // tile.
        let mut bands = BandIter::from_edges([
            (10, 10)..=(10, 50),
            (10, 50)..=(50, 50),
            (50, 50)..=(50, 30),
            (50, 30)..=(40, 30),
            (40, 30)..=(30, 30),
            (30, 30)..=(30, 40),
            (30, 40)..=(20, 40),
            (20, 40)..=(20, 20),
            (20, 20)..=(30, 20),
            (30, 20)..=(30, 30),
            (30, 30)..=(40, 30),
            (40, 30)..=(50, 30),
            (50, 30)..=(50, 10),
            (50, 10)..=(10, 10),
        ]);

        assert_eq!(
            bands.next(),
            Some(Band {
                rows: 10..=19,
                runs: vec![10..51],
                reds: vec![10, 50],
            })
        );
        assert_eq!(
            bands.next(),
            Some(Band {
                rows: 20..=20,
                runs: vec![10..51],
                reds: vec![20, 40],
            })
        );
        assert_eq!(
            bands.next(),
            Some(Band {
                rows: 21..=29,
                runs: vec![10..21, 40..51],
                reds: vec![],
            })
        );
        assert_eq!(
            bands.next(),
            Some(Band {
                rows: 30..=39,
                runs: vec![10..51],
                reds: vec![20, 40],
            })
        );
        assert_eq!(
            bands.next(),
            Some(Band {
                rows: 40..=49,
                runs: vec![10..51],
                reds: vec![30],
            })
        );
        assert_eq!(
            bands.next(),
            Some(Band {
                rows: 50..=50,
                runs: vec![10..51],
                reds: vec![10, 50],
            })
        );
        assert_eq!(bands.next(), None);
    }
}
