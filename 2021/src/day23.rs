#![allow(unused_imports, dead_code)]
use aoc_runner_derive::{aoc, aoc_generator};
use anyhow::{anyhow, bail, Result};
use std::{cmp, fmt, iter};
use crate::astar_weighted::astar_weighted;

// #############
// #89abcdefghi#
// ###0#2#4#6### where a..i == 10..18
//   #1#3#5#7#
//   #########
//
// Amphipods are identified by the spots they belong in (although, either
// amphipod can go in either spot). So amphipods 0 and 1 are Amber.

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
struct Positions {
    pos: [usize; 8],
}

struct Map {
    map: [usize; 19],
}

#[aoc_generator(day23, part1, jimb)]
fn generator(input: &str) -> Result<Positions> {
    let lines = input.lines().collect::<Vec<_>>();

    let mut pos = [!0; 8];
    let mut map = [EMPTY; 19];
    assert_eq!(lines[0], "#############");

    for hall in 8..19 {
        map[hall] = ascii_to_map(lines[1].as_bytes()[hall - 8 + 1]);
    }

    map[0] = ascii_to_map(lines[2].as_bytes()[3]);
    map[1] = ascii_to_map(lines[3].as_bytes()[3]);

    map[2] = ascii_to_map(lines[2].as_bytes()[5]);
    map[3] = ascii_to_map(lines[3].as_bytes()[5]);

    map[4] = ascii_to_map(lines[2].as_bytes()[7]);
    map[5] = ascii_to_map(lines[3].as_bytes()[7]);

    map[6] = ascii_to_map(lines[2].as_bytes()[9]);
    map[7] = ascii_to_map(lines[3].as_bytes()[9]);

    assert_eq!(lines[4], "  #########");

    for i in 0..19 {
        if map[i] == EMPTY {
            continue;
        }
        if pos[map[i]] != !0 {
            map[i] += 1;
        }
        pos[map[i]] = i;
    }

    Ok(Positions { pos })
}

impl Positions {
    /// An underestimate of how much energy this pod must expend to get home.
    /// But zero only if pod is home.
    fn mome(&self, pod: usize) -> usize {
        let room = pod & !1;
        let pos = self.pos[pod];
        if pos & !1 == room {
            0
        } else {
            cost(pod, pos, room)
        }
    }

    /// An underestimate of the total amount of energy that must be expended
    /// to resolve this state. Zero only if puzzle is solved.
    fn all_mome(&self) -> usize {
        (0..PODS).map(|pod| self.mome(pod)).sum()
    }
}

impl Map {
    fn from_positions(pos: &Positions) -> Self {
        let mut map = [EMPTY; 19];
        for pod in 0..PODS {
            if pos.pos[pod] != !0 {
                map[pos.pos[pod]] = pod;
            }
        }

        Map { map }
    }
}

fn steps(a: usize, b: usize) -> usize {
    let a = a as isize;
    let b = b as isize;
    if a < 8 && b < 8 && a & !1 == b & !1 {
        (a - b).abs() as usize
    } else {
        fn to_hall(p: isize) -> (isize, isize) {
            if p < 8 {
                ((p & 1) + 1, 10 + (p & !1))
            } else {
                (0, p)
            }
        }
        let (pre, a) = to_hall(a);
        let (post, b) = to_hall(b);
        (pre + (a - b).abs() + post) as usize
    }
}

fn cost(pod: usize, a: usize, b: usize) -> usize {
    steps(a, b) * 10_usize.pow(pod as u32 / 2)
}

fn pod_moves(map: &Map, pod: usize, start: usize) -> Vec<usize> {
    let mut moves = vec![];

    // If we're in a room, we can move to the other spot in that room, if it's empty.
    if start < 8 && map.map[start ^ 1] == EMPTY {
        moves.push(start ^ 1);
    }

    // If we are in a hallway, or can reach a hallway, consider where we can go
    // from there.
    if start >= 8 || (start < 8 && start & 1 == 0 || map.map[start - 1] == EMPTY) {
        let hall = if start >= 8 { start } else { 10 + (start & !1) }; // known empty

        // Check which hallway positions we might move to.
        for dest in 8..19 {
            // Don't move to our current position.
            if dest == start {
                continue;
            }

            // Is `dest` reachable from `hall`?
            if map.map[cmp::min(dest, hall) ..= cmp::max(dest, hall)]
                .iter()
                .all(|&m| m == EMPTY || m == pod) {
                    // Is `dest` outside a door?
                    let is_door = 10 <= dest && dest <= 16 && dest & 1 == 0;

                    if is_door {
                        // Is this a room we'd consider entering?
                        let room = dest - 10;
                        if room == pod & !1 &&
                            map.map[room..room + 2].iter().all(|&m| m == EMPTY || m & !1 == pod & !1)
                        {
                            if map.map[room] == EMPTY {
                                moves.push(room);
                                if map.map[room + 1] == EMPTY {
                                moves.push(room + 1);
                                }
                            }
                        }
                    } else {
                        // This isn't outside a door, so we might stop there.
                        moves.push(dest);
                    }
            }
        }
    }

    moves.sort();
    moves
}

#[aoc(day23, part1, jimb)]
fn part1(input: &Positions) -> usize {
    let states = astar_weighted(input.clone(), |positions| {
        let map = Map::from_positions(positions);
        (0..PODS)
            .flat_map(|pod| {
                iter::repeat(pod).zip(pod_moves(&map, pod, positions.pos[pod]))
            })
            .map(|(pod, dest)| {
                let mut neighbor = positions.clone();
                neighbor.pos[pod] = dest;
                let weight = cost(pod, positions.pos[pod], dest);
                let estimate = neighbor.all_mome();
                (neighbor, weight, estimate)
            })
            .collect::<Vec<_>>()
    });

    for edge in states {
        if edge.estimate == 0 {
            /*
            println!("cost: {}, estimate: {}\n{}->\n{}",
                     edge.path_weight,
                     edge.estimate,
                     Map::from_positions(&edge.from),
                     Map::from_positions(&edge.to));
            */

            return edge.path_weight;
        }
    }

    panic!("Never found solution");
}

fn is_valid(pos: &Positions, map: &Map) -> bool {
    let mut seen = [false; PODS];
    for (i, &square) in map.map.iter().enumerate() {
        if square < PODS {
            seen[square] = true;
            if pos.pos[square] != i {
                return false;
            }
        }
    }
    seen.iter().all(|&s| s)
}

impl fmt::Display for Map {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "#############")?;

        write!(f, "#")?;
        for i in 8..19 {
            write!(f, "{}", map_to_char(self.map[i]))?;
        }
        writeln!(f, "#")?;

        write!(f,"###")?;
        for i in 0..4 {
            write!(f, "{}#", map_to_char(self.map[i * 2]))?;
        }
        writeln!(f,"##")?;

        write!(f,"  #")?;
        for i in 0..4 {
            write!(f, "{}#", map_to_char(self.map[i * 2 + 1]))?;
        }
        writeln!(f)?;

        writeln!(f, "  #########")?;

        Ok(())
    }
}

const PODS: usize = 8;
const EMPTY: usize = PODS;

fn ascii_to_map(byte: u8) -> usize {
    match byte {
        b'A' => 0,
        b'B' => 2,
        b'C' => 4,
        b'D' => 6,
        b'.' => EMPTY,
        _ => panic!("unexpected character in map"),
    }
}

fn map_to_char(map: usize) -> char {
    match map {
        0 | 1 => 'A',
        2 | 3 => 'B',
        4 | 5 => 'C',
        6 | 7 => 'D',
        8 => '.',
        _ => panic!("unexpected value in map"),
    }
}

#[test]
fn test_generator() {
    assert_eq!(generator(&include_str!("sample/day23")).unwrap(),
               Positions {
                   pos: [1, 7, 0, 4, 2, 5, 3, 6],
               },
    );
}

#[test]
fn test_steps() {
    assert_eq!(steps(0,0), 0);
    assert_eq!(steps(7,7), 0);
    assert_eq!(steps(12,12), 0);

    assert_eq!(steps(0,1), 1);
    assert_eq!(steps(1,0), 1);
    assert_eq!(steps(6,7), 1);
    assert_eq!(steps(7,6), 1);

    assert_eq!(steps(8,18), 10);
    assert_eq!(steps(18,8), 10);

    assert_eq!(steps(0,7), 9);
    assert_eq!(steps(6,1), 9);
}

#[test]
fn test_cost() {
    assert_eq!(cost(6, 0, 7), 9000);
    assert_eq!(cost(7, 4, 6), 4000);
    assert_eq!(cost(0, 5, 9), 7);
}

#[test]
fn test_pod_moves() {
    let pos = generator(&include_str!("sample/day23")).unwrap();
    let map = Map::from_positions(&pos);

    assert_eq!(pod_moves(&map, 0, pos.pos[0]), vec![]);
    assert_eq!(pod_moves(&map, 1, pos.pos[1]), vec![]);
    assert_eq!(pod_moves(&map, 2, pos.pos[2]), vec![8, 9, 11, 13, 15, 17, 18]);
    assert_eq!(pod_moves(&map, 3, pos.pos[3]), vec![8, 9, 11, 13, 15, 17, 18]);
    assert_eq!(pod_moves(&map, 4, pos.pos[4]), vec![8, 9, 11, 13, 15, 17, 18]);
    assert_eq!(pod_moves(&map, 5, pos.pos[5]), vec![]);
    assert_eq!(pod_moves(&map, 6, pos.pos[6]), vec![]);
    assert_eq!(pod_moves(&map, 7, pos.pos[7]), vec![8, 9, 11, 13, 15, 17, 18]);

    let pos = generator("\
#############
#.......B...#
###.#A#.#D###
  #.#.#.#.#
  #########
").unwrap();
    let map = Map::from_positions(&pos);

    assert_eq!(pod_moves(&map, 0, pos.pos[0]), vec![0, 1, 3, 8, 9, 11, 13]);
    assert_eq!(pod_moves(&map, 2, pos.pos[2]), vec![8, 9, 11, 13, 17, 18]);
    assert_eq!(pod_moves(&map, 6, pos.pos[6]), vec![7, 17, 18]);
}
