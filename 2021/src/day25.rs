use aoc_runner_derive::{aoc, aoc_generator};
use ndarray::Array2;
use anyhow::{anyhow, bail, Result};
use std::fmt;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum Spot {
    Empty,
    East,
    South,
}

use Spot::*;

#[aoc_generator(day25)]
fn generate(input: &str) -> Result<Array2<Spot>> {
    let width = input.lines().next().ok_or(anyhow!("should have at least one line of input"))?.chars().count();
    let height = input.lines().count();

    let mut map = Array2::from_elem((height, width), Empty);

    for (row, line) in input.lines().enumerate() {
        for (col, ch) in line.chars().enumerate() {
            match ch {
                '.' => { map[[row, col]] = Empty }
                '>' => { map[[row, col]] = East }
                'v' => { map[[row, col]] = South }
                _ => bail!("unexpected character in map: {:?}", ch),
            }
        }
    }

    Ok(map)
}

#[test]
fn test_generate() {
    let map = generate(include_str!("sample/day25")).unwrap();

    assert_eq!(map.dim(), (9, 10));
}

fn advance(map: &mut Array2<Spot>) -> bool {
    let dim = map.dim();
    let mut skip_next = false;
    let mut any = false;

    let mut to_clear = vec![];
    for row in 0 .. dim.0 {
        for col in 0 .. dim.1 {
            if skip_next {
                skip_next = false;
                continue;
            }
            if map[[row, col]] == East {
                let next = (col + 1) % dim.1;
                if map[[row, next]] == Empty {
                    to_clear.push([row, col]);
                    map[[row, next]] = East;
                    skip_next = true;
                    any = true;
                }
            }
        }
        skip_next = false;
    }

    for old in to_clear.drain(..) {
        map[old] = Empty;
    }

    for col in 0 .. dim.1 {
        for row in 0 .. dim.0 {
            if skip_next {
                skip_next = false;
                continue;
            }
            if map[[row, col]] == South {
                let next = (row + 1) % dim.0;
                if map[[next, col]] == Empty {
                    to_clear.push([row, col]);
                    map[[next, col]] = South;
                    skip_next = true;
                    any = true;
                }
            }
        }
        skip_next = false;
    }

    for old in to_clear.drain(..) {
        map[old] = Empty;
    }

    any
}

#[test]
fn test_advance() {
    let mut map = generate(include_str!("sample/day25")).unwrap();

    assert_eq!(Pretty(&map).to_string(),
               "v...>>.vv>\n\
                .vv>>.vv..\n\
                >>.>v>...v\n\
                >>v>>.>.v.\n\
                v>v.vv.v..\n\
                >.>>..v...\n\
                .vv..>.>v.\n\
                v.v..>>v.v\n\
                ....v..v.>\n\
                ");

    for _ in 0..5 {
        assert_eq!(advance(&mut map), true);
    }

    assert_eq!(Pretty(&map).to_string(),

"\
vv>...>v>.
v.v.v>.>v.
>.v.>.>.>v
>v>.>..v>>
..v>v.v...
..>.>>vvv.
.>...v>v..
..v.v>>v.v
v.v.>...v.
"
               );
}

#[aoc(day25, part1)]
fn part1(map: &Array2<Spot>) -> usize {
    let mut map = map.clone();

    std::iter::repeat(()).take_while(|_| advance(&mut map)).count() + 1
}

#[test]
fn test_part1() {
    let map = generate(include_str!("sample/day25")).unwrap();

    assert_eq!(part1(&map), 58);
}

struct Pretty<T>(T);

impl fmt::Display for Spot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ch = match *self {
            Empty => '.',
            East => '>',
            South => 'v',
        };
        std::fmt::Write::write_char(f, ch)
    }
}

impl fmt::Display for Pretty<&'_ Array2<Spot>> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let dim = self.0.dim();
        for row in 0 .. dim.0 {
            for col in 0 .. dim.1 {
                write!(f, "{}", self.0[[row, col]])?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}
