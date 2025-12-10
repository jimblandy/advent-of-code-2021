type Point = (u64, u64);

struct Problem {
    red: Vec<Point>,
}

impl Problem {
    fn from_str(input: &str) -> Self {
        Problem {
            red: input
                .lines()
                .map(|line| {
                    let mut coords = line
                        .split(',')
                        .map(|coord| coord.parse().unwrap());
                    (
                        coords.next().unwrap(),
                        coords.next().unwrap(),
                    )
                })
                .collect()
        }
    }
}

fn area(a: Point, b: Point) -> u64 {
    use std::cmp::{min, max};

    let ul = (min(a.0, b.0), min(a.1, b.1));
    let lr = (max(a.0, b.0), max(a.1, b.1));
    (lr.0 + 1 - ul.0) * (lr.1 + 1 - ul.1)
}

fn part1(problem: &Problem) -> u64 {
    problem.red
        .iter()
        .enumerate()
        .flat_map(|(i, &a)| problem.red[..i].iter().map(move |&b| (a, b)))
        .map(|(a, b)| area(a, b))
        .max()
        .unwrap()
}

fn main() {
    println!("part 1: {}", part1(&Problem::from_str(include_str!("input.txt"))));
}
