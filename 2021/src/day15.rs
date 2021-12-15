#![allow(unused_imports, dead_code)]

use aoc_runner_derive::{aoc, aoc_generator};
use anyhow::{anyhow, bail, Result};
use crate::astar_weighted::astar_weighted;
use crate::{compass, around};
use ndarray::Array2;
#[cfg(test)]
use ndarray::array;

#[aoc_generator(day15)]
fn generate(input: &str) -> Result<Array2<u32>> {
    let width = input.lines().map(|l| l.len()).max().unwrap();
    let height = input.lines().count();

    let mut map = Array2::zeros([height, width]);

    for (row, line) in input.lines().enumerate() {
        for (col, ch) in line.chars().enumerate() {
            map[[row, col]] = ch.to_digit(10)
                .ok_or_else(|| anyhow!("bad digit in map: {:?}", ch))?;
        }
    }

    Ok(map)
}

#[test]
fn test_generate() {
    assert_eq!(generate(include_str!("sample/day15")).unwrap(),
               array![
                   [1, 1, 6, 3, 7, 5, 1, 7, 4, 2],
                   [1, 3, 8, 1, 3, 7, 3, 6, 7, 2],
                   [2, 1, 3, 6, 5, 1, 1, 3, 2, 8],
                   [3, 6, 9, 4, 9, 3, 1, 5, 6, 9],
                   [7, 4, 6, 3, 4, 1, 7, 1, 1, 1],
                   [1, 3, 1, 9, 1, 2, 8, 1, 3, 7],
                   [1, 3, 5, 9, 9, 1, 2, 4, 2, 1],
                   [3, 1, 2, 5, 4, 2, 1, 6, 3, 9],
                   [1, 2, 9, 3, 1, 3, 8, 5, 2, 1],
                   [2, 3, 1, 1, 9, 4, 4, 5, 8, 1]
               ]);
}

#[aoc(day15, part1, jimb)]
fn part1(input: &Array2<u32>) -> usize {
    let size = input.dim();
    let end = [size.0 - 1, size.1 - 1];

    let mut search = astar_weighted([0,0], |&p: &[usize; 2]| {
        around(p, input.dim(), compass())
            .map(|n| {
                (n, input[n] as usize,
                 (end[0] - n[0] + end[1] - n[1]))
            })
    });

    search.find(|edge| edge.to == end)
        .expect("Didn't find any path to end")
        .path_weight
}

#[test]
fn test_part1() {
    assert_eq!(part1(&generate(include_str!("sample/day15")).unwrap()),
               40);
}

#[aoc(day15, part2, jimb)]
fn part2(input: &Array2<u32>) -> usize {
    let tile_size = input.dim();
    let size = (tile_size.0 * 5, tile_size.1 * 5);
    let end = [size.0 - 1, size.1 - 1];

    let mut search = astar_weighted([0,0], |&p: &[usize; 2]| {
        around(p, size, compass())
            .map(|n| {
                let reduced = [n[0] % tile_size.0, n[1] % tile_size.1];
                let offset = n[0] / tile_size.0 + n[1] / tile_size.1;
                let weight = ((input[reduced] as usize + offset) - 1) % 9 + 1;
                (n, weight, (end[0] - n[0] + end[1] - n[1]))
            })
    });

    search.find(|edge| edge.to == end)
        .expect("Didn't find any path to end")
        .path_weight
}

#[test]
fn test_part2() {
    assert_eq!(part2(&generate(include_str!("sample/day15")).unwrap()),
               315);
}
