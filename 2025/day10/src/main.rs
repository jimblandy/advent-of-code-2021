// [.##.] (3) (1,3) (2) (2,3) (0,2) (0,1) {3,5,4,7}
// [...#.] (0,2,3,4) (2,3) (0,4) (0,1,2) (1,2,3,4) {7,5,12,7,2}
// [.###.#] (0,1,2,3,4) (0,3,4) (0,1,2,4,5) (1,2) {10,11,11,5,10,5}

mod parse;

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
        let mut search = breadth_first(0, |&state| {
            self.buttons.iter().map(move |&button| state ^ button)
        });
        let end = search.find(|&(_from, to, _path_length)| to == self.lights);
        let Some((_from, _to, path_length)) = end else {
            panic!("Initialized state unreachable using the machine's buttons?");
        };
        path_length
    }

    /// Return a lower bound on the number of button presses required to get from
    /// `here` to the goal `self.joltages`.
    ///
    /// If it is impossible to reach `self.joltages` from `here` (there's no way
    /// to count down), return `None`.
    ///
    /// This estimate simply presses buttons greedily, preferring buttons that
    /// affect more counters, to get as close to the goal as possible without
    /// overshooting it.
    fn part2_remaining(&self, here: &[u64]) -> Option<usize> {
        let goal = &self.joltages;

        if here.iter().zip(goal).any(|(&here, &goal)| here > goal) {
            return None;
        }

        let mut greedy = here.to_owned();
        let mut presses = 0;
        for &button in &self.buttons_by_size {
            let shortest = greedy
                .iter()
                .zip(goal)
                .map(|(&here, &goal)| goal - here)
                .min()
                .unwrap();

            presses += shortest;
            press(button, shortest, &mut greedy);
        }

        Some(presses as usize)
    }

    fn part2_neighbors(&self, counters: Vec<u64>) -> impl Iterator<Item = (Vec<u64>, usize)> {
        self.buttons.iter().filter_map(move |&button| {
            let mut to = counters.clone();
            press(button, 1, &mut to);
            let remaining = self.part2_remaining(&to);
            remaining.map(|remaining| {
                log::debug!(
                    "distance from {to:?} to {:?} is at least {remaining}",
                    self.joltages
                );
                (to, remaining)
            })
        })
    }

    fn part2(&self) -> usize {
        let counters = vec![0; self.joltages.len()];
        let neighbors = |counters: &Vec<u64>| {
            let counters = counters.clone();
            self.part2_neighbors(counters.clone())
        };
        let mut search = astar(counters, neighbors);
        let end = search
            .inspect(|e| log::debug!("{e:?}"))
            .find(|&Edge { ref to, .. }| to == &self.joltages);
        let Some(Edge { path_length, .. }) = end else {
            panic!("Full-joltage state unreachable using the machine's buttons?");
        };
        path_length
    }
}

fn part1(problem: &Problem) -> usize {
    problem.machines.iter().map(Machine::part1).sum()
}

/// Return the effect on `counters` of pressing `button` `n` times.
///
/// Here, `button` is a bitmask of the counters affected.
fn press(button: u64, n: u64, counters: &mut [u64]) {
    for (i, counter) in counters.iter_mut().enumerate() {
        let increment = if button & (1 << i) != 0 { n } else { 0 };
        *counter += increment;
    }
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
