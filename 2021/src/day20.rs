use aoc_runner_derive::{aoc, aoc_generator};
use anyhow::{anyhow, bail, Result};
use ndarray::{Array2, s};
#[cfg(test)]
use ndarray::array;
use std::{fmt, ops};

type Rule = [bool; 512];
#[derive(Clone, Debug, Eq, PartialEq)]
struct State {
    inside: Array2<bool>,
    outside: bool,
}

impl State {
    #[inline]
    fn check_index(&self, index: [isize; 2]) -> Option<[usize; 2]> {
        if index[0] < 0 || index[1] < 0 {
            return None;
        }

        let index = [index[0] as usize, index[1] as usize];
        let dim = self.inside.dim();
        if index[0] >= dim.0 || index[1] >= dim.1 {
            return None;
        }

        Some(index)
    }

    fn enlarge(&mut self) {
        let old_size = self.inside.dim();
        let mut new = Array2::from_elem((old_size.0 + 2, old_size.1 + 2), self.outside);
        new.slice_mut(s![1 .. old_size.0 + 1, 1 .. old_size.1 + 1])
            .assign(&self.inside);

        self.inside = new;
    }
}

impl ops::Index<[isize; 2]> for State {
    type Output = bool;

    fn index(&self, index: [isize; 2]) -> &bool {
        match self.check_index(index) {
            Some(index) => &self.inside[index],
            None => &self.outside
        }
    }
}

impl ops::IndexMut<[isize; 2]> for State {
    fn index_mut(&mut self, index: [isize; 2]) -> &mut bool {
        match self.check_index(index) {
            Some(index) => &mut self.inside[index],
            None => panic!("Attempt to store outside of allocated space at {:?}", index),
        }
    }
}

impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let dim = self.inside.dim();
        for row in -1 .. dim.0 as isize + 1 {
            for col in -1 .. dim.1 as isize + 1 {
                write!(f, "{}", char_from_pixel(self[[row, col]]))?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

#[test]
fn test_state() {
    let mut state = State { inside: Array2::default((8, 16)), outside: true };

    assert_eq!(state[[-5,5]], true);

    state[[5, 5]] = true;
    assert_eq!(state[[ 5,   5]], true);

    assert_eq!(state[[0,  0]], false);
    assert_eq!(state[[0, 15]], false);
    assert_eq!(state[[7,  0]], false);
    assert_eq!(state[[7, 15]], false);

    assert_eq!(state[[ 0, -1]], true);
    assert_eq!(state[[ 0, 16]], true);
    assert_eq!(state[[-1,  0]], true);
    assert_eq!(state[[ 8,  0]], true);

    state.enlarge();
    state.enlarge();
    state.enlarge();
    state.enlarge();

    assert_eq!(state[[ 9,  9]], true);

    assert_eq!(state[[ 4,  4]], false);
    assert_eq!(state[[ 4, 19]], false);
    assert_eq!(state[[11,  4]], false);
    assert_eq!(state[[11, 19]], false);

    assert_eq!(state[[ 3,  4]], true);
    assert_eq!(state[[ 3, 19]], true);
    assert_eq!(state[[11,  3]], true);
    assert_eq!(state[[11, 20]], true);

}

fn pixel_from_char(ch: char) -> Result<bool> {
    match ch {
        '#' => Ok(true),
        '.' => Ok(false),
        _ => bail!("Bad character in rule: {:?}", ch),
    }
}

fn char_from_pixel(p: bool) -> char {
    if p { '#' } else { '.' }
}

fn parse_state<'a>(lines: impl Iterator<Item = &'a str> + Clone) -> Result<State> {
    let height = lines.clone().count();
    if height == 0 {
        bail!("no initial state in input");
    }
    let width = lines.clone().next().unwrap().chars().count();

    let mut inside = Array2::from_elem((height, width), false);
    for (row, line) in lines.enumerate() {
        for (col, ch) in line.chars().enumerate() {
            inside[[row, col]] = pixel_from_char(ch)?;
        }
    }

    Ok(State { inside, outside: false })
}

#[aoc_generator(day20, part1, jimb)]
#[aoc_generator(day20, part2, jimb)]
fn generator(input: &str) -> Result<(Rule, State)> {
    let mut lines = input.lines();
    let rule_text = lines.next()
        .ok_or_else(|| anyhow!("input missing rule line"))?;
    if rule_text.len() != 512 {
        bail!("Rule line has bad length: {}", rule_text.len())
    }
    let mut rule = [false; 512];
    for (i, ch) in rule_text.chars().enumerate() {
        rule[i] = pixel_from_char(ch)?;
    }

    match lines.next() {
        Some("") => {},
        other => bail!("expected blank line following rule, got {:?}", other),
    }

    let state = parse_state(lines)?;

    Ok((rule, state))
}

#[cfg(test)]
fn sample() -> (Rule, State) {
    generator(include_str!("sample/day20"))
        .expect("failed to parse sample")
}

#[test]
fn test_generator() {
    let (_rule, state) = sample();
    assert_eq!(state.outside, false);
    assert_eq!(state.inside,
               array![
                   [  true, false, false,  true, false ],
                   [  true, false, false, false, false ],
                   [  true,  true, false, false, true  ],
                   [ false, false,  true, false, false ],
                   [ false, false,  true,  true, true  ],
               ]);
}

fn index(state: &State, row: isize, col: isize) -> usize {
    let mut bits = 0;
    for (drow, dcol) in crate::cartesian_product(-1..=1, -1..=1) {
        let bit = state[[row + drow, col + dcol]] as usize;
        bits = bits << 1 | bit;
    }
    bits
}

impl State {
    fn step(&self, rule: &Rule) -> State {
        let mut next = self.clone();
        next.enlarge();

        for row in 0 .. next.inside.nrows() as isize {
            for col in 0 .. next.inside.ncols() as isize {
                next[[row, col]] = rule[index(self, row - 1, col - 1)];
            }
        }

        next.outside = rule[if self.outside { 511 } else { 0 }];

        next
    }
}

#[test]
fn test_step() {
    let (rule, state) = sample();

    println!("{}", state);
    let next = state.step(&rule);
    println!("\n{}", next);
    let expected = parse_state(include_str!("sample/day20.step2").lines()).unwrap();
    println!("\n{}", expected);
    assert_eq!(next, expected);

    let next = next.step(&rule);
    println!("\n{}", next);
    let expected = parse_state(include_str!("sample/day20.step3").lines()).unwrap();
    println!("\n{}", expected);
    assert_eq!(next, expected)
}

#[aoc(day20, part1, jimb)]
fn part1((rule, initial): &(Rule, State)) -> usize {
    let out = initial.step(rule).step(rule);

    assert_eq!(out.outside, false); // because otherwise the answer is infinity
    out.inside.iter().filter(|&&pixel| pixel).count()
}

#[test]
fn test_part1() {
    assert_eq!(part1(&sample()), 35);
}

#[aoc(day20, part2, jimb)]
fn part2((rule, state): &(Rule, State)) -> usize {
    let mut state = state.clone();
    for _ in 0..50 {
        state = state.step(rule);
    }

    state.inside.iter().filter(|&&pixel| pixel).count()
}
