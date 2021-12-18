#![allow(unused_imports, dead_code)]

use aoc_runner_derive::{aoc, aoc_generator};
use anyhow::{anyhow, bail, Result};
use std::fmt;
use std::ops::Range;
use std::str::FromStr;

type Value = u8;

/// A snailfish number represented as a tree flattened into an array.
///
/// A three-level-deep tree, with pairs nested at most two deep, would have
/// nodes placed like this in the array:
///
///     0   1   2   3   4   5   6   7
///
///             +------root-----+
///         +---*---+       +---*---+
///         *       *       *       *
///
/// If a node is a constant, then its entire subtree is left empty.
///
/// In general, a tree with pairs nested at most N deep needs an N + 1 level tree,
/// which requires 2^(N + 1) elements.
///
/// This lets us store no pointers, hold all data in a compact range of memory,
/// use array scans to find nearest neighbors in the tree, and use bit twiddling
/// to find children and parents.
struct Basis<const N: usize> {
    elts: [Elt; N]
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Elt {
    Empty,
    Pair,
    Value(Value),
}

impl<const N: usize> fmt::Debug for Basis<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for i in 0..N {
            match self.elts[i] {
                Elt::Empty => write!(f, ". ")?,
                Elt::Pair => write!(f, "* ")?,
                Elt::Value(v) => write!(f, "{:<2}", v)?,
            }
        }
        writeln!(f)?;
        write!(f, "  ")?;
        for i in 1..N {
            let ch = match (i - 1) & !i {
                0 => ' ',
                1 => '_',
                3 => '.',
                7 => '-',
                15 => '^',
                31 => '*',
                _ => panic!("unexpected mask"),
            };
            write!(f, "{} ", ch)?;
        }
        writeln!(f)?;

        Ok(())
    }
}

/// AoC trees are temporarily 5 deep, so we need 2^(5+1) = 64 elements.
type AocBasis = Basis<64>;

type Node = usize;

/// N should be a power of two.
impl<const N: usize> Basis<N> {
    const ROOT: usize = (N + 1) / 2;

    fn new() -> Self {
        Basis { elts: [Elt::Empty; N] }
    }

    fn clear(&mut self) {
        self.elts.fill(Elt::Empty);
    }

    fn is_pair(&self, n: Node) -> bool { self.elts[n] == Elt::Pair }

    fn children(&self, n: Node) -> (Node, Node) {
        assert!(self.is_pair(n));
        // the bit below the least significant 1-bit.
        let d = (n & !(n - 1)) / 2;
        (n - d, n + d)
    }

    /// The extent of the subtree rooted at the node n.
    fn subtree(&self, n: Node) -> Range<Node> {
        // all bits below the least significant 1-bit.
        let d = (n - 1) & !n;
        n - d .. n + d
    }

    // This doesn't preserve tree invariants; the caller
    // must promise to set both children as well.
    fn set_pair(&mut self, n: Node) -> (Node, Node) {
        assert!(n & 1 == 0, "too low to be a pair");
        self.elts[n] = Elt::Pair;
        self.children(n)
    }

    fn set_const(&mut self, n: Node, v: Value) {
        let t = self.subtree(n);
        self.elts[t].fill(Elt::Empty);
        self.elts[n] = Elt::Value(v);
    }

    fn get(&self, n: Node) -> Value {
        match self.elts[n] {
            Elt::Value(v) => v,
            _ => panic!("element at {} is not a value", n),
        }
    }

    fn get_mut(&mut self, n: Node) -> &mut Value {
        match self.elts[n] {
            Elt::Value(ref mut v) => v,
            _ => panic!("element at {} is not a value", n),
        }
    }

    /// Find the next node to explode, if any. Assume this is the deepest possible level.
    fn find_too_deep(&self) -> Option<Node> {
        // The deepest possible pairs appear at multiples of four remainder 2.
        (0 .. N / 4).map(|n| 4 * n + 2).find(|&n| self.elts[n] == Elt::Pair)
    }

    /// Find the first node that needs to split.
    fn find_needs_split(&self) -> Option<Node> {
        self.elts.iter().position(|&e| {
            match e {
                Elt::Value(v) if v >= 10 => true,
                _ => false,
            }
        })
    }

    /// Find the next number to the left of the pair at `n`, if any.
    fn find_left(&self, n: Node) -> Option<Node> {
        let t = self.subtree(n);
        (1 .. t.start).rev().find(|&n| matches!(self.elts[n], Elt::Value(_)))
    }

    /// Find the next number to the right of the pair at `n`, if any.
    fn find_right(&self, n: Node) -> Option<Node> {
        let t = self.subtree(n);
        (t.end + 1 .. N).find(|&n| matches!(self.elts[n], Elt::Value(_)))
    }

    fn change_bottom_pair_to_value(&mut self, pair: Node, v: Value) {
        assert!(pair & 0b11 == 0b10); // bottom-level pairs only
        let (left, right) = self.children(pair);
        self.elts[left] = Elt::Empty;
        self.elts[right] = Elt::Empty;
        self.elts[pair] = Elt::Value(v);
    }
}

impl<const N: usize> fmt::Display for Basis<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let r = NodeRef {
            basis: self,
            node: Self::ROOT,
        };

        r.fmt(f)
    }
}

/// A `Basis` and `Node` wrapped up together for formatting.
struct NodeRef<'a, const N: usize> {
    basis: &'a Basis<N>,
    node: Node,
}

impl<'a, const N: usize> NodeRef<'a, N> {
    fn is_pair(&self) -> bool {
        self.basis.is_pair(self.node)
    }

    fn children(&self) -> (Self, Self) {
        let (left, right) = self.basis.children(self.node);
        (self.at(left), self.at(right))
    }

    fn get(&self) -> Value {
        self.basis.get(self.node)
    }

    fn at(&self, node: Node) -> Self {
        NodeRef {
            basis: self.basis,
            node
        }
    }
}

impl<'a, const N: usize> fmt::Display for NodeRef<'a, N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_pair() {
            let (left, right) = self.children();
            write!(f, "[{},{}]", left, right)
        } else {
            write!(f, "{}", self.get())
        }
    }
}

/// Parse `input` as a snailfish number, and store it at `pos` in `basis`.
fn parse<const N: usize>(input: &str, basis: &mut Basis<N>) -> Result<()> {
    fn recur<'i, 'b, const N: usize>(input: &'i str, basis: &'b mut Basis<N>, pos: Node) -> Result<&'i str> {
        if input.starts_with("[") {
            let (left, right) = basis.set_pair(pos);
            let rest = recur(&input[1..], basis, left)?;
            let rest = match rest.split_once(",") {
                Some(("", rest)) => rest,
                _ => bail!("missing , in pair: {:?}, rest = {:?}", input, rest),
            };
            let rest = recur(rest, basis, right)?;
            match rest.split_once("]") {
                Some(("", rest)) => Ok(rest),
                _ => bail!("expected closing bracket: {:?}", rest),
            }
        } else {
            let (value, rest) = match input.find(|ch: char| !ch.is_digit(10)) {
                Some(0) => bail!("confusing start to input: {:?}", input),
                Some(end) => (Value::from_str(&input[..end])?, &input[end..]),
                None => (Value::from_str(input)?, "")
            };

            basis.set_const(pos, value);
            Ok(rest)
        }
    }

    basis.clear();
    let rest = recur(input.trim(), basis, Basis::<N>::ROOT)?;
    if !rest.is_empty() {
        bail!("Garbage at end of input: {:?}", rest);
    }

    Ok(())
}

#[test]
fn test_parse() {
    use Elt::*;

    let mut basis = Basis::<4>::new();
    assert!(parse("[1,2]", &mut basis).is_ok());
    assert_eq!(basis.elts, [Empty, Value(1), Pair, Value(2)]);
    assert_eq!(basis.to_string(), "[1,2]");

    let mut basis = Basis::<8>::new();
    assert!(parse("[[1,9],[8,5]]", &mut basis).is_ok());
    assert_eq!(basis.elts,
               [Empty, Value(1), Pair, Value(9), Pair, Value(8), Pair, Value(5)]);
    assert_eq!(basis.to_string(), "[[1,9],[8,5]]");
}

impl<const N: usize> Basis<N> {
    fn explode(&mut self) -> bool {
        if let Some(node) = self.find_too_deep() {
            let (left, right) = self.children(node);
            let left_value = self.get(left);
            let right_value = self.get(right);
            self.change_bottom_pair_to_value(node, 0);
            if let Some(left_into) = self.find_left(node) {
                *self.get_mut(left_into) += left_value;
            }
            if let Some(right_into) = self.find_right(node) {
                *self.get_mut(right_into) += right_value;
            }
            true
        } else {
            false
        }
    }
}

#[test]
fn test_explode() {
    fn e(input: &str, expected: &str) {
        println!("Input: {:?}", input);
        let mut basis = AocBasis::new();
        assert!(parse(input, &mut basis).is_ok());
        println!("tree: {}", basis);
        println!("{:?}", basis);
        assert!(basis.explode());
        assert_eq!(basis.to_string(), expected);
    }

    e("[[[[[9,8],1],2],3],4]", "[[[[0,9],2],3],4]");
    e("[7,[6,[5,[4,[3,2]]]]]", "[7,[6,[5,[7,0]]]]");
    e("[[6,[5,[4,[3,2]]]],1]", "[[6,[5,[7,0]]],3]");
    e("[[3,[2,[1,[7,3]]]],[6,[5,[4,[3,2]]]]]", "[[3,[2,[8,0]]],[9,[5,[4,[3,2]]]]]");
    e("[[3,[2,[8,0]]],[9,[5,[4,[3,2]]]]]", "[[3,[2,[8,0]]],[9,[5,[7,0]]]]");
}

impl<const N: usize> Basis<N> {
    fn split(&mut self) -> bool {
        if let Some(node) = self.find_needs_split() {
            let value = self.get(node);
            let (left, right) = self.set_pair(node);
            self.set_const(left, value / 2);
            self.set_const(right, value - value / 2);
            true
        } else {
            false
        }
    }
}

#[test]
fn test_split() {
    fn s(input: &str, expected: &str) {
        println!("Input: {:?}", input);
        let mut basis = AocBasis::new();
        assert!(parse(input, &mut basis).is_ok());
        println!("tree: {}", basis);
        println!("{:?}", basis);
        assert!(basis.split());
        assert_eq!(basis.to_string(), expected);
    }

    s("[[[[0,7],4],[15,[0,13]]],[1,1]]",
      "[[[[0,7],4],[[7,8],[0,13]]],[1,1]]");

    s("[[[[0,7],4],[[7,8],[0,13]]],[1,1]]",
      "[[[[0,7],4],[[7,8],[0,[6,7]]]],[1,1]]");
}

impl<const N: usize> Basis<N> {
    fn reduce(&mut self) {
        while self.explode() || self.split() {}
    }
}

#[test]
fn test_reduce() {
    fn r(input: &str, expected: &str) {
        println!("Input: {:?}", input);
        let mut basis = AocBasis::new();
        assert!(parse(input, &mut basis).is_ok());
        println!("tree: {}", basis);
        println!("{:?}", basis);
        basis.reduce();
        assert_eq!(basis.to_string(), expected);
    }

    r("[[[[[4,3],4],4],[7,[[8,4],9]]],[1,1]]",
      "[[[[0,7],4],[[7,8],[6,0]]],[8,1]]");
}

impl<const N: usize> Basis<N> {
    fn add(&mut self, other: &Self) {
        assert!((0 .. N/2).all(|i| self.elts[2 * i + 1] == Elt::Empty));
        assert!((0 .. N/2).all(|i| other.elts[2 * i + 1] == Elt::Empty));
        for i in 0 .. N/2 {
            self.elts[i] = self.elts[2 * i];
        }
        for i in 0 .. N/2 {
            self.elts[N/2 + i] = other.elts[2 * i];
        }
        assert!(self.elts[N/2] == Elt::Empty);
        self.elts[N/2] = Elt::Pair;

        self.reduce();
    }
}

#[test]
fn test_sums() {
    fn s(input: &[&str], output: &str) {
        let mut sum = AocBasis::new();
        let mut right = AocBasis::new();
        assert!(parse(input[0], &mut sum).is_ok());
        for s in &input[1..] {
            assert!(parse(s, &mut right).is_ok());
            sum.add(&right);
        }
        assert_eq!(sum.to_string(), output);
    }

    s(&[
        "[1,1]",
        "[2,2]",
        "[3,3]",
        "[4,4]",
    ],
      "[[[[1,1],[2,2]],[3,3]],[4,4]]");

    s(&[
        "[1,1]",
        "[2,2]",
        "[3,3]",
        "[4,4]",
        "[5,5]",
    ],
      "[[[[3,0],[5,3]],[4,4]],[5,5]]");

    s(&[
        "[1,1]",
        "[2,2]",
        "[3,3]",
        "[4,4]",
        "[5,5]",
        "[6,6]",
    ],
      "[[[[5,0],[7,4]],[5,5]],[6,6]]");

    s(&[
        "[[[0,[4,5]],[0,0]],[[[4,5],[2,6]],[9,5]]]",
        "[7,[[[3,7],[4,3]],[[6,3],[8,8]]]]",
        "[[2,[[0,8],[3,4]]],[[[6,7],1],[7,[1,6]]]]",
        "[[[[2,4],7],[6,[0,5]]],[[[6,8],[2,8]],[[2,1],[4,5]]]]",
        "[7,[5,[[3,8],[1,4]]]]",
        "[[2,[2,2]],[8,[8,1]]]",
        "[2,9]",
        "[1,[[[9,3],9],[[9,0],[0,7]]]]",
        "[[[5,[7,4]],7],1]",
        "[[[[4,2],2],6],[8,7]]",
    ],
      "[[[[8,7],[7,7]],[[8,6],[7,7]]],[[[0,7],[6,6]],[8,7]]]");
}

impl<const N: usize> Basis<N> {
    fn magnitude_from(&self, n: Node) -> usize {
        match self.elts[n] {
            Elt::Empty => panic!("malformed tree"),
            Elt::Pair => {
                let (left, right) = self.children(n);
                3 * self.magnitude_from(left) + 2 * self.magnitude_from(right)
            }
            Elt::Value(v) => v as usize,
        }
    }
    fn magnitude(&self) -> usize {
        self.magnitude_from(Self::ROOT)
    }
}

#[test]
fn test_magnitude() {
    fn m(input: &str) -> usize {
        let mut basis = AocBasis::new();
        assert!(parse(input, &mut basis).is_ok());
        basis.magnitude()
    }

    assert_eq!(m("[9,1]"), 29);
    assert_eq!(m("[[9,1],[1,9]]"), 129);

    assert_eq!(m("[[1,2],[[3,4],5]]"), 143);
    assert_eq!(m("[[[[0,7],4],[[7,8],[6,0]]],[8,1]]"), 1384);
    assert_eq!(m("[[[[1,1],[2,2]],[3,3]],[4,4]]"), 445);
    assert_eq!(m("[[[[3,0],[5,3]],[4,4]],[5,5]]"), 791);
    assert_eq!(m("[[[[5,0],[7,4]],[5,5]],[6,6]]"), 1137);
    assert_eq!(m("[[[[8,7],[7,7]],[[8,6],[7,7]]],[[[0,7],[6,6]],[8,7]]]"), 3488);
}

fn sum_list<'i, I, T>(list: I) -> Result<AocBasis>
    where I: IntoIterator<Item = &'i T> + 'i,
          T: AsRef<str> + 'i,
          T: ?Sized,
{
    let mut left = AocBasis::new();
    let mut right = AocBasis::new();
    let mut list = list.into_iter();
    let first = list.next().unwrap().as_ref();
    parse(first, &mut left)?;

    for next in list {
        parse(next.as_ref(), &mut right)?;
        left.add(&right);
    }

    Ok(left)
}

#[test]
fn test_homework() -> Result<()> {
    let lines = include_str!("sample/day18.homework").lines();
    let sum = sum_list(lines)?;
    assert_eq!(sum.magnitude(), 4140);
    Ok(())
}

#[aoc_generator(day18, part1, jimb_heap)]
fn generator(input: &str) -> Vec<String> {
    input.lines().map(|s| s.to_owned()).collect()
}

#[aoc(day18, part1, jimb_heap)]
fn part1(input: &Vec<String>) -> usize {
    let sum = sum_list(input).unwrap();
    sum.magnitude()
}
