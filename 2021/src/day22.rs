use aoc_runner_derive::{aoc, aoc_generator};
use anyhow::{anyhow, bail, Result};
use crate::linear;
use crate::linear::Point;
use std::{cmp, fmt};
use std::ops::Range;
use std::str::FromStr;
use hashbrown::HashSet;
use std::hash::Hash;

#[derive(Clone, Debug)]
struct Command {
    on: bool,
    cuboid: Range<Point>,
}

fn parse_range(input: &str) -> Result<Range<i64>> {
    let (left, right) = input.split_once("..")
        .ok_or_else(|| anyhow!("range missing '..': {:?}", input))?;
    let left = left.trim();
    let right = right.trim();

    Ok(i64::from_str(left)? .. i64::from_str(right)? + 1)
}

fn parse_cuboid(input: &str) -> Result<Range<Point>> {
    let mut cuboid = linear::ZEROP .. linear::ZEROP;
    for field in input.split(',') {
        let (axis, range) = field.split_once('=')
            .ok_or_else(|| anyhow!("cuboid field missing '=': {:?}", input))?;
        let range = parse_range(range)?;
        match axis {
            "x" => { cuboid.start.0 = range.start; cuboid.end.0 = range.end; }
            "y" => { cuboid.start.1 = range.start; cuboid.end.1 = range.end; }
            "z" => { cuboid.start.2 = range.start; cuboid.end.2 = range.end; }
            _ => bail!("weird axis {:?} in cuboid {:?}", axis, input),
        }
    }
    Ok(cuboid)
}

#[aoc_generator(day22, part1)]
#[aoc_generator(day22, part2)]
fn generator(input: &str) -> Result<Vec<Command>> {
    Ok(input.lines()
        .map(|line| {
            let (cmd, cuboid) = line.split_once(' ')
                .ok_or_else(|| anyhow!("missing space after command"))?;
            let on = match cmd {
                "on" => true,
                "off" => false,
                _ => bail!("unrecognized command: {:?}", cmd),
            };
            let cuboid = parse_cuboid(cuboid)?;
            Ok(Command { on, cuboid })
        })
        .collect::<Result<_>>()?)
}

#[cfg(test)]
fn sample() -> Vec<Command> {
    generator(include_str!("sample/day22")).unwrap()
}

#[test]
fn test_generator() {
    let s = sample();
    assert_eq!(s.len(), 22);
    assert!(s[0].on);
    assert_eq!(s[0].cuboid, Point(-20, -36, -47) .. Point(27, 18, 8));
}

#[derive(Debug)]
struct Event<T> {
    pos: i64,
    tag: T,
    kind: EventKind,
}

#[derive(Debug)]
enum EventKind {
    Start,
    Stop,
}

impl<T> cmp::PartialEq for Event<T> {
    fn eq(&self, other: &Self) -> bool {
        self.pos == other.pos
    }
}

impl<T> cmp::Eq for Event<T> { }

impl<T> cmp::Ord for Event<T> {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.pos.cmp(&other.pos)
    }
}

impl<T> cmp::PartialOrd for Event<T> {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

fn sum_runs<T, I, F>(ranges: I, mut body: F) -> i64
    where I: IntoIterator<Item = (T, Range<i64>)>,
          F: FnMut(Range<i64>, &HashSet<T>) -> i64,
          T: Clone + Hash + Eq + fmt::Debug,
{
    let mut events: Vec<Event<T>> = ranges.into_iter()
        .flat_map(|(tag, range)| {
            [Event { pos: range.start, tag: tag.clone(), kind: EventKind::Start },
             Event { pos: range.end, tag, kind: EventKind::Stop },
            ]
        })
        .collect();

    if events.is_empty() {
        return 0;
    }

    events.sort();

    let mut total = 0;
    let mut live = HashSet::new();
    let mut prev = events[0].pos;
    for event in events {
        if event.pos > prev {
            total += body(prev..event.pos, &live);
            prev = event.pos;
        }

        match event.kind {
            EventKind::Start => assert!(live.insert(event.tag)),
            EventKind::Stop => assert!(live.remove(&event.tag)),
        }
    }

    assert!(live.is_empty());

    total
}

fn all_ranges<'a>(commands: &'a [Command])
                  -> impl Iterator<Item = (usize, Range<i64>)> + 'a
{
    commands
        .iter()
        .enumerate()
        .map(|(i, c)| (i, c.cuboid.start.2 .. c.cuboid.end.2))
}

fn live_ranges<'a, A>(commands: &'a [Command], live: &'a HashSet<usize>, axis: A)
                   -> impl Iterator<Item = (usize, Range<i64>)> + 'a
    where A: Fn(&Point) -> i64 + 'static
{
    live
        .iter()
        .map(move |&i| {
            let c = &commands[i];
            (i, axis(&c.cuboid.start) .. axis(&c.cuboid.end))
        })
}

fn range_area(commands: &[Command], live: &HashSet<usize>, range: Range<i64>) -> i64 {
    // If the final command live in this range is on, then
    // include this in the count.
    live.iter().max().map(|&i| {
        if commands[i].on {
            range.end - range.start
        } else {
            0
        }
    }).unwrap_or(0)
}

fn volume(commands: &[Command]) -> i64 {
    sum_runs(all_ranges(commands), |range, plane_live| {
        (range.end - range.start) *
            sum_runs(live_ranges(commands, plane_live, |pt| pt.1), |range, row_live| {
                (range.end - range.start) *
                    sum_runs(live_ranges(commands, row_live, |pt| pt.0), |range, live| {
                        range_area(commands, live, range)
                    })
            })
    })
}

#[test]
fn test_volume() {
    assert_eq!(volume(&[Command { on: true, cuboid: Point(0,0,0) .. Point(1,1,1) }]),
               1);
    assert_eq!(volume(&[Command { on: true, cuboid: Point(0,0,0) .. Point(3,3,3) }]),
               27);
    assert_eq!(
        volume(&[
            Command { on: true, cuboid: Point(0,0,0) .. Point(1,1,1) },
            Command { on: false, cuboid: Point(0,0,0) .. Point(1,1,1) }
        ]),
        0);

    assert_eq!(
        volume(&[
            Command { on: true, cuboid: Point(0,0,0) .. Point(2,2,2) },
            Command { on: false, cuboid: Point(1,1,1) .. Point(3,3,3) }
        ]),
        7
    );

}

#[aoc(day22, part1)]
fn part1(input: &Vec<Command>) -> i64 {
    fn clip(i: i64) -> i64 {
        use cmp::{max, min};
        min(max(i, -50), 51)
    }

    let mut input = input.clone();
    for c in &mut input {
        let cuboid = &mut c.cuboid;
        cuboid.start.0 = clip(cuboid.start.0);
        cuboid.end.0   = clip(cuboid.end.0);
        cuboid.start.1 = clip(cuboid.start.1);
        cuboid.end.1   = clip(cuboid.end.1);
        cuboid.start.2 = clip(cuboid.start.2);
        cuboid.end.2   = clip(cuboid.end.2);
    }

    volume(&input)
}

#[test]
fn test_part1() {
    let s = sample();
    assert_eq!(part1(&s), 590784);
}

#[aoc(day22, part2)]
fn part2(input: &Vec<Command>) -> i64 {
    volume(&input)
}
