#![allow(unused_variables, dead_code)]

mod input;

struct Problem<'a> {
    map: &'a [u8],
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
    map: SAMPLE_INPUT_MAP,
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
}

fn part1(problem: &Problem<'_>) -> usize {
    const STEPS: [isize; 3] = [-1, 0, 1];
    let offsets = STEPS
        .into_iter()
        .flat_map(|dx| STEPS.into_iter().map(move |dy| (dx, dy)))
        .filter(|&(dx, dy)| dx != 0 || dy != 0);

    let coords = (0..problem.width).flat_map(|x| (0..problem.height).map(move |y| (x, y)));

    coords
        .filter(|&(x, y)| problem.get(x, y) == b'@')
        .filter(|&(x, y)| {
            offsets
                .clone()
                .filter(|&(dx, dy)| problem.get(x + dx, y + dy) == b'@')
                .count()
                < 4
        })
        .count()
}

#[test]
fn test_part1() {
    assert_eq!(part1(&SAMPLE_INPUT), 13);
}

fn main() {
    println!("part 1: {}", part1(&input::INPUT));
}
