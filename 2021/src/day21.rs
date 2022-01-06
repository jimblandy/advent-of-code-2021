use aoc_runner_derive::{aoc, aoc_generator};
use anyhow::{anyhow, Result};
use hashbrown::HashMap;
use hashbrown::hash_map::Entry;
use std::cmp;
use std::collections::BinaryHeap;

type Pos = u8;

#[aoc_generator(day21)]
fn generator(input: &str) -> Result<(Pos, Pos)> {
    let (line1, line2) = input.split_once('\n')
        .ok_or_else(|| anyhow!("Input has only one line: {:?}", input))?;
    let player1 = line1.split_once(": ")
        .ok_or_else(|| anyhow!("Player 1 line missing colon: {:?}", input))?
        .1
        .trim()
        .parse()?;
    let player2 = line2.split_once(": ")
        .ok_or_else(|| anyhow!("Player 2 line missing colon: {:?}", input))?
        .1
        .trim()
        .parse()?;
    Ok((player1, player2))
}

#[cfg(test)]
fn sample() -> (Pos, Pos) {
    generator("\
Player 1 starting position: 4
Player 2 starting position: 8
")
        .expect("failed to parse sample input")
}

#[aoc(day21, part1)]
fn part1(&(mut pos1, mut pos2): &(Pos, Pos)) -> u32 {
    let mut num_rolls = 0;

    let mut score1 = 0;
    let mut score2 = 0;

    fn roll(num_rolls: &mut usize) -> u32 {
        let roll = (*num_rolls % 100) as u32 + 1;
        *num_rolls += 1;
        roll
    }

    fn turn(pos: &mut Pos, score: &mut u32, die: &mut usize) -> bool {
        let sum = roll(die) + roll(die) + roll(die);
        *pos += (sum % 10) as u8;
        if *pos > 10 {
            *pos -= 10;
        }

        *score += *pos as u32;
        *score >= 1000
    }


    loop {
        if turn(&mut pos1, &mut score1, &mut num_rolls) ||
           turn(&mut pos2, &mut score2, &mut num_rolls) {
            break;
        }
    }

    num_rolls as u32 * cmp::min(score1, score2)
}

#[test]
fn test_part1() {
    assert_eq!(part1(&sample()), 739785);
}

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
struct State {
    player: u8,
    pos: [Pos; 2],
    score: [u8; 2],
}

/// Enqueued states are ordered by the sum of the two players' scores,
/// where *lower* scores precede *higher* scores.
struct Enqueued(State);

#[aoc(day21, part2)]
fn part2(&(pos1, pos2): &(Pos, Pos)) -> u64 {
    let initial = State {
        player: 0,
        pos: [pos1, pos2],
        score: [0, 0],
    };

    // For each reachable state, the number of ways we have discovered so far
    // that it can be reached from the initial state.
    let mut reachable_ways = HashMap::new();

    // Work queue of states whose successors we must consider. Insert a state
    // only the first time we reach it, to ensure we will visit each state
    // exactly once.
    //
    // The ordering on `Enqueued` ensures that we visit a state only after we
    // have visited all its possible predecessors, to ensure that by the time we
    // need to generate its successors, we have a complete count of how many
    // ways the state itself can be reached.
    let mut work = BinaryHeap::new();

    reachable_ways.insert(initial.clone(), 1_u64);
    work.push(Enqueued(initial));

    let mut wins = [0, 0];
    while let Some(enqueued) = work.pop() {
        let state = enqueued.0;

        // We will never this state again, so we can remove it from the hash
        // table, to reduce its size.
        let ways = reachable_ways.remove(&state).unwrap();

        // If this state is a winning state, add it to the appropriate player's
        // count.
        if let Some(winner) = state.is_winning() {
            wins[winner] += ways;

            // Don't consider this state's successors. This is how the queue is
            // able to eventually empty.
            continue;
        }

        // Consider the successor to `state` reached by rolling `sum`, which can
        // occur in `weight` different ways rolling a three-sided die three
        // times.
        let mut consider = |sum: u8, weight: u64| {
            let ways = ways * weight;

            let player = state.player as usize;
            let mut next_pos = state.pos[player] + sum;
            if next_pos > 10 {
                next_pos -= 10;
            }

            let mut next_state = state.clone();
            next_state.pos[player] = next_pos;
            next_state.score[player] += next_pos;
            next_state.player = player as u8 ^ 1;

            match reachable_ways.entry(next_state.clone()) {
                Entry::Occupied(mut entry) => {
                    // We have discovered more paths to `next_state`.
                    *entry.get_mut() += ways;
                }
                Entry::Vacant(entry) => {
                    // This is the first time we've found paths to `next_state`.
                    // Set it up with the number of ways it can be reached from
                    // the start state, and enqueue it to be considered once all
                    // its possible predecessors have been considered.
                    entry.insert(ways);
                    work.push(Enqueued(next_state));
                }
            }
        };

        // Consider all the successors of `state`: the sum of the three rolls,
        // and the number of ways that sum could be produced.
        consider(3, 1);
        consider(4, 3);
        consider(5, 6);
        consider(6, 7);
        consider(7, 6);
        consider(8, 3);
        consider(9, 1);
    }

    cmp::max(wins[0], wins[1])
}

#[test]
fn test_part2() {
    assert_eq!(part2(&sample()), 444356092776315);
}

impl State {
    fn is_winning(&self) -> Option<usize> {
        self.score.iter().position(|&s| s >= 21)
    }
}

impl Enqueued {
    fn sum(&self) -> u8 {
        let score = &self.0.score;
        score[0] + score[1]
    }
}

impl cmp::PartialEq for Enqueued {
    fn eq(&self, other: &Self) -> bool {
        self.sum() == other.sum()
    }
}

impl cmp::Eq for Enqueued { }

impl cmp::Ord for Enqueued {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.sum().cmp(&other.sum()).reverse()
    }
}

impl cmp::PartialOrd for Enqueued {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}
