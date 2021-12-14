#![allow(unused_imports, dead_code)]

use aoc_runner_derive::{aoc, aoc_generator};
use anyhow::{anyhow, bail, Result};
use std::fmt;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Elt(u8);

impl Elt {
    fn from_index(index: usize) -> Self {
        if index >= 26 {
            panic!("Element index out of range: {}", index);
        }
        Elt(index as u8)
    }
    fn index(self) -> usize { self.0 as usize }
    fn name(self) -> char { (self.0 + 65) as char }
    fn all() -> impl Iterator<Item = Elt> + Clone {
        (0..26).map(Elt::from_index)
    }
    fn all_pairs() -> impl Iterator<Item = (Elt, Elt)> {
        crate::cartesian_product(Elt::all(), Elt::all())
    }
}

impl TryFrom<char> for Elt {
    type Error = anyhow::Error;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            'A' ..= 'Z' => Ok(Elt((value as u32 - 'A' as u32) as u8)),
            _ => Err(anyhow!("bad element: {:?}", value))
        }
    }
}

impl fmt::Display for Elt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

type Rules = [[Option<Elt>; 26]; 26];
type Counts = [[usize; 26]; 26];

#[derive(Debug, PartialEq)]
struct Problem {
    template: Vec<Elt>,
    rules: Rules,
}

#[aoc_generator(day14)]
fn generate(input: &str) -> Result<Problem> {
    let mut lines = input.lines();

    let template = lines.next()
        .ok_or_else(|| anyhow!("input missing template line"))?
        .chars()
        .map(Elt::try_from)
        .collect::<Result<Vec<Elt>>>()?;

    match lines.next() {
        Some(line) if line.trim().is_empty() => {}
        _ => bail!("input missing blank line"),
    }

    let mut rules = [[None; 26]; 26];
    for line in lines {
        let (pair, new) = line.split_once("->")
            .ok_or_else(|| anyhow!("pair insertion rule line missing '->': {:?}", line))?;
        let pair: Vec<char> = pair.trim().chars().collect();
        if pair.len() != 2 {
            bail!("bad pair in insertion rule: {:?}", line);
        }
        let left = Elt::try_from(pair[0])?;
        let right = Elt::try_from(pair[1])?;

        let new: Vec<char> = new.trim().chars().collect();
        if new.len() != 1 {
            bail!("bad new element in insertion rule: {:?}", line);
        }
        let new = Elt::try_from(new[0])?;

        let entry = &mut rules[left.index()][right.index()];
        if entry.is_some() {
            bail!("multiple insertion rules for {}{}", left, right);
        }
        *entry = Some(new);
    }

    Ok(Problem { template, rules })
}

#[test]
fn test_generate() {
    let mut expected_rules = [[None; 26]; 26];
    let mut rule = |left, right, new| {
        let left = Elt::try_from(left).unwrap();
        let right = Elt::try_from(right).unwrap();
        let new = Elt::try_from(new).unwrap();
        expected_rules[left.index()][right.index()] = Some(new);
    };

    rule('C', 'H', 'B');
    rule('H', 'H', 'N');
    rule('C', 'B', 'H');
    rule('N', 'H', 'C');
    rule('H', 'B', 'C');
    rule('H', 'C', 'B');
    rule('H', 'N', 'C');
    rule('N', 'N', 'C');
    rule('B', 'H', 'H');
    rule('N', 'C', 'B');
    rule('N', 'B', 'B');
    rule('B', 'N', 'B');
    rule('B', 'B', 'N');
    rule('B', 'C', 'B');
    rule('C', 'C', 'N');
    rule('C', 'N', 'C');

    let expected_template: Vec<_> = "NNCB".chars().map(|ch| Elt::try_from(ch).unwrap()).collect();

    assert_eq!(generate(include_str!("sample/day14")).unwrap(),
               Problem {
                   template: expected_template,
                   rules: expected_rules,
               });
}

fn count_template(template: &[Elt]) -> Counts {
    let mut counts = [[0; 26]; 26];
    for window in template.windows(2) {
        counts[window[0].index()][window[1].index()] += 1;
    }
    counts
}

#[cfg(test)]
fn print_counts(counts: &Counts) {
    for (left, right) in Elt::all_pairs() {
        let prevalence = counts[left.index()][right.index()];
        if prevalence > 0 {
            println!("{}{} = {}", left, right, prevalence);
        }
    }
}

fn step(counts: &Counts, rules: &Rules) -> Counts {
    let mut next = [[0; 26]; 26];
    for (left, right) in Elt::all_pairs() {
        let prevalence = counts[left.index()][right.index()];
        if let Some(new) = rules[left.index()][right.index()] {
            next[left.index()][new.index()] += prevalence;
            next[new.index()][right.index()] += prevalence;
        } else {
            next[left.index()][right.index()] += prevalence;
        }
    }

    next
}

fn element_prevalence(counts: &Counts, original_template: &[Elt]) -> [usize; 26] {
    let mut prevalence = [0; 26];
    for i in 0..26 {
        for j in 0..26 {
            let pair_prevalence = counts[i][j];
            prevalence[i] += pair_prevalence;
            prevalence[j] += pair_prevalence;
        }
    };

    // Now every element has been double-counted, since it appears as the left
    // side of one pair and the right side of another - except for the template
    // ends.
    prevalence[original_template.first().unwrap().index()] += 1;
    prevalence[original_template.last().unwrap().index()] += 1;
    for i in 0..26 {
        assert_eq!(prevalence[i] & 1, 0);
        prevalence[i] /= 2;
    }

    prevalence
}

fn print_prevalence(prevalence: &[usize; 26]) {
    for elt in Elt::all() {
        if prevalence[elt.index()] > 0 {
            println!("{}: {}", elt, prevalence[elt.index()]);
        }
    }
}

fn most_and_least(counts: &Counts, original_template: &[Elt]) -> (usize, usize) {
    let prevalence = element_prevalence(counts, original_template);

    let non_zero = prevalence.iter().filter(|&&n| n > 0);
    let most = *non_zero.clone().max().unwrap();
    let least = *non_zero.min().unwrap();

    (most, least)
}

#[aoc(day14, part1, jimb)]
fn part1(input: &Problem) -> usize {
    let final_counts = (0..10).fold(count_template(&input.template), |counts, _| {
        step(&counts, &input.rules)
    });
    let (most, least) = most_and_least(&final_counts, &input.template);

    most - least
}

#[test]
fn test_part1() {
    let sample = generate(include_str!("sample/day14")).unwrap();
    let mut counts = count_template(&sample.template);

    for i in 1..=10 {
        counts = step(&counts, &sample.rules);
        println!("after step {}:", i);
        print_counts(&counts);
        print_prevalence(&element_prevalence(&counts, &sample.template));
    }

    println!("most and least: {:?}", most_and_least(&counts, &sample.template));

    assert_eq!(part1(&sample), 1588);
}

#[aoc(day14, part2, jimb)]
fn part2(input: &Problem) -> usize {
    let final_counts = (0..40).fold(count_template(&input.template), |counts, _| {
        step(&counts, &input.rules)
    });
    let (most, least) = most_and_least(&final_counts, &input.template);

    most - least
}

