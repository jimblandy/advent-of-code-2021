#![allow(dead_code, unused_variables)]

use std::{mem, sync::LazyLock};

mod input;

struct Problem {
    width: usize,
    start: usize,
    rows: Vec<Row>,
}

struct Row {
    splitters: Vec<usize>,
}

static TEST_INPUT: LazyLock<Problem> = LazyLock::new(|| Problem::parse(TEST_INPUT_TEXT));

static TEST_INPUT_TEXT: &str = "\
.......S.......
...............
.......^.......
...............
......^.^......
...............
.....^.^.^.....
...............
....^.^...^....
...............
...^.^...^.^...
...............
..^...^.....^..
...............
.^.^.^.^.^...^.
...............
";

impl Problem {
    fn parse(input: &str) -> Self {
        let mut lines = input.lines();
        let width = lines.clone().map(|line| line.len()).max().unwrap();
        let start = lines.next().unwrap().chars().position(|ch| ch == 'S').unwrap();
        let rows = lines
            .map(|line| Row {
                splitters: line.chars()
                    .enumerate()
                    .filter(|&(_, ch)| ch == '^')
                    .map(|(col, _)| col)
                    .collect()
            })
            .collect();
        Problem { width, start, rows }
    }
}

fn part1(problem: &Problem) -> usize {
    let mut next = Vec::with_capacity(problem.width);

    let mut current = vec![false; problem.width];
    current[problem.start] = true;

    let mut splits = 0;
    for row in &problem.rows {
        next.clone_from(&current);
        for &splitter in &row.splitters {
            if current[splitter] {
                next[splitter - 1] = true;
                next[splitter] = false;
                next[splitter + 1] = true;
                splits += 1;
            }
        }
        mem::swap(&mut current, &mut next);
    }

    splits
}

#[test]
fn test_part1() {
    assert_eq!(part1(&TEST_INPUT), 21);
}

fn part2(problem: &Problem) -> usize {
    let mut next = Vec::with_capacity(problem.width);

    let mut current = vec![0; problem.width];
    current[problem.start] = 1;

    for row in &problem.rows {
        next.clone_from(&current);
        for &splitter in &row.splitters {
            let here = current[splitter];
            next[splitter - 1] += here;
            next[splitter] = 0;
            next[splitter + 1] += here;
        }
        mem::swap(&mut current, &mut next);
    }

    current.iter().sum()
}

#[test]
fn test_part2() {
    assert_eq!(part2(&TEST_INPUT), 40);
}

fn main() {
    println!("part 1: {}", part1(&Problem::parse(include_str!("input.txt"))));
    println!("part 2: {}", part2(&Problem::parse(include_str!("input.txt"))));
}
