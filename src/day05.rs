use log;
use thiserror::Error;
use aoc_runner_derive::{aoc, aoc_generator};
use std::cmp;
use std::collections::BTreeSet;
use std::collections::HashSet;
use std::num::ParseIntError;
use std::ops::Range;
use std::str::FromStr;

#[derive(Clone, Debug, Eq, PartialEq)]
struct Point(i32, i32);

#[derive(Clone, Debug, Eq, PartialEq)]
struct Line {
    start: Point,
    end: Point,
}

#[derive(Error, Debug)]
pub enum ParsePairError<I> {
    #[error("Couldn't parse {ty}: missing {sep:?} separator in {input:?}")]
    MissingSeparator { ty: String, sep: String, input: String },
    #[error("Couldn't parse {ty}: multiple {sep:?} separators in {input:?}")]
    ExtraSeparator { ty: String, sep: String, input: String },
    #[error(transparent)]
    Inner(#[from] I),
}

fn parse_pair<T>(whole: &str, input: &str, sep: &str) -> Result<(T, T), ParsePairError<T::Err>>
    where T: FromStr,
{
    let mut iter = input.split(sep);
    let first = iter.next().unwrap();
    let second = iter.next().ok_or_else(|| {
        ParsePairError::MissingSeparator {
            ty: whole.to_owned(),
            sep: sep.to_owned(),
            input: input.to_owned(),
        }
    })?;

    if iter.next().is_some() {
        return Err(ParsePairError::ExtraSeparator {
            ty: whole.to_owned(),
            sep: sep.to_owned(),
            input: input.to_owned(),
        });
    }

    let first = T::from_str(first.trim())?;
    let second = T::from_str(second.trim())?;

    Ok((first, second))
}

impl FromStr for Point {
    type Err = ParsePairError<ParseIntError>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse_pair::<i32>("Point", s, ",")
            .map(|(x, y)| Point(x, y))
    }
}

#[test]
fn parse_point() {
    assert!(Point::from_str("").is_err());
    assert!(Point::from_str("1000, 2000,").is_err());
    assert!(Point::from_str("1000,").is_err());
    assert!(Point::from_str("1000").is_err());
    assert_eq!(Point::from_str("1000, 2000").unwrap(), Point(1000, 2000));
}

type ParseLineError = ParsePairError<<Point as FromStr>::Err>;
impl FromStr for Line {
    type Err = ParseLineError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse_pair::<Point>("Line", s, "->")
            .map(|(start, end)| Line { start, end })
    }
}

#[test]
fn parse_line() {
    assert!(matches!(Line::from_str(""),
                     Err(ParsePairError::MissingSeparator { .. })));
    assert!(matches!(Line::from_str("x"),
                     Err(ParsePairError::MissingSeparator { .. })));
    assert!(matches!(Line::from_str("x -> y -> z"),
                     Err(ParsePairError::ExtraSeparator { .. })));
    assert!(matches!(Line::from_str("x -> y "),
                     Err(ParsePairError::Inner(_))));
    assert!(matches!(Line::from_str("0, 1 -> 20, 300"),
                     Ok(Line { start: Point(0,1), end: Point(20, 300) })));
}

#[aoc_generator(day5)]
fn input_generator(input: &str) -> Result<Vec<Line>, ParseLineError> {
    input
        .lines()
        .map(Line::from_str)
        .collect()
}

#[derive(Clone, Debug)]
struct Action {
    pos: i32,
    kind: ActionKind,
    index: usize,
}

impl Action {
    fn start(pos: i32, index: usize) -> Self {
        Action {
            pos,
            kind: ActionKind::Start,
            index
        }
    }

    fn end(pos: i32, index: usize) -> Self {
        Action {
            pos,
            kind: ActionKind::End,
            index
        }
    }
}

impl cmp::PartialEq for Action {
    fn eq(&self, other: &Self) -> bool {
        self.pos == other.pos
    }
}

impl cmp::Eq for Action { }

impl cmp::PartialOrd for Action {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl cmp::Ord for Action {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.pos.cmp(&other.pos)
    }
}

#[derive(Clone, Debug)]
enum ActionKind {
    Start,
    End,
}

struct Rect {
    h: Range<i32>,
    v: Range<i32>
}

fn sparse_horiz_vert(input: &Vec<Line>) -> i32 {
    let rects: Vec<Rect> = input.iter()
        .map(|line| Rect {
            h: cmp::min(line.start.0, line.end.0) .. cmp::max(line.start.0, line.end.0),
            v: cmp::min(line.start.1, line.end.1) .. cmp::max(line.start.1, line.end.1),
        })
        .collect();

    let mut v_actions = vec![];
    for (i, rect) in rects.iter().enumerate() {
        v_actions.push(Action::start(rect.v.start, i));
        v_actions.push(Action::end(rect.v.end + 1, i));
    }
    v_actions.sort();
    log::trace!("v_actions: {:#?}", v_actions);

    // A sorted list of actions describing the current horizontal status.
    // We defer sorting this until we actually advance vertically.
    let mut live_h: Vec<Action> = vec![];

    // Defer removals from live_h until we actually advance vertically, so that
    // we can handle many removals with a single `Vec::retain` call.
    let mut rects_to_remove = HashSet::new();

    let mut area = 0;

    let mut last_v = 0;
    for v_action in v_actions {
        let v_dist = v_action.pos - last_v;
        if v_dist > 0 {
            log::trace!("V: advancing to {}", v_action.pos);

            // Do deferred removals.
            live_h.retain(|action| {
                !rects_to_remove.contains(&action.index)
            });

            // Do deferred sort.
            live_h.sort();
            rects_to_remove.clear();

            log::trace!("  live_h: {:#?}", live_h);

            let mut h_area = 0;
            let mut last_h = 0;
            let mut count = 0;
            for h_action in &live_h {
                let h_dist = h_action.pos - last_h;
                if count > 1 {
                    h_area += h_dist;
                }
                last_h = h_action.pos;
                log::trace!("    count: {}  h_area: {}", count, h_area);

                log::trace!("    action: {:?}", h_action);
                match h_action.kind {
                    ActionKind::Start => count += 1,
                    ActionKind::End => count -= 1,
                }
            }
            log::trace!("    h_area: {}", h_area);

            area += v_dist * h_area;
            log::trace!("  area: {}", area);
        }

        last_v = v_action.pos;

        match v_action.kind {
            ActionKind::Start => {
                let range = &rects[v_action.index].h;
                live_h.push(Action::start(range.start, v_action.index));
                live_h.push(Action::end(range.end + 1, v_action.index));
            }
            ActionKind::End => {
                assert!(rects_to_remove.insert(v_action.index));
            }
        }

    }

    area
}

#[aoc(day5, part1)]
fn part1_sparse(lines: &Vec<Line>) -> i32 {
    let lines: Vec<_> = lines.iter()
        // Horizontal or vertical lines only
        .filter(|line| line.start.0 == line.end.0 || line.start.1 == line.end.1)
        .cloned()
        .collect();

    sparse_horiz_vert(&lines)
}

#[cfg(test)]
fn sample_input() -> Vec<Line> {
    let input = "\
    0,9 -> 5,9
8,0 -> 0,8
9,4 -> 3,4
2,2 -> 2,1
7,0 -> 7,4
6,4 -> 2,0
0,9 -> 2,9
3,4 -> 1,4
0,0 -> 8,8
5,5 -> 8,2
";

    input_generator(input)
        .expect("failed to parse input")
}

#[cfg(test)]
fn init_logging() {
    let _ = env_logger::builder().is_test(true).try_init();
}

#[test]
fn test_day5_part1() {
    init_logging();

    assert_eq!(sparse_horiz_vert(&vec![                          // a   a   a   .   .
        Line { start: Point(0,0), end: Point(2,2) },  // a   ab  ab  b   .
        Line { start: Point(1,1), end: Point(3,3) },  // a   ab  abc bc  c
        Line { start: Point(2,2), end: Point(4,4) },  // .   b   bc  bc  c
    ]), 7);                                           // .   .   c   c   c

    assert_eq!(sparse_horiz_vert(&vec![
        Line { start: Point(0,0), end: Point(1,1) },
        Line { start: Point(1,1), end: Point(2,2) },
    ]), 1);

    assert_eq!(sparse_horiz_vert(&vec![
        Line { start: Point(0,0), end: Point(10,0) },
        Line { start: Point(1,0), end: Point(6,0) },
        Line { start: Point(5,0), end: Point(9,0) },
    ]), 9);

    assert_eq!(sparse_horiz_vert(&vec![
        Line { start: Point(0,0), end: Point(4,0) },
        Line { start: Point(5,0), end: Point(10,0) },
    ]), 0);

    assert_eq!(sparse_horiz_vert(&vec![
        Line { start: Point(0,0), end: Point(4,0) },
        Line { start: Point(2,0), end: Point(6,0) },
    ]), 3);

    assert_eq!(sparse_horiz_vert(&vec![
        Line { start: Point(0,0), end: Point(4,0) }
    ]), 0);
}

#[test]
fn test_day5_part1_sample() {
    init_logging();

    let lines = sample_input();
    let lines: Vec<_> = lines.into_iter()
        // Horizontal or vertical lines only
        .filter(|line| line.start.0 == line.end.0 || line.start.1 == line.end.1)
        .collect();

    assert_eq!(sparse_horiz_vert(&lines), 5);
}

fn sparse_all_lines(input: &Vec<Line>) -> i32 {
    let mut lines = input.clone();

    // Arrange all lines such that start is always the top.
    for line in &mut lines {
        if line.start.1 > line.end.1 {
            std::mem::swap(&mut line.start, &mut line.end);
        }
        // All lines are either horizontal, vertical, or 45° diagonal.
        assert!(line.start.1 == line.end.1 ||
                line.start.0 == line.end.0 ||
                i32::abs(line.start.0 - line.end.0) == line.end.1 - line.start.1);
    }

    let mut v_actions = vec![];
    for (i, line) in lines.iter().enumerate() {
        v_actions.push(Action::start(line.start.1, i));
        v_actions.push(Action::end(line.end.1 + 1, i));
    }
    v_actions.sort();
    v_actions.reverse();
    log::trace!("v_actions: {:#?}", v_actions);

    // note: actions are reversed
    let top = v_actions.last().unwrap().pos;
    let bottom = v_actions.first().unwrap().pos;

    // The current set of live lines.
    let mut live: BTreeSet<usize> = BTreeSet::new();

    // The horizontal state of affairs will, in general, change at every line,
    // so we might as well just compute it from scratch. But let's retain the
    // heap storage from one iteration to the next.
    let mut h_actions = vec![];

    let mut area = 0;
    for v in top..bottom {
        // Apply actions scheduled at this vertical position.
        while let Some(v_action) = v_actions.last() {
            assert!(v_action.pos >= v);
            if v_action.pos != v {
                break;
            }

            match v_action.kind {
                ActionKind::Start => assert!(live.insert(v_action.index)),
                ActionKind::End => assert!(live.remove(&v_action.index)),
            }
            v_actions.pop();
        }

        // Build a list of start and end actions for this line.
        h_actions.clear();
        for &index in &live {
            let line = &lines[index];
            // Horizontal line?
            if line.start.1 == line.end.1 {
                let left = cmp::min(line.start.0, line.end.0);
                let right = cmp::max(line.start.0, line.end.0);
                h_actions.push(Action::start(left, index));
                h_actions.push(Action::end(right + 1, index));
            } else {
                // What is this line's horizontal position at row v?
                let slope = i32::signum(line.end.0 - line.start.0);
                let h = line.start.0 + slope * (v - line.start.1);
                h_actions.push(Action::start(h, index));
                h_actions.push(Action::end(h + 1, index));
            }
        }
        h_actions.sort();
        log::trace!("  row {} h_actions: {:#?}", v, h_actions);

        let mut h_area = 0;
        let mut last_h = 0;
        let mut count = 0;
        for h_action in &h_actions {
            let h_dist = h_action.pos - last_h;
            if count > 1 {
                h_area += h_dist;
            }
            last_h = h_action.pos;
            log::trace!("    count: {}  h_area: {}", count, h_area);

            log::trace!("    action: {:?}", h_action);
            match h_action.kind {
                ActionKind::Start => count += 1,
                ActionKind::End => count -= 1,
            }
        }
        log::trace!("    h_area: {}", h_area);

        area += h_area;
        log::trace!("  area: {}", area);
    }

    area
}

#[test]
fn test_day5_part2_unit() {
    init_logging();

    assert_eq!(sparse_all_lines(&vec![
        Line { start: Point(0,0), end: Point(2,2) },
        Line { start: Point(0,2), end: Point(2,0) },
    ]), 1);

    assert_eq!(sparse_all_lines(&vec![
        Line { start: Point(0,0), end: Point(3,3) },
        Line { start: Point(0,3), end: Point(3,0) },
    ]), 0);

    assert_eq!(sparse_all_lines(&vec![
        Line { start: Point(0,0), end: Point(4,0) },
        Line { start: Point(5,0), end: Point(10,0) },
    ]), 0);

    assert_eq!(sparse_all_lines(&vec![
        Line { start: Point(0,0), end: Point(4,0) },
        Line { start: Point(2,0), end: Point(6,0) },
    ]), 3);

    assert_eq!(sparse_all_lines(&vec![
        Line { start: Point(0,0), end: Point(4,0) }
    ]), 0);

    assert_eq!(sparse_all_lines(&vec![                // a   .   .   .   .
        Line { start: Point(0,0), end: Point(2,2) },  // .   ab  .   .   .
        Line { start: Point(1,1), end: Point(3,3) },  // .   .   abc .   .
        Line { start: Point(2,2), end: Point(4,4) },  // .   .   .  bc  .
    ]), 3);                                           // .   .   .   .   c

    assert_eq!(sparse_all_lines(&vec![
        Line { start: Point(0,0), end: Point(1,1) },
        Line { start: Point(1,1), end: Point(2,2) },
    ]), 1);

    assert_eq!(sparse_all_lines(&vec![
        Line { start: Point(0,0), end: Point(10,0) },
        Line { start: Point(1,0), end: Point(6,0) },
        Line { start: Point(5,0), end: Point(9,0) },
    ]), 9);

}

#[aoc(day5, part2)]
fn part2(lines: &Vec<Line>) -> i32 {
    sparse_all_lines(lines)
}

#[test]
fn test_day5_part2_sample() {
    init_logging();

    let lines = sample_input();
    assert_eq!(sparse_all_lines(&lines), 12);
}
