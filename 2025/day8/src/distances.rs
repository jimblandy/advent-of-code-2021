use super::Point;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Edge {
    from: usize,
    to: usize,
}

impl Edge {
    pub fn new(a: usize, b: usize) -> Edge {
        if a < b {
            Edge { from: a, to: b }
        } else {
            Edge { from: b, to: a }
        }
    }

    pub fn to(&self) -> usize {
        self.to
    }
    pub fn from(&self) -> usize {
        self.from
    }
}

pub struct Distances {
    len: usize,

    /// Triangular matrix of squared distances, indexed by `Edge`.
    distances: Vec<u64>,
}

impl std::ops::Index<Edge> for Distances {
    type Output = u64;

    fn index(&self, edge: Edge) -> &u64 {
        let flat_index = self.flat_index(edge);
        &self.distances[flat_index]
    }
}

impl Distances {
    pub fn from_points(points: &[Point]) -> Self {
        let mut distances = Vec::with_capacity(triangle(points.len()));

        fn sq_diff(a: u64, b: u64) -> u64 {
            let diff = a.abs_diff(b);
            diff * diff
        }

        for (to_index, to_point) in points.iter().enumerate() {
            for (from_index, from_point) in points.iter().enumerate().take(to_index) {
                distances.push(
                    sq_diff(to_point.0, from_point.0)
                        + sq_diff(to_point.1, from_point.1)
                        + sq_diff(to_point.2, from_point.2),
                );
            }
        }

        Distances {
            len: points.len(),
            distances,
        }
    }

    fn flat_index(&self, edge: Edge) -> usize {
        assert!(0 < edge.to); // should be true by construction
        assert!(edge.to < self.len);
        let row_base = triangle(edge.to - 1);
        row_base + edge.from
    }

    pub fn edges_by_length(&self) -> Vec<Edge> {
        let mut edges: Vec<Edge> = (0..self.len)
            .flat_map(|to| (0..to).map(move |from| Edge::new(from, to)))
            .collect();

        edges.sort_by(|&a, &b| -> std::cmp::Ordering { self[a].cmp(&self[b]) });

        edges
    }
}

#[test]
fn test_distances() {
    let d = Distances::from_points(&[(0, 1, 2), (5, 4, 3), (6, 8, 7)]);
    assert_eq!(d.distances.len(), 3);
    assert_eq!(d[Edge::new(0, 1)], 35);
    assert_eq!(d[Edge::new(0, 2)], 36 + 49 + 25);
    assert_eq!(d[Edge::new(1, 2)], 1 + 16 + 16);

    assert_eq!(d[Edge::new(1, 0)], 35);
    assert_eq!(d[Edge::new(2, 0)], 36 + 49 + 25);
    assert_eq!(d[Edge::new(2, 1)], 1 + 16 + 16);

    assert_eq!(
        d.edges_by_length(),
        vec![Edge::new(1, 2), Edge::new(0, 1), Edge::new(0, 2),]
    );
}

/// Return the `n''th triangle number.
///
/// This is the number of elements occupied in the distances
/// half-matrix by rows `0` through `n`, inclusive.
fn triangle(n: usize) -> usize {
    if n & 1 == 0 {
        (n / 2) * (n + 1)
    } else {
        n * ((n + 1) / 2)
    }
}

#[test]
fn test_triangle() {
    assert_eq!(triangle(0), 0);
    assert_eq!(triangle(1), 1);
    assert_eq!(triangle(2), 3);
    assert_eq!(triangle(3), 6);
    assert_eq!(triangle(4), 10);
    assert_eq!(triangle(100), 5050);
}
