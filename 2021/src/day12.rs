use aoc_runner_derive::{aoc, aoc_generator};
use anyhow::{anyhow, Result};
use std::collections::HashMap;

#[derive(Debug, Eq, PartialEq)]
struct Node {
    name: String,
    big: bool,
    out: Vec<usize>,
}

impl Node {
    fn new(name: &str) -> Node {
        Node {
            name: name.to_string(),
            big: name.starts_with(char::is_uppercase),
            out: vec![]
        }
    }
}

/// Indexed by node id.
type Graph = Vec<Node>;

const START: usize = 0;
const END: usize = 1;

#[aoc_generator(day12)]
fn generate(input: &str) -> Result<Graph> {
    let mut graph = vec![Node::new("start"), Node::new("end")];
    let mut numbers = HashMap::new();
    numbers.insert("start", 0);
    numbers.insert("end", 1);

    for line in input.lines() {
        let (start, end) = line.split_once("-")
            .ok_or_else(|| anyhow!("missing '-' separator: {:?}", line))?;

        // Not needed, for the actual inputs.
        let start = start.trim();
        let end = end.trim();

        let start_ix = *numbers.entry(start).or_insert_with(|| {
            graph.push(Node::new(start));
            graph.len() - 1
        });

        let end_ix = *numbers.entry(end).or_insert_with(|| {
            graph.push(Node::new(end));
            graph.len() - 1
        });

        graph[start_ix].out.push(end_ix);
        graph[end_ix].out.push(start_ix);
    }

    Ok(graph)
}

#[cfg(test)]
fn small_sample() -> Graph {
    generate(include_str!("sample/day12.small"))
        .expect("failed to parse day12.small")
}

#[cfg(test)]
fn bigger_sample() -> Graph {
    generate(include_str!("sample/day12.bigger"))
        .expect("failed to parse day12.bigger")
}

#[cfg(test)]
fn even_larger_sample() -> Graph {
    generate(include_str!("sample/day12.even-larger"))
        .expect("failed to parse day12.even-larger")
}

#[test]
fn test_generate() {
    assert_eq!(small_sample(),
               vec![
                   Node { name: "start".to_string(), big: false, out: vec![2, 3] },
                   Node { name: "end".to_string(), big: false, out: vec![2, 3] },
                   Node { name: "A".to_string(), big: true, out: vec![0, 4, 3, 1] },
                   Node { name: "b".to_string(), big: false, out: vec![0, 2, 5, 1] },
                   Node { name: "c".to_string(), big: false, out: vec![2] },
                   Node { name: "d".to_string(), big: false, out: vec![3] },
               ]);
}

fn count_from(graph: &Graph, start: usize, visited: u64) -> usize {
    let visited = visited | (1 << start);
    if start == END {
        return 1;
    }

    graph[start].out
        .iter()
        .filter(|&&out| graph[out].big || visited & (1 << out) == 0)
        .map(|&out| count_from(graph, out, visited))
        .sum()
}

#[aoc(day12, part1)]
fn part1(input: &Graph) -> usize {
    count_from(input, START, 0)
}

#[test]
fn test_part1() {
    assert_eq!(part1(&small_sample()), 10);
    assert_eq!(part1(&bigger_sample()), 19);
    assert_eq!(part1(&even_larger_sample()), 226);
}

fn count_from2(graph: &Graph, start: usize, twice: Option<usize>, visited: u64) -> usize {
    //println!("{:>depth$}{}", "", graph[start].name, depth = 4 * visited.count_ones() as usize);

    let visited = visited | (1 << start);
    if start == END {
        return 1;
    }

    let mut count = 0;
    for &out in &graph[start].out {
        let out_bit = 1 << out;
        if graph[out].big || visited & out_bit == 0 {
            count += count_from2(graph, out, twice, visited);
        } else if twice.is_none() && out != START {
            count += count_from2(graph, out, Some(out), visited);
        }
    }
    count
}

#[aoc(day12, part2)]
fn part2(input: &Graph) -> usize {
    count_from2(input, START, None, 0)
}

#[test]
fn test_part2() {
    assert_eq!(part2(&small_sample()), 36);
    assert_eq!(part2(&bigger_sample()), 103);
    assert_eq!(part2(&even_larger_sample()), 3509);
}
