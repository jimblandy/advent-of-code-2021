use aoc_runner_derive::{aoc, aoc_generator};
use std::iter;
use std::str::FromStr;
use std::num::ParseIntError;

/// The `i`'th element is the number of fish whose timer value is `i`.
type Cohorts = [usize; 9];

#[aoc_generator(day6)]
fn input_generator(input: &str) -> Result<Cohorts, ParseIntError> {
    let mut cohorts = Cohorts::default();

    input
        .split(',')
        .map(|s| usize::from_str(s))
        .try_for_each(|n| {
            cohorts[n?] += 1;
            Ok(())
        })?;

    Ok(cohorts)
}

fn step(now: &Cohorts) -> Cohorts {
    let mut next = Cohorts::default();
    next[0..8].copy_from_slice(&now[1..9]);
    next[8] = now[0];
    next[6] += now[0];

    next
}

fn pop(cohorts: Cohorts) -> usize {
    cohorts.iter().sum()
}

fn series(initial: Cohorts) -> impl Iterator<Item = Cohorts> + Clone {
    iter::successors(Some(initial), |c| Some(step(c)))
}

#[test]
fn test_step() {
    assert_eq!(step(&[1,2,3,4,5,6,7,8,9]),
               [2,3,4,5,6,7,9,9,1]);

    assert_eq!(step(&[0,1,1,2,1,0,0,0,0]), //initial
               [1,1,2,1,0,0,0,0,0]);
    assert_eq!(step(&[1,1,2,1,0,0,0,0,0]), //day1
               [1,2,1,0,0,0,1,0,1]);
    assert_eq!(step(&[1,2,1,0,0,0,1,0,1]), //day2
               [2,1,0,0,0,1,1,1,1]);

    let sample = series([0,1,1,2,1,0,0,0,0]);
    assert_eq!(sample
               .clone()
               .nth(18)
               .map(pop)
               .unwrap(),
               26);

    assert_eq!(sample
               .clone()
               .nth(80)
               .map(pop)
               .unwrap(),
               5934);
}

#[aoc(day6, part1)]
fn day6_part1(cohorts: &Cohorts) -> usize {
    series(*cohorts).nth(80).map(pop).unwrap()
}

#[aoc(day6, part2)]
fn day6_part2(cohorts: &Cohorts) -> usize {
    series(*cohorts).nth(256).map(pop).unwrap()
}

