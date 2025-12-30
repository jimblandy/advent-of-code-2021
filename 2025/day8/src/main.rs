#![allow(dead_code, unused_variables)]

mod distances;
mod sets;

use distances::Distances;
use sets::Sets;

use std::str::FromStr as _;
use std::time::Instant;

type Point = (u64, u64, u64);

struct Problem {
    boxes: Vec<Point>,
}

impl Problem {
    fn parse(input: &str) -> Problem {
        let boxes = input
            .lines()
            .map(|line| {
                let mut iter = line.split(',').map(|coord| u64::from_str(coord).unwrap());
                (
                    iter.next().unwrap(),
                    iter.next().unwrap(),
                    iter.next().unwrap(),
                )
            })
            .collect();

        Problem { boxes }
    }
}

fn part1(problem: &Problem, num_connections: usize) -> usize {
    let distances = Distances::from_points(&problem.boxes);
    let edges_by_length = distances.edges_by_length();
    let mut circuits = Sets::new(problem.boxes.len());
    let start = Instant::now();
    for edge in &edges_by_length[..num_connections] {
        circuits.join(edge.from(), edge.to());
    }
    let mut clumps = circuits.sets();
    clumps.sort_by(|&a, &b| circuits.size(a).cmp(&circuits.size(b)).reverse());
    let end = Instant::now();
    eprintln!("time elapsed: {:?}", end - start);
    clumps
        .iter()
        .take(3)
        .map(|&clump| circuits.size(clump))
        .product()
}

#[test]
fn test_part1() {
    assert_eq!(
        part1(&Problem::parse(include_str!("test_input.txt")), 10),
        40
    );
}

fn part2(problem: &Problem) -> u64 {
    let start = Instant::now();
    let distances = Distances::from_points(&problem.boxes);
    let edges_by_length = distances.edges_by_length();
    let end = Instant::now();
    eprintln!("time to get edges by length: {:?}", end - start);
    let mut circuits = Sets::new(problem.boxes.len());
    let start = Instant::now();
    for edge in &edges_by_length {
        if circuits.join(edge.from(), edge.to()) == problem.boxes.len() {
            let end = Instant::now();
            eprintln!("time elapsed: {:?}", end - start);
            return problem.boxes[edge.from()].0 * problem.boxes[edge.to()].0;
        }
    }
    panic!("never connected everybody");
}

#[test]
fn test_part2() {
    assert_eq!(
        part2(&Problem::parse(include_str!("test_input.txt"))),
        25272
    );
}

fn main() {
    println!(
        "part 1: {}",
        part1(&Problem::parse(include_str!("input.txt")), 1000)
    );
    println!(
        "part 2: {}",
        part2(&Problem::parse(include_str!("input.txt")))
    );
}
