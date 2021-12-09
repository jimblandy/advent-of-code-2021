use aoc_runner_derive::{aoc, aoc_generator};
use std::str::FromStr;
use std::num::ParseIntError;

#[aoc_generator(day7)]
fn part1_input(input: &str) -> Result<Vec<i32>, ParseIntError> {
    input.split(',')
        .map(|s| i32::from_str(s))
        .collect()
}

fn part1_median(crabs: &[i32]) -> i32 {
    let mut crabs = crabs.to_vec();
    crabs.sort();
    let mid = crabs.len() / 2;
    let pos = if crabs.len() & 1 == 0 {
        assert_eq!(crabs[mid - 1], crabs[mid]);
        crabs[mid]
    } else {
        crabs[mid + 1]
    };

    crabs.iter().map(|c| (c - pos).abs()).sum()
}

#[cfg(test)]
fn sample() -> Vec<i32> {
    part1_input("16,1,2,0,4,2,7,1,2,14")
        .expect("error parsing sample input")
}

#[test]
fn part1_sample() {
    assert_eq!(part1_median(&sample()), 37);
}

#[aoc(day7, part1)]
fn part1(crabs: &Vec<i32>) -> i32 {
    part1_median(crabs)
}

fn cost(dist: i32) -> i32 { (dist * dist + dist) / 2 }

fn cost_all(pos: i32, crabs: &[i32]) -> i32 {
    crabs.iter().map(|crab| cost((crab - pos).abs())).sum()
}

fn part2_mean(crabs: &[i32]) -> i32 {
    let sum: i32 = crabs.iter().sum();
    let mean = sum as f32 / crabs.len() as f32;
    let pos1 = mean.floor() as i32;
    let pos2 = mean.ceil() as i32;
    std::cmp::min(cost_all(pos1, crabs),
                  cost_all(pos2, crabs))
}

#[test]
fn part2_sample() {
    assert_eq!(part2_mean(&sample()), 168);
}

#[aoc(day7, part2)]
fn part2(crabs: &Vec<i32>) -> i32 {
    part2_mean(crabs)
}

