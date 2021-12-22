use aoc_runner_derive::{aoc, aoc_generator};
use anyhow::Result;
use std::ops::RangeInclusive;

#[aoc_generator(day17, part1, jimb)]
#[aoc_generator(day17, part2, jimb)]
fn generator(_input: &str) -> Target {
    Target {
        x: 179..=201, 
        y: -109..=-63
    }
}

struct Target {
    x: RangeInclusive<i32>,
    y: RangeInclusive<i32>
}

#[aoc(day17, part1, jimb)]
fn part1(input: &Target) -> i32 {
    for initial_vx in suitable_initial_vx(input.x.clone()) {
        for initial_vy in (0 ..= -*input.y.start()).rev() {
            let mut max_y = 0;
            if positions(initial_vx, initial_vy)
                .inspect(|pos| {
                    if pos.1 > max_y {
                        max_y = pos.1;
                    }
                })
                .take_while(|&(x, y)| x <= *input.x.end() && y >= *input.y.start())
                .any(|(x, y)| input.x.contains(&x) && input.y.contains(&y))
            {
                return max_y;
            }
        }
    }
    panic!("nothing worked");
}

fn suitable_initial_vx(range: RangeInclusive<i32>) -> impl Iterator<Item = i32> {
    (0 ..= *range.end())
        .filter(move |&initial_x| {
            x_positions(initial_x).any(|pos| range.contains(&pos))
        })
}

fn x_positions(initial_vx: i32) -> impl Iterator<Item = i32> {
    let mut pos = 0;
    (0 ..= initial_vx)
        .rev()
        .map(move |vx| { pos += vx; pos })
}

fn positions(mut vx: i32, mut vy: i32) -> impl Iterator<Item = (i32, i32)> {
    let (mut x, mut y) = (0, 0);
    std::iter::repeat_with(move || {
        x += vx;
        y += vy;
        vx -= i32::signum(vx);
        vy -= 1;

        (x, y)
    })
}

#[test]
fn test_part1() {
    assert_eq!(part1(&Target { x: 20..=30, y: -10 ..= -5 }),
               45);
}

#[aoc(day17, part2, jimb)]
fn part2(input: &Target) -> usize {
    suitable_initial_vx(input.x.clone())
        .flat_map(|initial_vx| {
            (*input.y.start() ..= -*input.y.start())
                .rev()
                .map(move |init_y| (initial_vx, init_y))
        })
        .filter(|&(vx, vy)| {
            positions(vx, vy)
                .take_while(|&(x, y)| x <= *input.x.end() && y >= *input.y.start())
                .any(|(x, y)| input.x.contains(&x) && input.y.contains(&y))
        })
        .count()
}

#[test]
fn test_part2() {
    assert_eq!(part2(&Target { x: 20..=30, y: -10 ..= -5 }),
               112);
}
