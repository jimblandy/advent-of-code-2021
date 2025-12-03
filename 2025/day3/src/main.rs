#![allow(unused_variables, dead_code)]

use std::{cmp, mem};

mod part1_input;

static TEST_INPUT: &[&[u64]] = &[
    &[9, 8, 7, 6, 5, 4, 3, 2, 1, 1, 1, 1, 1, 1, 1],
    &[8, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 9],
    &[2, 3, 4, 2, 3, 4, 2, 3, 4, 2, 3, 4, 2, 7, 8],
    &[8, 1, 8, 1, 8, 1, 9, 1, 1, 1, 1, 2, 1, 1, 1],
];

fn max_joltage(num_digits: usize, bank: &[u64]) -> u64 {
    // Grow our answer from right to left, starting with the empty
    // string of digits (whose value is zero) and adding digits at the
    // left end. That is, from the least significant digit to the most
    // significant.
    //
    // As we add digits to our answer, `best_rest[i]` is the largest
    // value of the given length that can be formed using the digits
    // in `bank[i + 1 ..]`. Since the length is initially zero digits,
    // `best_rest` begins initialized to zeros.
    let mut best_rest = vec![0; bank.len()];

    //eprintln!();
    for answer_length in 0..(num_digits - 1) {
        let place_value = 10_u64.checked_pow(answer_length as u32).unwrap();

        // `best_rest` now holds the largest numbers we can form of
        // length `answer_length`. Based on that, find the largest
        // numbers we can form by prepending one digit.
        //
        // Note that we're not allowed to reuse the final
        // `answer_length` digits in the bank.
        let usable = best_rest.len() - answer_length;
        let mut max = 0;
        for (best, &digit) in best_rest[..usable].iter_mut().zip(bank).rev() {
            let new_max = cmp::max(digit * place_value + *best, max);
            *best = mem::replace(&mut max, new_max);
        }
        //eprintln!("{best_rest:?}");
    }

    // Choose the best front digit.
    //
    // Again, note that we're not allowed to reuse the final
    // `num_digits - 1` digits in the bank.
    let place_value = 10_u64.checked_pow(num_digits as u32 - 1).unwrap();
    bank[..bank.len() - num_digits + 1]
        .iter()
        .zip(&best_rest)
        .map(|(&digit, &rest)| digit * place_value + rest)
        .max()
        .unwrap()
}

#[test]
fn test_part1() {
    assert_eq!(max_joltage(2, TEST_INPUT[0]), 98);
    assert_eq!(max_joltage(2, TEST_INPUT[1]), 89);
    assert_eq!(max_joltage(2, TEST_INPUT[2]), 78);
    assert_eq!(max_joltage(2, TEST_INPUT[3]), 92);
}

fn part1(input: &[&[u64]]) -> u64 {
    input.iter().cloned().map(|bank| max_joltage(2, bank)).sum()
}

#[test]
fn test_part2() {
    assert_eq!(max_joltage(12, TEST_INPUT[0]), 987654321111);
    assert_eq!(max_joltage(12, TEST_INPUT[1]), 811111111119);
    assert_eq!(max_joltage(12, TEST_INPUT[2]), 434234234278);
    assert_eq!(max_joltage(12, TEST_INPUT[3]), 888911112111);
}

fn part2(input: &[&[u64]]) -> u64 {
    input.iter().cloned().map(|bank| max_joltage(12, bank)).sum()
}

fn main() {
    eprintln!("part 1 test: {}", part1(TEST_INPUT));
    eprintln!("part 1: {}", part1(part1_input::INPUT));
    eprintln!();
    eprintln!("part 2 test: {}", part2(TEST_INPUT));
    eprintln!("part 2: {}", part2(part1_input::INPUT));
}
