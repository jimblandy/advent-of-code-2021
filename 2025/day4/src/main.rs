#![allow(unused_variables, dead_code)]

use std::borrow::Cow;

mod input;

#[derive(Clone)]
struct Problem<'a> {
    map: Cow<'a, [u8]>,
    width: isize,
    height: isize,
}

static SAMPLE_INPUT_MAP: &[u8] = b"\
..@@.@@@@.\
@@@.@.@.@@\
@@@@@.@.@@\
@.@@@@..@.\
@@.@@@@.@@\
.@@@@@@@.@\
.@.@.@.@@@\
@.@@@.@@@@\
.@@@@@@@@.\
@.@.@@@.@.\
";

static SAMPLE_INPUT: Problem = Problem {
    map: Cow::Borrowed(SAMPLE_INPUT_MAP),
    width: 10,
    height: 10,
};

impl Problem<'_> {
    fn get(&self, x: isize, y: isize) -> u8 {
        if 0 <= x && x < self.width && 0 <= y && y < self.height {
            *self.map.get((y * self.width + x) as usize).unwrap()
        } else {
            b'.'
        }
    }

    fn get_mut(&mut self, x: isize, y: isize) -> &mut u8 {
        if 0 <= x && x < self.width && 0 <= y && y < self.height {
            self.map.to_mut().get_mut((y * self.width + x) as usize).unwrap()
        } else {
            panic!("Problem coords out of range: {:?}", (x, y));
        }
    }

    fn coords(&self) -> impl Iterator<Item = (isize, isize)> + '_ {
        (0..self.width).flat_map(|x| (0..self.height).map(move |y| (x, y)))
    }

    fn rolls(&self) -> impl Iterator<Item = (isize, isize)> + '_ {
        self.coords().filter(|&(x, y)| self.get(x, y) == b'@')
    }

    fn movable(&self) -> impl Iterator<Item = (isize, isize)> + '_ {
        self.rolls()
            .filter(|&(x, y)| {
                offsets()
                    .filter(|&(dx, dy)| self.get(x + dx, y + dy) == b'@')
                    .count()
                    < 4
            })
    }
}

fn offsets() -> impl Iterator<Item = (isize, isize)> {
    const STEPS: [isize; 3] = [-1, 0, 1];
    STEPS
        .into_iter()
        .flat_map(|dx| STEPS.into_iter().map(move |dy| (dx, dy)))
        .filter(|&(dx, dy)| dx != 0 || dy != 0)
}

fn part1(problem: &Problem<'_>) -> usize {
    problem.movable().count()
}

#[test]
fn test_part1() {
    assert_eq!(part1(&SAMPLE_INPUT), 13);
    assert_eq!(part1(&input::INPUT), 1416);
}


fn remove(input: &Problem<'_>, output: &mut Problem<'_>) -> usize {
    let mut removed = 0;

    output.map.to_mut().copy_from_slice(&*input.map);
    for (x, y) in input.movable() {
        *output.get_mut(x, y) = b'.';
        removed += 1;
    }

    removed
}

fn part2(problem: &Problem<'_>) -> usize {
    let mut removed = 0;

    let mut temp1 = problem.clone();
    let mut temp2 = problem.clone();

    loop {
        let just_removed = remove(&temp1, &mut temp2);
        if just_removed == 0 {
            return removed
        }
        removed += just_removed;
        std::mem::swap(&mut temp1, &mut temp2);
    }
}

#[test]
fn test_part2() {
    assert_eq!(part2(&SAMPLE_INPUT), 43);
}


fn main() {
    println!("part 1: {}", part1(&input::INPUT));
    println!("part 2: {}", part2(&input::INPUT));
}
