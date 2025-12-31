// [.##.] (3) (1,3) (2) (2,3) (0,2) (0,1) {3,5,4,7}
// [...#.] (0,2,3,4) (2,3) (0,4) (0,1,2) (1,2,3,4) {7,5,12,7,2}
// [.###.#] (0,1,2,3,4) (0,3,4) (0,1,2,4,5) (1,2) {10,11,11,5,10,5}

mod parse;
mod part2;

use std::str::FromStr as _;
use std::sync::LazyLock;

use aoc_utils::astar::{Edge, astar};
use aoc_utils::bfs::breadth_first;

#[derive(Debug)]
struct Problem {
    machines: Vec<Machine>,
}

#[derive(Debug)]
struct Machine {
    /// If light `i` should be on, then bit `1 << i` is set.
    lights: u64,

    /// A bitmap of the lights affected by each button.
    buttons: Vec<u64>,

    /// `buttons`, but sorted by decreasing number of lights affected.
    /// This is just used for estimates, so the button indices don't matter.
    buttons_by_size: Vec<u64>,

    /// The goal joltages for all the counters.
    joltages: Vec<u64>,
}

static SAMPLE_INPUT: LazyLock<Problem> = LazyLock::new(|| {
    Problem::from_str(
        r#"
[.##.] (3) (1,3) (2) (2,3) (0,2) (0,1) {3,5,4,7}
[...#.] (0,2,3,4) (2,3) (0,4) (0,1,2) (1,2,3,4) {7,5,12,7,2}
[.###.#] (0,1,2,3,4) (0,3,4) (0,1,2,4,5) (1,2) {10,11,11,5,10,5}
"#,
    )
    .unwrap()
});

static INPUT: LazyLock<Problem> =
    LazyLock::new(|| Problem::from_str(include_str!("input.txt")).unwrap());

impl Machine {
    fn part1(&self) -> usize {
        // Breadth-first search is not a great algorithm here, as it doesn't
        // understand that the order in which buttons are pressed makes no
        // difference, and will thus spend a bunch of time dithering around with
        // different orderings of button presses.
        //
        // For part 2 this problem is fatal, but for part 1 we can get away with
        // it, and it's a nice short solution.
        let mut search = breadth_first(0, |&state| {
            self.buttons.iter().map(move |&button| state ^ button)
        });
        let end = search.find(|&(_from, to, _path_length)| to == self.lights);
        let Some((_from, _to, path_length)) = end else {
            panic!("Initialized state unreachable using the machine's buttons?");
        };
        path_length
    }
}

fn part1(problem: &Problem) -> usize {
    problem.machines.iter().map(Machine::part1).sum()
}

fn part2(problem: &Problem) -> usize {
    problem
        .machines
        .iter()
        .inspect(|m| log::debug!("Part 2 machine: {m:#?}"))
        .map(Machine::part2)
        .sum()
}

fn main() -> anyhow::Result<()> {
    env_logger::init();
    aoc_utils::limit_memory();

    println!("part 1 example: {}", part1(&SAMPLE_INPUT));
    println!("part 1: {}", part1(&INPUT));
    println!("part 2 example: {}", part2(&SAMPLE_INPUT));
    println!("part 2: {}", part2(&INPUT));

    Ok(())
}
