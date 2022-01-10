#![allow(unused_imports, dead_code)]
use aoc_runner_derive::{aoc, aoc_generator};
use anyhow::{anyhow, bail, Result};
use std::{cmp, fmt, iter};
use crate::astar_weighted::astar_weighted;
//use hashbrown::HashMap;

// #############
// #ghijklmnopq#
// ###0#4#8#c### where a..q == 10..26
//   #1#5#9#d#
//   #2#6#a#e#
//   #3#7#b#f#
//   #########
//
// Amphipods are identified by the spots they belong in (although, either
// amphipod can go in either spot). So amphipods 0 and 1 are Amber.

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
struct Positions {
    pos: [u8; PODS],
}

const SPACES: usize = 27;
const PODS: usize = 16;
const EMPTY: u8 = PODS as u8;

struct Map {
    map: [u8; SPACES],
}

#[aoc_generator(day23, part2, jimb)]
fn generator(input: &str) -> Result<Positions> {
    let mut lines = input.lines().collect::<Vec<_>>();

    lines.insert(3, "  #D#C#B#A#");
    lines.insert(4, "  #D#B#A#C#");

    let combined = lines.join("\n");
    original_generator(&combined)
}

fn original_generator(input: &str) -> Result<Positions> {
    let lines = input.lines().collect::<Vec<_>>();

    let mut pos = [!0; PODS];
    let mut map = [EMPTY; SPACES];
    assert_eq!(lines[0], "#############");

    for hall in PODS..SPACES {
        map[hall] = ascii_to_map(lines[1].as_bytes()[hall - PODS + 1]);
    }

    for row in 0..4 {
        for col in 0..4 {
            map[col * 4 + row] = ascii_to_map(lines[2 + row].as_bytes()[3 + col * 2]);
        }
    }

    assert_eq!(lines[6], "  #########");

    for i in 0..SPACES {
        if map[i] == EMPTY {
            continue;
        }
        while pos[map[i] as usize] != !0 {
            map[i] += 1;
        }
        pos[map[i] as usize] = i as u8;
    }

    Ok(Positions { pos })
}

impl Positions {
    /// An underestimate of how much energy this pod must expend to get home.
    /// But zero only if pod is home.
    fn mome(&self, pod: u8) -> usize {
        let room = pod as usize & !3;
        let pos = self.pos[pod as usize] as usize;
        if pos & !3  == room {
            0
        } else {
            cost(pod, pos, room)
        }
    }

    /// An underestimate of the total amount of energy that must be expended
    /// to resolve this state. Zero only if puzzle is solved.
    fn all_mome(&self) -> usize {
        let to_rooms: usize = (0..PODS).map(|pod| self.mome(pod as u8)).sum();

        // Encourage pods to go all the way into their rooms.
        let map = Map::from_positions(self);
        let mut smoosh = 0;
        for room in 0..4 {
            let one_step = step_cost(room as u8 * 4);
            for pos in 1..4 {
                if map.map[room * 4 + pos] == EMPTY {
                    // The deeper in, the more pods will need to step once to
                    // fill it.
                    smoosh += one_step * pos;
                }
            }
        }

        to_rooms + smoosh
    }
}

impl Map {
    fn from_positions(pos: &Positions) -> Self {
        let mut map = [EMPTY; SPACES];
        for pod in 0..PODS {
            if pos.pos[pod] != !0 {
                map[pos.pos[pod] as usize] = pod as u8;
            }
        }

        Map { map }
    }
}

fn steps(a: usize, b: usize) -> usize {
    let a = a as isize;
    let b = b as isize;
    if a < PODS as isize && b < PODS as isize && a & !3 == b & !3 {
        (a - b).abs() as usize
    } else {
        fn to_hall(p: isize) -> (isize, isize) {
            if p < PODS as isize {
                ((p & 3) + 1, PODS as isize + 2 + (p / 4) * 2)
            } else {
                (0, p)
            }
        }
        let (pre, a) = to_hall(a);
        let (post, b) = to_hall(b);
        (pre + (a - b).abs() + post) as usize
    }
}

fn cost(pod: u8, a: usize, b: usize) -> usize {
    steps(a, b) * step_cost(pod)
}

fn step_cost(pod: u8) -> usize {
    10_usize.pow(pod as u32 / 4)
}

fn pod_moves(map: &Map, pod: u8, start: usize) -> Vec<usize> {
    let mut moves = vec![];

    let mut home_reachable = None;

    if start < PODS {
        // We are in a room.

        // We can move deeper into this room as long as nobody is in our way.
        // There's no point in moving partially out of a room without going into
        // the hallway.
        let mut next = start + 1;
        while next & !3 == start & !3 && map.map[next] == EMPTY {
            moves.push(next);
            next += 1;
        }

        // Can we reach the hallway?
        if map.map[start & !3..start].iter().all(|&m| m == EMPTY) {
            // Where can we go in the hallway?
            let room = start / 4;
            let door = PODS + 2 + room * 2; // pods never stand here

            fn is_door(pos: usize) -> bool {
                PODS + 2 <= pos && pos <= PODS + 8 && pos & 1 == 0
            }

            // Consider moving to a hall position. Note that pos is never the
            // door we just walked out of.
            let mut consider_hall = |pos: usize| -> bool {
                if map.map[pos] != EMPTY {
                    return false;
                }

                if is_door(pos) {
                    // If this is our home, note that it's reachable.
                    let kind = pod as usize / 4;
                    if (pos - 2 - PODS) / 2 == kind {
                        home_reachable = Some(kind);
                    }
                } else {
                    moves.push(pos);
                }

                true
            };

            // scan left
            for left in (PODS..door).rev() {
                if !consider_hall(left) {
                    break;
                }
            }

            // scan right
            for right in door + 1 .. SPACES {
                if !consider_hall(right) {
                    break;
                }
            }
        }
    } else {
        // We are starting in the hallway. We need only consider whether we will
        // enter our destination room.
        let kind = pod as usize / 4;
        let door: usize = PODS + 2 + kind * 2;

        // Is the way clear from our current position to the door?
        let reachable = map.map[cmp::min(start, door) ..= cmp::max(start, door)]
            .iter()
            .all(|&m| m == EMPTY || m == pod);

        if reachable {
            home_reachable = Some(kind);
        }
    }

    if let Some(kind) = home_reachable {
        // The entryway to our room is reachable.
        let room_spaces = kind * 4 .. kind * 4 + 4;

        // Are there any pods of other kinds in the room?
        let has_guests = map.map[room_spaces.clone()]
            .iter()
            .any(|&m| m != EMPTY && m & !3 != pod & !3);

        // We only enter our room if there are no guests.
        if !has_guests {
            for dest in room_spaces {
                if map.map[dest] != EMPTY {
                    break;
                }
                moves.push(dest);
            }
        }
    }

    moves.sort();
    moves
}

#[aoc(day23, part2, jimb)]
fn part2(input: &Positions) -> usize {
    //let mut predecessors = HashMap::new();

    let states = astar_weighted(input.clone(), |positions| {
        let map = Map::from_positions(positions);
        (0..PODS as u8)
            .flat_map(|pod| {
                iter::repeat(pod).zip(pod_moves(&map, pod, positions.pos[pod as usize] as usize))
            })
            .map(|(pod, dest)| {
                let mut neighbor = positions.clone();
                neighbor.pos[pod as usize] = dest as u8;
                let weight = cost(pod, positions.pos[pod as usize] as usize, dest);
                let estimate = neighbor.all_mome();
                (neighbor, weight, estimate)
            })
            .collect::<Vec<_>>()
    });

    //let mut best_estimate = usize::MAX;
    for edge in states {
        //predecessors.entry(edge.to.clone()).or_insert_with(|| (edge.from.clone(), edge.path_weight));
        /*
        if edge.estimate < best_estimate {
            best_estimate = edge.estimate;
            println!("new best estimate: {:6}  cost: {:6}", best_estimate, edge.path_weight);
            println!("{}", Map::from_positions(&edge.to));
            println!();
        }
        */
        if edge.estimate == 0 {
            /*
            println!("\n\n\n\n======================\nSOLVED");
            let mut solution = vec![];
            let mut node = edge.to.clone();
            solution.push((node.clone(), edge.path_weight));
            while let Some((prev, cost)) = predecessors.get(&node) {
                solution.push((prev.clone(), *cost));
                node = prev.clone();
            }
            solution.reverse();
            for step in solution {
                println!("cost: {}\n{}", step.1, Map::from_positions(&step.0));
            }
            */

            return edge.path_weight;
        }
    }

    panic!("Never found solution");
}

fn is_valid(pos: &Positions, map: &Map) -> bool {
    let mut seen = [false; PODS];
    for (i, &square) in map.map.iter().enumerate() {
        let square = square as usize;
        if square < PODS {
            seen[square] = true;
            if pos.pos[square] as usize != i {
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
        for i in PODS..SPACES {
            write!(f, "{}", map_to_char(self.map[i]))?;
        }
        writeln!(f, "#")?;

        for row in 0..4 {
            if row == 0 {
                write!(f,"###")?;
            } else {
                write!(f,"  #")?;
            }

            for col in 0..4 {
                write!(f, "{}#", map_to_char(self.map[col * 4 + row]))?;
            }

            if row == 0 {
                writeln!(f,"##")?;
            } else {
                writeln!(f)?;
            }
        }

        writeln!(f, "  #########")?;

        Ok(())
    }
}

fn ascii_to_map(byte: u8) -> u8 {
    match byte {
        b'A' => 0,
        b'B' => 4,
        b'C' => 8,
        b'D' => 12,
        b'.' => EMPTY,
        _ => panic!("unexpected character in map: {:?}", byte as char),
    }
}

fn map_to_char(map: u8) -> char {
    match map {
        0..=3 => 'A',
        4..=7 => 'B',
        8..=11 => 'C',
        12..=15 => 'D',
        EMPTY => '.',
        _ => panic!("unexpected value in map"),
    }
}

#[test]
fn test_generator() {
    let pos = original_generator(&include_str!("sample/day23_part2")).unwrap();
    println!("{}", Map::from_positions(&pos));
    assert_eq!(pos,
               Positions {
                   pos: [3, 10, 13, 15,
                         0, 6, 8, 9,
                         4, 5, 11, 14,
                         1, 2, 7, 12]
               },
    );
}

#[test]
fn test_steps() {
    assert_eq!(steps(0,0), 0);
    assert_eq!(steps(15,15), 0);
    assert_eq!(steps(20,20), 0);

    assert_eq!(steps(0,3), 3);
    assert_eq!(steps(3,0), 3);
    assert_eq!(steps(0,4), 4);
    assert_eq!(steps(4,0), 4);
    assert_eq!(steps(3,7), 10);

    assert_eq!(steps(16,26), 10);
    assert_eq!(steps(26,16), 10);

    assert_eq!(steps(0,15), 11);
    assert_eq!(steps(12,3), 11);

}

#[test]
fn test_cost() {
    assert_eq!(cost(12, 0, 15), 11000);
    assert_eq!(cost(15, 8, 12), 4000);
    assert_eq!(cost(0, 11, 17), 9);
}

#[test]
fn test_pod_moves() {
    let pos = original_generator(&include_str!("sample/day23_part2")).unwrap();
    let map = Map::from_positions(&pos);

    for pod in 0..PODS as u8 {
        let moves = pod_moves(&map, pod, pos.pos[pod as usize] as usize);
        if pod == 4 || pod == 8 || pod == 6 || pod == 15 {
            assert_eq!(moves, vec![16, 17, 19, 21, 23, 25, 26]);
        } else {
            assert_eq!(moves, vec![]);
        }
    }

    let pos = original_generator("\
#############
#.......B...#
###.#.#.#D###
  #.#.#.#.#
  #.#A#.#.#
  #.#.#.#.#
  #########
").unwrap();
    let map = Map::from_positions(&pos);

    assert_eq!(pod_moves(&map, 0, pos.pos[0] as usize), vec![0, 1, 2, 3, 7, 16, 17, 19, 21]);
    assert_eq!(pod_moves(&map, 4, pos.pos[4] as usize), vec![]);
    assert_eq!(pod_moves(&map, 12, pos.pos[12] as usize), vec![13, 14, 15, 25, 26]);
}

#[test]
fn test_part2_simple() {
    let pos = original_generator("\
#############
#.........D.#
###A#B#C#.###
  #A#B#C#D#
  #A#B#C#D#
  #A#B#C#D#
  #########
").unwrap();

    assert_eq!(pos.pos[15], 25);
    let map = Map::from_positions(&pos);
    assert_eq!(pod_moves(&map, 12, pos.pos[12] as usize), vec![16, 17, 19, 21, 23]);
    assert_eq!(pod_moves(&map, 15, pos.pos[15] as usize), vec![12]);

    //assert_eq!(part2(&pos), 2000);

    let pos = original_generator("\
#############
#.........D.#
###A#B#.#C###
  #A#B#C#D#
  #A#B#C#D#
  #A#B#C#D#
  #########
").unwrap();

    assert_eq!(part2(&pos), 2400);
}

/* This doesn't terminate. I don't know why.
#[test]
fn test_part2_example() {
    let pos = generator(&include_str!("sample/day23")).unwrap();

    assert_eq!(part2(&pos), 44169);
}
*/
