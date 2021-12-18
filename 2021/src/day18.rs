#![allow(unused_imports, dead_code)]

use aoc_runner_derive::{aoc, aoc_generator};
use anyhow::{anyhow, bail, Result};
use crate::cartesian_product;
use std::fmt;
use std::mem::replace;
use std::num::ParseIntError;
use std::str::FromStr;

#[derive(Clone, Debug, Eq, PartialEq)]
enum Num {
    Pair(Box<Num>, Box<Num>),
    Const(u64),
}

use Num::*;

impl Num {
    fn pair(left: Num, right: Num) -> Num {
        Pair(Box::new(left), Box::new(right))
    }

    fn magnitude(&self) -> u64 {
        match *self {
            Pair(ref left, ref right) => 3 * left.magnitude() + 2 * right.magnitude(),
            Const(n) => n,
        }
    }
}

impl fmt::Display for Num {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Pair(ref left, ref right) => {
                write!(f, "[{},{}]", **left, **right)
            }
            Const(n) => {
                write!(f, "{}", n)
            }
        }
    }
}

#[test]
fn test_num_display() {
    assert_eq!(parse_num("[[3,[2,[1,[7,3]]]],[6,[5,[4,[3,2]]]]]").unwrap().to_string(),
               "[[3,[2,[1,[7,3]]]],[6,[5,[4,[3,2]]]]]");
}

#[aoc_generator(day18)]
fn generator(input: &str) -> Result<Vec<Num>> {
    input.lines().map(parse_num).collect()
}

fn parse_num(input: &str) -> Result<Num> {
    fn parse(input: &str) -> Result<(Num, &str)> {
        if input.starts_with("[") {
            let (left, rest) = parse(&input[1..])?;
            let rest = match rest.split_once(",") {
                Some(("", rest)) => rest,
                _ => bail!("missing , in pair: {:?}, rest = {:?}", input, rest),
            };
            let (right, rest) = parse(rest)?;
            match rest.split_once("]") {
                Some(("", rest)) => Ok((Num::pair(left, right), rest)),
                _ => bail!("expected closing bracket: {:?}", input),
            }
        } else {
            match input.find(|ch: char| !ch.is_digit(10)) {
                Some(0) => bail!("confusing start to input: {:?}", input),
                Some(index) => {
                    let value = u64::from_str(&input[..index])?;
                    Ok((Const(value), &input[index..]))
                }
                None => {
                    let value = u64::from_str(input)?;
                    Ok((Const(value), ""))
                }
            }
        }
    }

    let (num, rest) = parse(input.trim())?;
    if !rest.is_empty() {
        bail!("Garbage at end of input: {:?}", rest);
    }

    Ok(num)
}

#[test]
fn test_generator() {
    assert_eq!(parse_num("[1,2]").unwrap(),
               Num::pair(Const(1), Const(2)));
    assert_eq!(parse_num("[[1,9],[8,5]]").unwrap(),
               Num::pair(
                   Num::pair(Const(1), Const(9)),
                   Num::pair(Const(8), Const(5))
               ));
}

#[derive(Debug, PartialEq)]
enum Explode {
    Explode(Option<u64>, Option<u64>),
    Ok,
}

fn explode_once(num: &mut Num, within: usize) -> Explode {
    match *num {
        Const(_) => Explode::Ok,
        Pair(ref left, ref right) if within >= 4 => {
            match (&**left, &**right) {
                (&Const(left), &Const(right)) => {
                    *num = Const(0);
                    return Explode::Explode(Some(left), Some(right));
                }
                (_, _) => panic!("Pair to explode was not two numbers: {:?}", num),
            }
        }
        Pair(ref mut left, ref mut right) => {
            match explode_once(left, within + 1) {
                Explode::Explode(el, er) => {
                    if let Some(er) = er {
                        insert_at_left(er, right);
                    }
                    return Explode::Explode(el, None);
                }
                Explode::Ok => {}
            }
            match explode_once(right, within + 1) {
                Explode::Explode(el, er) => {
                    if let Some(el) = el {
                        insert_at_right(el, left);
                    }
                    return Explode::Explode(None, er);
                }
                Explode::Ok => {}
            }

            Explode::Ok
        }
    }
}

#[derive(Debug, PartialEq)]
enum Split {
    Split,
    Ok
}

fn split_once(num: &mut Num) -> Split {
    match *num {
        Const(n) if n < 10 => Split::Ok,
        Const(n) => {
            *num = Num::pair(Const(n / 2), Const(n - n / 2));
            Split::Split
        }
        Pair(ref mut left, ref mut right) => {
            match split_once(left) {
                Split::Split => return Split::Split,
                Split::Ok => {}
            }
            match split_once(right) {
                Split::Split => return Split::Split,
                Split::Ok => {}
            }

            Split::Ok
        }
    }
}

#[derive(Debug, PartialEq)]
enum Out {
    Explode(Option<u64>, Option<u64>),
    Split,
    Ok,
}

fn reduce_once(num: &mut Num) -> Out {
    if let Explode::Explode(l, r) = explode_once(num, 0) {
        return Out::Explode(l, r);
    }

    if let Split::Split = split_once(num) {
        return Out::Split;
    }

    Out::Ok
}

fn insert_at_left(n: u64, num: &mut Num) {
    match *num {
        Const(ref mut k) => *k += n,
        Pair(ref mut left, _) => insert_at_left(n, &mut **left),
    }
}

fn insert_at_right(n: u64, num: &mut Num) {
    match *num {
        Const(ref mut k) => *k += n,
        Pair(_, ref mut right) => insert_at_right(n, &mut **right),
    }
}

#[test]
fn test_reduce_once() {
    fn g(input: &str, out: Out, output: &str) {
        let mut num = parse_num(input).unwrap();
        assert_eq!(reduce_once(&mut num), out);
        assert_eq!(num.to_string(), output);
    }

    g("[[[[[9,8],1],2],3],4]",
      Out::Explode(Some(9), None),
      "[[[[0,9],2],3],4]");

    g("[7,[6,[5,[4,[3,2]]]]]",
      Out::Explode(None, Some(2)),
      "[7,[6,[5,[7,0]]]]");

    g("[[6,[5,[4,[3,2]]]],1]",
      Out::Explode(None, None),
      "[[6,[5,[7,0]]],3]");

    // (the pair [3,2] is unaffected because the pair [7,3] is further to the
    // left; [3,2] would explode on the next action).
    g("[[3,[2,[1,[7,3]]]],[6,[5,[4,[3,2]]]]]",
      Out::Explode(None, None),
      "[[3,[2,[8,0]]],[9,[5,[4,[3,2]]]]]");

    g("[[3,[2,[8,0]]],[9,[5,[4,[3,2]]]]]",
      Out::Explode(None, Some(2)),
      "[[3,[2,[8,0]]],[9,[5,[7,0]]]]");

    g("[[[[0,7],4],[15,[0,13]]],[1,1]]",
      Out::Split,
      "[[[[0,7],4],[[7,8],[0,13]]],[1,1]]");

    g("[[[[0,7],4],[[7,8],[0,13]]],[1,1]]",
      Out::Split,
      "[[[[0,7],4],[[7,8],[0,[6,7]]]],[1,1]]");
}

fn reduce(num: &mut Num) {
    loop {
        let out = reduce_once(num);
        match out {
            Out::Ok => break,
            _ => ()
        }
    }
}

#[test]
fn test_reduce() {
    fn g(input: &str, output: &str) {
        let mut num = parse_num(input).unwrap();
        reduce(&mut num);
        assert_eq!(num.to_string(), output);
    }

    g("[[[[[4,3],4],4],[7,[[8,4],9]]],[1,1]]",
      "[[[[0,7],4],[[7,8],[6,0]]],[8,1]]");
}

fn add(left: Num, right: Num) -> Num {
    let mut sum = Num::pair(left, right);
    reduce(&mut sum);
    sum
}

#[test]
fn test_sums() {
    fn g(input: &[&str], output: &str) {
        let sum = input[1..].iter().fold(parse_num(input[0]).unwrap(),
                                         |acc, &n| add(acc, parse_num(n).unwrap()));
        assert_eq!(sum.to_string(), output);
    }

    g(&[
        "[1,1]",
        "[2,2]",
        "[3,3]",
        "[4,4]",
    ],
      "[[[[1,1],[2,2]],[3,3]],[4,4]]");

    g(&[
        "[1,1]",
        "[2,2]",
        "[3,3]",
        "[4,4]",
        "[5,5]",
    ],
      "[[[[3,0],[5,3]],[4,4]],[5,5]]");

    g(&[
        "[1,1]",
        "[2,2]",
        "[3,3]",
        "[4,4]",
        "[5,5]",
        "[6,6]",
    ],
      "[[[[5,0],[7,4]],[5,5]],[6,6]]");

    g(&[
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

#[test]
fn test_magnitude() {
    assert_eq!(parse_num("[[9,1],[1,9]]").unwrap().magnitude(),
               129);
}

#[aoc(day18, part1)]
fn part1(input: &Vec<Num>) -> u64 {
   (&input[1..]).iter().cloned().fold(input[0].clone(), add).magnitude()
}

#[test]
fn test_part1() {
    assert_eq!(part1(&generator(include_str!("sample/day18.homework")).unwrap()),
               4140);
}

#[aoc(day18, part2)]
fn test_part2(input: &Vec<Num>) -> u64 {
    cartesian_product(0..input.len(), 0..input.len())
        .filter(|(i, j)| i != j)
        .map(|(i, j)| add(input[i].clone(), input[j].clone()).magnitude())
        .max()
        .unwrap()
}
