#![allow(dead_code, unused_variables)]

use std::{cmp::max, ops::RangeInclusive, sync::LazyLock};

mod input;

#[derive(Clone)]
struct Problem {
    fresh: Vec<RangeInclusive<usize>>,
    available: Vec<usize>,
}

static TEST_INPUT: LazyLock<Problem> = LazyLock::new(|| Problem {
    fresh: vec![3..=5, 10..=14, 16..=20, 12..=18],
    available: vec![1, 5, 8, 11, 17, 32],
});

fn simplify_ranges(ranges: &mut Vec<RangeInclusive<usize>>) {
    if ranges.is_empty() {
        return;
    }

    ranges.sort_by_key(|r| *r.start());
    let mut extending = 0;
    for i in 0..ranges.len() {
        if ranges[i].start() > ranges[extending].end() {
            extending += 1;
            ranges[extending] = ranges[i].clone();
        } else {
            ranges[extending] =
                *ranges[extending].start()..=*max(ranges[extending].end(), ranges[i].end());
        }
    }
    ranges.truncate(extending + 1);
}

fn in_ranges(ranges: &Vec<RangeInclusive<usize>>, n: usize) -> bool {
    let p = ranges.partition_point(|r| *r.end() < n);
    ranges.get(p).is_some_and(|r| r.contains(&n))
}

fn part1(input: &Problem) -> usize {
    let mut input = input.clone();
    simplify_ranges(&mut input.fresh);
    input.available.iter().filter(|&&item| in_ranges(&input.fresh, item)).count()
}

fn checked_range_len(range: &RangeInclusive<usize>) -> usize {
    (range.end() - range.start()).checked_add(1).unwrap()
}

fn part2(input: &Problem) -> usize {
    let mut input = input.clone();
    simplify_ranges(&mut input.fresh);
    input.fresh.iter().map(checked_range_len).sum()
}

#[test]
fn test_part1() {
    assert_eq!(part1(&TEST_INPUT), 3);
}

#[test]
fn test_part2() {
    assert_eq!(part2(&TEST_INPUT), 14);
}

fn main() {
    println!("part 1: {}", part1(&input::INPUT));
    println!("part 2: {}", part2(&input::INPUT));
}
