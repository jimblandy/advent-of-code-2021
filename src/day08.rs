#![allow(dead_code, unused_imports)]
use anyhow::{anyhow, bail, Context, Error, Result};
use aoc_runner_derive::{aoc, aoc_generator};
use itertools::Itertools;
use std::str::FromStr;
use std::num::ParseIntError;

type Pattern = [bool; 7];

struct Entry {
    scrambled: [Pattern; 10],
    out_digits: [Pattern; 4],
}

fn population(pattern: &Pattern) -> usize {
    pattern
        .iter()
        .filter(|&&bit| bit)
        .count()
}

fn parse_pattern(input: &str) -> Result<Pattern> {
    let mut pat = Pattern::default();

    for ch in input.chars() {
        match ch {
            'a'..='g' => pat[ch as usize - 'a' as usize] = true,
            _ => bail!("non-segment letter in pattern: {:?}", ch),
        }
    }

    Ok(pat)
}

fn parse_patterns(input: &str) -> Result<Vec<Pattern>> {
    input
        .split_whitespace()
        .map(parse_pattern)
        .collect()
}

#[aoc_generator(day8)]
fn part1_input(input: &str) -> Result<Vec<Entry>> {
    input
        .lines()
        .map(|line| -> Result<Entry> {
            let (patterns, rest) = line.split_once('|')
                .ok_or(anyhow!("Missing '|' delimiter"))?;
            let scrambled = parse_patterns(patterns)?;
            let out_digits = parse_patterns(rest)?;
            Ok(Entry {
                scrambled: scrambled.try_into().map_err(|_| anyhow!("scrambled is wrong length"))?,
                out_digits: out_digits.try_into().map_err(|_| anyhow!("out_digits is wrong length"))?,
            })
        })
        .collect()
}

#[aoc(day8, part1)]
fn part1(input: &Vec<Entry>) -> usize {
    input
        .iter()
        .flat_map(|entry| &entry.out_digits)
        .filter(|pat| [2_usize, 4, 3, 7].contains(&population(pat)))
        .count()
}

fn sample2() -> Vec<Entry> {
    part1_input("\
be cfbegad cbdgef fgaecd cgeb fdcge agebfd fecdb fabcd edb | fdgacbe cefdb cefbgd gcbe
edbfga begcd cbg gc gcadebf fbgde acbgfd abcde gfcbed gfec | fcgedb cgb dgebacf gc
fgaebd cg bdaec gdafb agbcfd gdcbef bgcad gfac gcb cdgabef | cg cg fdcagb cbg
fbegcd cbd adcefb dageb afcb bc aefdc ecdab fgdeca fcdbega | efabcd cedba gadfec cb
aecbfdg fbg gf bafeg dbefa fcge gcbea fcaegb dgceab fcbdga | gecf egdcabf bgf bfgea
fgeab ca afcebg bdacfeg cfaedg gcfdb baec bfadeg bafgc acf | gebdcfa ecba ca fadegcb
dbcfg fgd bdegcaf fgec aegbdf ecdfab fbedc dacgb gdcebf gf | cefg dcbef fcge gbcadfe
bdfegc cbegaf gecbf dfcage bdacg ed bedf ced adcbefg gebcd | ed bcgafe cdgba cbgef
egadfb cdbfeg cegd fecab cgb gbdefca cg fgcdab egfdb bfceg | gbdfcae bgc cg cgb
gcafb gcf dcaebfg ecagb gf abcdeg gaef cafbge fdbac fegbdc | fgae cfgab fg bagce
").expect("failed to parse input")
}

#[test]
fn test_part1() {
    assert_eq!(part1(&sample1()), 0);
    assert_eq!(part1(&sample2()), 26);
}

static DIGITS: [Pattern; 10] = [
    //  a      b     c     d       e      f      g
    [  true,  true,  true, false,  true,  true,  true ], // 0
    [ false, false,  true, false, false,  true, false ], // 1
    [  true, false,  true,  true,  true, false,  true ], // 2
    [  true, false,  true,  true, false,  true,  true ], // 3
    [ false,  true,  true,  true, false,  true, false ], // 4
    [  true,  true, false,  true, false,  true,  true ], // 5
    [  true,  true, false,  true,  true,  true,  true ], // 6
    [  true, false,  true, false, false,  true, false ], // 7
    [  true,  true,  true,  true,  true,  true,  true ], // 8
    [  true,  true,  true,  true, false,  true,  true ], // 9
];

/// For each segment, the set of segments it could possibly be.
type Permitted = [Pattern; 7];

// f 9
// a 8
// c 8
// d 7
// g 7
// b 6
// e 4

fn sample1() -> Vec<Entry> {
    part1_input("acedgfb cdfbe gcdfa fbcad dab cefabd cdfgeb eafb cagedb ab | cdfeb fcadb cdfeb cdbaf")
        .expect("failed to parse input")
}

type Permutation = [usize; 7];

fn solve_mapping(scrambled: &[Pattern]) -> Result<Permutation> {
    for permutation in (0..7).permutations(7) {
        let permutation: Permutation = permutation.try_into().unwrap();
        if scrambled.iter().all(|digit| {
            DIGITS.contains(&([0,1,2,3,4,5,6].map(|i| digit[permutation[i]])))
        }) {
            return Ok(permutation);
        }
    }

    bail!("Not a valid set of scrambled digits");
}

#[test]
fn test_solve_mapping() -> Result<()> {
    assert_eq!(solve_mapping(&sample1()[0].scrambled)?,
               [3,4,0,5,6,1,2]);

    Ok(())
}

fn fix_digit(scrambled: &Pattern, perm: &Permutation) -> usize {
    DIGITS.iter().position(|digit| digit == &([0,1,2,3,4,5,6].map(|i| scrambled[perm[i]])))
        .expect("didn't find unscrambled digit")
}

fn decode_digits(digits: &[Pattern], perm: &Permutation) -> usize {
    digits
        .iter()
        .fold(0, |a, digit| a * 10 + fix_digit(digit, perm))
}

fn solve_entry(entry: &Entry) -> usize {
    let perm = solve_mapping(&entry.scrambled)
        .expect("failed to solve mapping");
    decode_digits(&entry.out_digits, &perm)
}

#[test]
fn test_decode_digits() {
    let sample1 = &sample1()[0];
    assert_eq!(solve_entry(&sample1), 5353);

    assert_eq!(sample2()
               .iter()
               .map(solve_entry)
               .collect::<Vec<_>>(),
               vec![
                   8394,
                   9781,
                   1197,
                   9361,
                   4873,
                   8418,
                   4548,
                   1625,
                   8717,
                   4315,
               ]);
}

#[aoc(day8, part2)]
fn part2(input: &Vec<Entry>) -> usize {
    input
        .iter()
        .map(solve_entry)
        .sum()
}

#[test]
fn test_part2() {
    assert_eq!(part2(&sample2()), 61229);
}
