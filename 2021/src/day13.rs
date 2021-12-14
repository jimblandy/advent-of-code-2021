#![allow(unused_imports, dead_code)]

use aoc_runner_derive::{aoc, aoc_generator};
use anyhow::{anyhow, Result};
use std::cmp;
use std::collections::HashSet;
use std::str::FromStr;

#[derive(Debug, PartialEq)]
struct Instructions {
    points: Vec<(i32, i32)>,
    folds: Vec<Fold>,
}

#[derive(Debug, PartialEq)]
enum Fold {
    Up(i32),   // "along the line y = ..."
    Left(i32), // "along the line x = ..."
}

impl Fold {
    fn apply(&self, (x, y): (i32, i32)) -> (i32, i32) {
        fn flip(about: i32, n: i32) -> i32 {
            if n <= about {
                n
            } else {
                2 * about - n
            }
        }

        match *self {
            Fold::Up(about) => (x, flip(about, y)),
            Fold::Left(about) => (flip(about, x), y),
        }
    }
}

#[aoc_generator(day13, part1, jimb)]
#[aoc_generator(day13, part2, jimb)]
fn generate(input: &str) -> Result<Instructions> {
    let mut lines = input.lines();
    let points: Vec<_> = lines
        .by_ref()
        .take_while(|l| !l.trim().is_empty())
        .map(|l| {
            let (x,y) = l.split_once(',')
                .ok_or_else(|| anyhow!("dot is missing ',' separator: {:?}", l))?;
            Ok((i32::from_str(x.trim())?, i32::from_str(y.trim())?))
        })
        .collect::<Result<_>>()?;

    let folds: Vec<_> = lines
        .map(|l| {
            if let Some(("", rest)) = l.split_once("fold along x=") {
                Ok(Fold::Left(i32::from_str(rest.trim())?))
            } else if let Some(("", rest)) = l.split_once("fold along y=") {
                Ok(Fold::Up(i32::from_str(rest.trim())?))
            } else {
                Err(anyhow!("Bad fold instruction: {:?}", l))
            }
        })
        .collect::<Result<_>>()?;

    Ok(Instructions { points, folds })
}

#[test]
fn test_generate() {
    assert_eq!(generate(include_str!("sample/day13")).unwrap(),
               Instructions {
                   points: vec![
                       (6,10),
                       (0,14),
                       (9,10),
                       (0,3),
                       (10,4),
                       (4,11),
                       (6,0),
                       (6,12),
                       (4,1),
                       (0,13),
                       (10,12),
                       (3,4),
                       (3,0),
                       (8,4),
                       (1,10),
                       (2,14),
                       (8,10),
                       (9,0),
                   ],
                   folds: vec![
                       Fold::Up(7),
                       Fold::Left(5),
                   ]
               })
}

#[aoc(day13, part1, jimb)]
fn part1(input: &Instructions) -> usize {
    input.points
        .iter()
        .map(|&pt| input.folds[0].apply(pt))
        .collect::<HashSet<(i32, i32)>>()
        .len()
}

#[test]
fn test_part1() {
    let instructions = generate(include_str!("sample/day13")).unwrap();
    assert_eq!(part1(&instructions), 17);
}

#[aoc(day13, part2, jimb)]
fn part2(input: &Instructions) -> &'static str {
    let folded = input.points
        .iter()
        .map(|&pt| input.folds.iter().fold(pt, |pt, fold| fold.apply(pt)))
        .collect::<HashSet<(i32, i32)>>();

    let extent = folded.iter().fold((0,0), |(w, h), &(x, y)| (cmp::max(x, w), cmp::max(y, h)));
    for y in 0 ..= extent.1 {
        for x in 0 ..= extent.0 {
            print!("{} ", if folded.contains(&(x, y)) { "#" } else { " " });
        }
        println!();
    }

    "see output"
}
