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
use std::cmp::{Ordering, max, min};
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
    pub fn from_edges(edges: impl IntoIterator<Item = Edge>) -> Self {
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
fn push_run(runs: &mut Vec<Range<u64>>, new_run: Range<u64>) {
    match runs.last_mut() {
        // If this is the first range, just add it to the vector.
        None => {
            runs.push(new_run);
        }

        // If there is a gap between this range and the previous one, just add
        // the new range to the end.
        Some(last) if last.end < new_run.start => {
            runs.push(new_run);
        }

        // Otherwise, extend the prior range to enclose this one.
        Some(last) => {
            last.end = max(last.end, new_run.end);
        }
    }
}

impl Iterator for BandIter {
    /// Non-overlapping runs of columns, sorted by column.
    type Item = Band;

    fn next(&mut self) -> Option<Band> {
        log::debug!("BandIter::next:");
        log::debug!("    next_row:  {:?}", self.next_row);
        log::debug!("    next:      {:?}", self.next);
        log::debug!("    unreached: {:?}", self.unreached);

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
        // left to right. Get the relevant edges from `self.next` and put them
        // in the easiest order for processing.
        let mut edges: Vec<Edge> = self.next.iter().cloned().collect();
        edges.sort_by(|a, b| {
            // Generally we want to visit edges from left to right.
            let by_left = edge_left(a).cmp(&edge_left(b));

            // If we have verticals joined by a horizontal, ensure the verticals
            // appear before and after the horizontal.
            let by_right = edge_right(a).cmp(&edge_right(b));

            // If we have two verticals joined together (in which case the above
            // two comparisons produce `Equal`), they should be connected; visit
            // the incoming before the outgoing.
            let by_arrival = if a.end() == b.start() {
                Ordering::Less
            } else if a.start() == b.end() {
                Ordering::Greater
            } else {
                Ordering::Equal
            };

            by_left.then(by_right).then(by_arrival)
        });
        log::debug!("sorted edges: {edges:?}");
        let mut runs: Vec<Range<u64>> = vec![];

        #[derive(Debug)]
        enum State {
            /// We are outside the shape.
            Outside,

            /// We entered the shape at the vertical edge `entry`.
            Inside {
                entry: Edge,

                /// If Some, this is the most recent edge that starts or ends on
                /// `band_top`, but to which we haven't seen the connecting
                /// edge. There should be only one such.
                dangling: Option<Edge>,
            },
        }

        // Whether we are inside the shape, and if so, how we
        // are connected to the boundary.
        let mut state = State::Outside;

        // Whether there are any horizontal edges on the `band_top` row where
        // the area below the edge is outside the shape.
        //
        // When we draw a bottom edge like that, the edge itself is
        // included inside the shape, but the next row of floor tiles
        // is outside, so the band has to end immediately after the
        // row containing the edge. And since red tiles appear only at
        // the top of a band, this means the band will contain a
        // single row.
        let mut includes_bottom_edge = false;

        // Process the edges.
        for edge in edges {
            log::debug!("    considering edge {edge:?}");
            if is_vertical(&edge) {
                match state {
                    State::Outside => {
                        let dangling = if edge.start().0 == band_top || edge.end().0 == band_top {
                            Some(edge.clone())
                        } else {
                            None
                        };
                        state = State::Inside {
                            entry: edge,
                            dangling,
                        };
                    }
                    
                    
                    State::Inside {
                        entry,
                        dangling: None,
                    }
                    => {
                        // Is this edge just a continuation of the edge we entered at?
                        if are_connected(&edge, &entry) {
                            assert!(is_vertical(&edge) == is_vertical(&entry));
                            state = State::Inside {
                                entry,
                                dangling: None,
                            }; // no change
                        } else if edge.start().0 == band_top || edge.end().0 == band_top {
                            // We've found a dangling edge. We'll need to see more
                            // to tell whether we're exiting the shape.
                            state = State::Inside {
                                entry,
                                dangling: Some(edge),
                            };
                        } else {
                            push_run(&mut runs, entry.start().1..edge.start().1 + 1);
                            state = State::Outside;
                        }
                    }
                    
                    
                    State::Inside {
                        entry,
                        dangling: Some(dangling),
                    }
                    => {
                        assert!(are_connected(&edge, &dangling));
                        if goes_down(&edge) == goes_down(&entry) {
                            // `edge` just continues the boundary in the same
                            // direction that `entry` established, but we no longer
                            // have a horizontal we're extending.
                            state = State::Inside {
                                entry,
                                dangling: None,
                            };
                        } else {
                            // We have exited the shape.
                            push_run(&mut runs, entry.start().1..edge.start().1 + 1);
                            state = State::Outside;
                        }
                    }
                }
            } else {
                match state {
                     State::Outside => {
                        panic!("bare horizontal edge outside: {edge:?}");
                    }
                    
                        State::Inside {
                            entry,
                            dangling: None,
                        }
                     => {
                        panic!("bare horizontal edge inside {entry:?}: {edge:?}");
                    }
                    
                        State::Inside {
                            entry,
                            dangling: Some(dangling),
                        }
                     => {
                        // This horizontal edge must extend the dangling boundary edge.
                        assert!(are_connected(&edge, &dangling));

                        // Determine whether `edge` is a bottom edge.
                        if dangling == entry {
                            // The dangling edge this horizontal edge connects to is the one
                            // by which we entered the shape. Since this edge is connected to it,
                            // that entry edge either starts or ends on this row.
                            if edge_bottom(&dangling) == band_top {
                                log::debug!("    is a bottom edge");
                                includes_bottom_edge = true;
                            }
                        } else if is_vertical(&dangling) {
                            if edge_top(&dangling) == band_top {
                                log::debug!("    is a bottom edge");
                                includes_bottom_edge = true;
                            }
                        }
                        state = State::Inside {
                            entry,
                            dangling: Some(edge),
                        };
                    }
                }
            }

            log::debug!("    -> {state:?}");
        }
        assert!(matches!(state, State::Outside));

        // Drop all edges from `next` that end at the top of this band.
        while let Some(maybe_done) = self.next.peek_mut() {
            let maybe_done_bottom = edge_bottom(&*maybe_done);
            assert!(band_top <= maybe_done_bottom);
            if band_top < maybe_done_bottom {
                break;
            }
            PeekMut::pop(maybe_done);
        }

        let band_bottom = if includes_bottom_edge {
            // If there was a bottom edge at `band_top`, then we're a one-row band.
            band_top
        } else {
            // Otherwise, the band extends up to (but not including) the next row
            // with red tiles.
            match (self.next.peek(), self.unreached.peek()) {
                (None, None) => band_top,
                (Some(next), None) => edge_bottom(next) - 1,
                (None, Some(unreached)) => edge_top(unreached) - 1,
                (Some(next), Some(unreached)) => min(edge_bottom(next), edge_top(unreached)) - 1,
            }
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
    fn square_simple() {
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
    #[ignore] // shared edges
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
    #[ignore] // shared edges
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
            (10, 10)..=(10, 40),
            (10, 40)..=(30, 40),
            (30, 40)..=(30, 30),
            (30, 30)..=(20, 30),
            (20, 30)..=(20, 20),
            (20, 20)..=(40, 20),
            (40, 20)..=(40, 10),
            (40, 10)..=(10, 10),
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

    /// A square with three red tiles along each edge: the corners and the
    /// midpoint.
    #[test]
    fn square_with_midpoints_clockwise() {
        let mut bands = BandIter::from_edges([
            (10, 10)..=(10, 20),
            (10, 20)..=(10, 30),
            (10, 30)..=(20, 30),
            (20, 30)..=(30, 30),
            (30, 30)..=(30, 20),
            (30, 20)..=(30, 10),
            (30, 10)..=(20, 10),
            (20, 10)..=(10, 10),
        ]);

        assert_eq!(
            bands.next(),
            Some(Band {
                rows: 10..=19,
                runs: vec![10..31],
                reds: vec![10, 20, 30],
            })
        );
        assert_eq!(
            bands.next(),
            Some(Band {
                rows: 20..=29,
                runs: vec![10..31],
                reds: vec![10, 30],
            })
        );
        assert_eq!(
            bands.next(),
            Some(Band {
                rows: 30..=30,
                runs: vec![10..31],
                reds: vec![10, 20, 30],
            })
        );
        assert_eq!(bands.next(), None);
    }

    #[test]
    fn square_with_midpoints_counterclockwise() {
        let mut bands = BandIter::from_edges([
            (10, 10)..=(20, 10),
            (20, 10)..=(30, 10),
            (30, 10)..=(30, 20),
            (30, 20)..=(30, 30),
            (30, 30)..=(20, 30),
            (20, 30)..=(10, 30),
            (10, 30)..=(10, 20),
            (10, 20)..=(10, 10),
        ]);

        assert_eq!(
            bands.next(),
            Some(Band {
                rows: 10..=19,
                runs: vec![10..31],
                reds: vec![10, 20, 30],
            })
        );
        assert_eq!(
            bands.next(),
            Some(Band {
                rows: 20..=29,
                runs: vec![10..31],
                reds: vec![10, 30],
            })
        );
        assert_eq!(
            bands.next(),
            Some(Band {
                rows: 30..=30,
                runs: vec![10..31],
                reds: vec![10, 20, 30],
            })
        );
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
