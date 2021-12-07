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

#[test]
fn part1_sample() -> Result<(), ParseIntError> {
    assert_eq!(part1(&part1_input("16,1,2,0,4,2,7,1,2,14")?),
               37);

    Ok(())
}

#[aoc(day7, part1)]
fn part1(crabs: &Vec<i32>) -> i32 {
    part1_median(crabs)
}
