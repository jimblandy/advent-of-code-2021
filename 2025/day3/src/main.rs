#![allow(unused_variables, dead_code)]

mod part1_input;

static PART1_TEST: &[&[u64]] = &[
    &[9, 8, 7, 6, 5, 4, 3, 2, 1, 1, 1, 1, 1, 1, 1],
    &[8, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 9],
    &[2, 3, 4, 2, 3, 4, 2, 3, 4, 2, 3, 4, 2, 7, 8],
    &[8, 1, 8, 1, 8, 1, 9, 1, 1, 1, 1, 2, 1, 1, 1],
];

fn max_joltage(digits: usize, bank: &[u64]) -> u64 {
    let mut max = 0;
    let mut best_after: Vec<u64> = bank
        .iter()
        .rev()
        .map(|&j| {
            let new_max = std::cmp::max(j, max);
            std::mem::replace(&mut max, new_max)
        })
        .collect();
    best_after.reverse();

    bank[..bank.len() - 1]
        .iter()
        .enumerate()
        .map(|(i, &tens)| tens * 10 + best_after[i])
        .max()
        .unwrap()
}

fn part1(input: &[&[u64]]) -> u64 {
    input.iter().cloned().map(|bank| max_joltage(2, bank)).sum()
}

fn main() {
    eprintln!("part 1 test: {}", part1(PART1_TEST));
    eprintln!("part 1: {}", part1(part1_input::INPUT));
}
