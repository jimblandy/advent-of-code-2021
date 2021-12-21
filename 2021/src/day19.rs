#![allow(unused_imports, dead_code)]
use aoc_runner_derive::{aoc, aoc_generator};
use anyhow::{anyhow, bail, Result};
use crate::cartesian_product;
use itertools::Itertools;
use std::{cmp, ops};
use std::collections::BinaryHeap;
use hashbrown::{HashSet, HashMap};
use std::fmt;

#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
struct Point(i32, i32, i32);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Matrix(Point, Point, Point);

impl Point {
    #[inline]
    const fn addp(this: Point, rhs: Point) -> Point {
        Point(this.0 + rhs.0, this.1 + rhs.1, this.2 + rhs.2)
    }

    #[inline]
    const fn muls(this: Point, rhs: i32) -> Point {
        Point(this.0 * rhs, this.1 * rhs, this.2 * rhs)
    }

    #[inline]
    const fn neg(this: Point) -> Point {
        Point::muls(this, -1)
    }

    #[inline]
    fn lexcmp(this: Point, rhs: Point) -> cmp::Ordering {
        this.0.cmp(&rhs.0)
            .then(this.1.cmp(&rhs.1))
            .then(this.2.cmp(&rhs.2))
    }

    fn manhattan(lhs: Point, rhs: Point) -> i32 {
        let diff = lhs - rhs;
        diff.0.abs() + diff.1.abs() + diff.2.abs()
    }
}

impl Matrix {
    #[inline]
    const fn mulp(this: Matrix, rhs: Point) -> Point {
        Point::addp(Point::addp(Point::muls(this.0, rhs.0),
                                Point::muls(this.1, rhs.1)),
                    Point::muls(this.2, rhs.2))
    }

    #[inline]
    const fn mulm(this: Matrix, rhs: Matrix) -> Matrix {
        Matrix(Matrix::mulp(this, rhs.0),
               Matrix::mulp(this, rhs.1),
               Matrix::mulp(this, rhs.2))
    }

    #[inline]
    const fn ipow(this: Matrix, pow: usize) -> Matrix{
        match pow % 4 {
            0 => IDENT,
            1 => this,
            2 => Matrix::mulm(this, this),
            3 => Matrix::mulm(Matrix::mulm(this, this), this),
            _ => ZEROM,
        }
    }
}

impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{},{},{}", self.0, self.1, self.2)
    }
}

impl ops::Add for Point {
    type Output = Point;

    fn add(self, rhs: Self) -> Self::Output {
        Point::addp(self, rhs)
    }
}

impl ops::Sub for Point {
    type Output = Point;

    fn sub(self, rhs: Self) -> Self::Output {
        Point(self.0 - rhs.0, self.1 - rhs.1, self.2 - rhs.2)
    }
}

impl ops::Neg for Point {
    type Output = Point;

    fn neg(self) -> Self::Output {
        Point::neg(self)
    }
}

impl ops::Mul<i32> for Point {
    type Output = Point;

    fn mul(self, rhs: i32) -> Self::Output {
        Point::muls(self, rhs)
    }
}

impl ops::Div<i32> for Point {
    type Output = Point;

    fn div(self, rhs: i32) -> Self::Output {
        Point(self.0 / rhs, self.1 / rhs, self.2 / rhs)
    }
}

impl ops::Mul<Point> for Matrix {
    type Output = Point;

    fn mul(self, rhs: Point) -> Self::Output {
        Matrix::mulp(self, rhs)
    }
}

impl ops::Mul<Matrix> for Matrix {
    type Output = Matrix;

    fn mul(self, rhs: Matrix) -> Self::Output {
        Matrix::mulm(self, rhs)
    }
}

const ZEROP: Point = Point(0, 0, 0);
const XHAT: Point = Point(1, 0, 0);
const YHAT: Point = Point(0, 1, 0);
const ZHAT: Point = Point(0, 0, 1);

const IDENT: Matrix = Matrix(XHAT, YHAT, ZHAT);
const ZEROM: Matrix = Matrix(ZEROP, ZEROP, ZEROP);
// "clockwise looking down the positive _ axis at the origin"
const CWX: Matrix = Matrix(XHAT, Point::neg(ZHAT), YHAT);
const CWY: Matrix = Matrix(ZHAT, YHAT, Point::neg(XHAT));
const CWZ: Matrix = Matrix(Point::neg(YHAT), XHAT, ZHAT);
static ORIENTATIONS: [Matrix; 24] = [
    // pointing along positive x
    Matrix::mulm(Matrix::ipow(CWX, 0), Matrix::ipow(CWZ, 0)),
    Matrix::mulm(Matrix::ipow(CWX, 1), Matrix::ipow(CWZ, 0)),
    Matrix::mulm(Matrix::ipow(CWX, 2), Matrix::ipow(CWZ, 0)),
    Matrix::mulm(Matrix::ipow(CWX, 3), Matrix::ipow(CWZ, 0)),

    // pointing along positive y
    Matrix::mulm(Matrix::ipow(CWY, 0), Matrix::ipow(CWZ, 1)),
    Matrix::mulm(Matrix::ipow(CWY, 1), Matrix::ipow(CWZ, 1)),
    Matrix::mulm(Matrix::ipow(CWY, 2), Matrix::ipow(CWZ, 1)),
    Matrix::mulm(Matrix::ipow(CWY, 3), Matrix::ipow(CWZ, 1)),

    // pointing along negative x
    Matrix::mulm(Matrix::ipow(CWX, 0), Matrix::ipow(CWZ, 2)),
    Matrix::mulm(Matrix::ipow(CWX, 1), Matrix::ipow(CWZ, 2)),
    Matrix::mulm(Matrix::ipow(CWX, 2), Matrix::ipow(CWZ, 2)),
    Matrix::mulm(Matrix::ipow(CWX, 3), Matrix::ipow(CWZ, 2)),

    // pointing along negative y
    Matrix::mulm(Matrix::ipow(CWY, 0), Matrix::ipow(CWZ, 3)),
    Matrix::mulm(Matrix::ipow(CWY, 1), Matrix::ipow(CWZ, 3)),
    Matrix::mulm(Matrix::ipow(CWY, 2), Matrix::ipow(CWZ, 3)),
    Matrix::mulm(Matrix::ipow(CWY, 3), Matrix::ipow(CWZ, 3)),

    // pointing along positive z
    Matrix::mulm(Matrix::ipow(CWZ, 0), Matrix::ipow(CWY, 1)),
    Matrix::mulm(Matrix::ipow(CWZ, 1), Matrix::ipow(CWY, 1)),
    Matrix::mulm(Matrix::ipow(CWZ, 2), Matrix::ipow(CWY, 1)),
    Matrix::mulm(Matrix::ipow(CWZ, 3), Matrix::ipow(CWY, 1)),

    // pointing along negative z
    Matrix::mulm(Matrix::ipow(CWZ, 0), Matrix::ipow(CWY, 3)),
    Matrix::mulm(Matrix::ipow(CWZ, 1), Matrix::ipow(CWY, 3)),
    Matrix::mulm(Matrix::ipow(CWZ, 2), Matrix::ipow(CWY, 3)),
    Matrix::mulm(Matrix::ipow(CWZ, 3), Matrix::ipow(CWY, 3)),
];

#[test]
fn test_matrix() {
    assert_eq!(CWX * XHAT, XHAT);
    assert_eq!(CWX * YHAT, -ZHAT);
    assert_eq!(CWX * ZHAT, YHAT);

    assert_eq!(CWX * (CWY * Point(1, 10, 100)), (CWX * CWY) * Point(1, 10, 100));
    assert_eq!(CWZ * CWZ * CWY * CWZ * CWY * CWX, IDENT);
    assert_eq!(CWZ * CWZ * CWY * CWZ * CWY * CWX * Point(1, 10, 100), Point(1, 10, 100));
}

#[test]
fn test_orientation_table() {
    let mut seen = HashSet::new();
    for &orientation in &ORIENTATIONS {
        assert!(seen.insert(orientation * Point(1, 10, 100)))
    }
    assert_eq!(seen.len(), 24);
}

#[derive(Debug)]
struct Problem {
    scanners: Vec<Scanner>,
}

impl fmt::Display for Problem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, s) in self.scanners.iter().enumerate() {
            writeln!(f, "--- scanner {} ---", i)?;
            for beacon in &s.beacons {
                writeln!(f, "{}", beacon)?;
            }
            if i < self.scanners.len() - 1 {
                writeln!(f)?;
            }
        }

        Ok(())
    }
}

#[derive(Debug)]
struct Scanner {
    beacons: Vec<Point>,
    distances: HashSet<Point>,
}

#[aoc_generator(day19)]
fn generator(input: &str) -> Result<Problem> {
    input.split("--- scanner ")
        .skip(1)
        .map(|scanner| {
            scanner.lines()
                .skip(1)
                .filter(|line| !line.trim().is_empty())
                .map(|beacon| {
                    let coords = beacon.split(',')
                        .map(|coord| i32::from_str_radix(coord, 10))
                        .collect::<std::result::Result<Vec<i32>, _>>()?;
                    if coords.len() != 3 {
                        bail!("bad beacon line: {:?}", beacon);
                    }

                    Ok(Point(coords[0], coords[1], coords[2]))
                })
                .collect::<Result<Vec<Point>>>()
                .map(|beacons| {
                    let distances = cartesian_product(&beacons, &beacons)
                        .map(|(&a, &b)| a - b)
                        .filter(|&d| d != ZEROP)
                        .collect();
                    Scanner { beacons, distances }
                })
        })
        .collect::<Result<Vec<Scanner>>>()
        .map(|scanners| Problem { scanners })
}

#[cfg(test)]
fn sample() -> Problem {
    generator(include_str!("sample/day19")).unwrap()
}

#[cfg(test)]
fn sample_tiny() -> Problem {
    generator(include_str!("sample/day19.tiny")).unwrap()
}

#[test]
fn test_generator() {
    let s = sample();
    assert_eq!(s.scanners.len(), 5);
    assert_eq!(s.scanners[0].beacons.len(), 25);
    assert_eq!(s.scanners[1].beacons.len(), 25);

    let t = sample_tiny();
    assert_eq!(t.scanners.len(), 5);
    for i in 0..t.scanners.len() {
        assert_eq!(t.scanners[i].beacons.len(), 6);
    }
}

impl Scanner {
    /// Find the orientation for `self` that best matches `other`, and
    /// the number of matches that produces.
    ///
    /// That is, find the transformation to apply to `self`'s distances
    /// that produces the most matches with `other`'s distances.
    fn best_orientation_for(&self, other: &Scanner) -> (Matrix, usize) {
        ORIENTATIONS
            .iter()
            .map(|&o| {
                let matches = self.distances
                    .iter()
                    .filter(|&&d| other.distances.contains(&(o * d)))
                    .count();
                (o, matches)
            })
            .max_by_key(|&(_, n)| n)
            .unwrap_or((ZEROM, 0))
    }
}

/// Element `[i][j]` is the orientation to apply to `scanner[i]` to best orient
/// it to match `scanner[j]`, paired with the number of distance matches that
/// orientation produces.
type Matches = Vec<Vec<(Matrix, usize)>>;

/// For a problem, return a vector mapping each scanner
/// to its best orientations with each other scanner.
fn pairwise_matches(problem: &Problem) -> Matches {
    let scanners = &problem.scanners;
    let n = scanners.len();
    (0..n)
        .map(|i| {
            (0..n)
                .map(|j| {
                    if i == j {
                        let n = scanners[i].beacons.len();
                        (IDENT, n * (n - 1))
                    } else {
                        scanners[i].best_orientation_for(&scanners[j])
                    }
                })
                .collect()
        })
        .collect()
}

#[test]
fn test_pairwise_matches() {
    let t = sample_tiny();
    let p = pairwise_matches(&t);

    let strengths: Vec<Vec<usize>> = p
        .iter()
        .map(|s| s.iter().map(|&(_o, m)| m).collect())
        .collect();

    assert_eq!(strengths,
               vec![
                   // There are two beacons with the same distance relative to
                   // each other.
                   vec![30, 28, 28, 28, 28],
                   vec![28, 30, 28, 28, 28],
                   vec![28, 28, 30, 28, 28],
                   vec![28, 28, 28, 30, 28],
                   vec![28, 28, 28, 28, 30],
               ]);

    let s = sample();
    let p = pairwise_matches(&s);

    let strengths: Vec<Vec<usize>> = p
        .iter()
        .map(|s| s.iter().map(|&(_o, m)| m).collect())
        .collect();

    assert_eq!(strengths,
               vec![
                   vec![600, 132,   6,   0,  30],
                   vec![132, 600,  30, 132, 132],
                   vec![  6,  30, 650,   6, 132],
                   vec![  0, 132,   6, 600,  30],
                   vec![ 30, 132, 132,  30, 650]
               ]);
}

type Tree = Vec<(usize, usize)>;

#[derive(Debug)]
struct Edge {
    from: usize,
    to: usize,
    steps: usize,
    strength: usize,
}

impl cmp::PartialEq for Edge {
    fn eq(&self, other: &Self) -> bool {
        self.steps == other.steps &&
            self.strength == other.strength
    }
}

impl cmp::Eq for Edge {}

impl cmp::PartialOrd for Edge {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl cmp::Ord for Edge {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.strength.cmp(&other.strength)
    }
}

/// Map the tree of strongest connections between scanners.
///
/// Return a vector of pairs `(from, to)` indicating a tree edge from scanner
/// `from` to scanner `to`. The pairs are ordered such that the first edge
/// starts from scanner `0`, and otherwise each scanner appears only as `from`
/// after it has appeared as `to`.
fn spanning_tree(matches: &Matches) -> Tree {
    let mut edges = vec![];
    let mut reached = vec![false; matches.len()];
    reached[0] = true;

    // Treat the matrix as a description of a complete graph between scanners,
    // whose edges are weighted by the firmness of the connection between the
    // two scanners. Find the firmest connection from scanner 0 to every other
    // scanner, composing orientations as we go.
    let mut pending = BinaryHeap::new();
    for (to, &(_o, m)) in matches[0].iter().enumerate().skip(1) {
        pending.push(Edge { from: 0, to, steps: 0, strength: m });
    }

    while let Some(edge) = pending.pop() {
        let here = edge.to;
        if !reached[here] {
            reached[here] = true;
            edges.push((edge.from, here));
            // Have we reached everybody?
            if edges.len() >= matches.len() - 1 {
                break;
            }

            for (next, &(_o, m)) in matches[here].iter().enumerate() {
                if !reached[next] {
                    pending.push(Edge { from: here, to: next, steps: edge.steps + 1, strength: m })
                }
            }
        }
    }

    edges
}

#[test]
fn test_spanning_tree() {
    let s = sample();
    let p = pairwise_matches(&s);
    assert_eq!(spanning_tree(&p), vec![(0, 1), (1, 3), (1, 4), (4, 2)]);
}

/// Return the position and orientation of each scanner relative to scanner 0.
///
/// In other words, if the `i`'th element is `(m, p)`, then `m b + p` transforms
/// a beacon position `b` from `i`'s coordinate space to scanner 0's coordinate
/// space.
///
/// The zero'th element is `(IDENT, ZEROP)`.
fn positions(problem: &Problem, matches: &Matches, tree: &Tree) -> Vec<(Matrix, Point)> {
    let scanners = &problem.scanners;
    let mut positions = vec![(IDENT, ZEROP); scanners.len()];

    for &(from, to) in tree {
        let step = matches[to][from].0; // back one step
        let rest = positions[from].0; // the rest the way to zero

        // Consider every pair of a beacon seen by `from` with a beacon seen by
        // `to`. Applying the orientation to `to`'s beacons, compute the offset
        // from `from`'s beacon to `to`'s beacon. The most common offset should
        // be the offset from the scanner `from` to the scanner `to`.
        let mut distances: HashMap<Point, usize> = HashMap::new();
        for &to_beacon in &scanners[to].beacons {
            for &from_beacon in &scanners[from].beacons {
                let delta = from_beacon - step * to_beacon;
                *distances.entry(delta).or_insert(0) += 1;
            }
        }
        // Offset, in from's orientation
        let offset = *distances.iter().max_by_key(|(_, &count)| count).unwrap().0;

        let full_offset = positions[from].1 + rest * offset;
        positions[to] = (rest * step, full_offset);
    }

    positions
}

#[test]
fn test_positions() {
    let s = sample();
    let m = pairwise_matches(&s);
    let t = spanning_tree(&m);
    let p = positions(&s, &m, &t);

    assert_eq!(p[0].1, Point(0, 0, 0));
    assert_eq!(p[1].1, Point(68, -1246, -43));
    assert_eq!(p[2].1, Point(1105, -1205, 1229));
    assert_eq!(p[3].1, Point(-92, -2380, -20));
    assert_eq!(p[4].1, Point(-20, -1133, 1061));
}

/// Return a HashSet of beacon's positions relative to scanner 0.
fn beacon_positions(problem: &Problem) -> HashSet<Point> {
    let m = pairwise_matches(&problem);
    let t = spanning_tree(&m);
    let p = positions(&problem, &m, &t);

    problem.scanners
        .iter()
        .enumerate()
        .flat_map(|(i, scanner)| {
            let (m, p) = p[i];
            scanner.beacons
                .iter()
                .map(move |&beacon| m * beacon + p)
        })
        .collect()
}

#[aoc(day19, part1)]
fn part1(problem: &Problem) -> usize {
    beacon_positions(problem).len()
}

#[test]
fn test_part1() {
    assert_eq!(part1(&sample()), 79);
}

#[aoc(day19, part2)]
fn part2(problem: &Problem) -> i32 {
    let m = pairwise_matches(&problem);
    let t = spanning_tree(&m);
    let p = positions(&problem, &m, &t);

    cartesian_product(&p, &p)
        .map(|(&(_, a), &(_, b))| Point::manhattan(a, b))
        .max()
        .unwrap()
}

#[test]
fn test_part2() {
    assert_eq!(part2(&sample()), 3621);
}
