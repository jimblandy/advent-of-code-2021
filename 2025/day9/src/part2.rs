use std::cmp::{max, min};
use std::ops::Range;

use crate::Problem;
use crate::bands::BandIter;
use crate::edge::Point;

#[derive(Debug)]
struct ActiveRed {
    /// The location of a red floor tile.
    red: Point,

    /// The range of columns within which this tile is visible at the
    /// current row of iteration.
    visible: Range<u64>,
}

pub fn for_each_contained_rectangle(problem: &Problem, mut body: impl FnMut(Point, Point)) {
    // Red tiles that were visible at the end of the prior band. A
    // tile is "visible" if some successive band could form more
    // rectangles enclosed by the shape using that tile as a corner.
    //
    // This list is kept sorted by the red tile's column.
    let mut active = vec![];

    let bands = BandIter::from_edges(problem.edges());
    for band in bands {
        log::debug!("considering band {band:?}");

        // Constrain the visibility of all active red tiles to fall within this
        // band's runs. Drop any tiles that are no longer visible.
        let mut runs = &band.runs[..];
        active.retain_mut(|active: &mut ActiveRed| {
            // Pop off any runs to the left of `active`. Since both lists are
            // sorted, we'll never seen any more tiles in those runs.
            let front = loop {
                let [front, rest @ ..] = runs else {
                    // If runs is empty, then this tile is no longer active. Don't
                    // retain it.
                    return false;
                };

                if active.red.1 < front.end {
                    break front;
                }

                // Since `front.end <= active.red.1` and `active` is sorted, no
                // further active tiles will fall within `front`, and we can
                // remove it from the list.
                runs = rest;
            };

            // If `active`'s column doesn't fall in any of the new band's runs,
            // then it's no longer visible. Note that its visible range might
            // even so intersect some run in the new band, but we've got to be
            // able to drop an edge straight down from `active`, so unless
            // `active`'s column *itself* extends into the current band, it's
            // done.
            if !front.contains(&active.red.1) {
                return false;
            }

            // Since `active` doesn't fall after `front`, and `runs` is sorted,
            // if `active` intersects any run, it'll be `front`. See if there is,
            // in fact, any intersection.
            let Some(remaining) = intersection(&active.visible, front) else {
                return false;
            };

            active.visible = remaining;
            true
        });
        log::debug!("culled active: {active:?}");

        // Build Active entries for the new tiles. But don't add them to `active` yet.
        let mut new_active = vec![];
        let mut runs = &band.runs[..];
        for &red in &band.reds {
            loop {
                match runs {
                    [front, ..] if red < front.start => {
                        unreachable!(
                            "red before first run;\
                                      every red tile in a band should fall in run from that band"
                        );
                    }
                    [front, ..] if front.contains(&red) => {
                        // We've found the run that contains this red tile, so
                        // add it to the active set, using that as its initial
                        // visible range
                        let red = (*band.rows.start(), red);
                        new_active.push(ActiveRed {
                            red,
                            visible: front.clone(),
                        });
                        break; // move on to the next new red tile
                    }
                    [_, rest @ ..] => {
                        // The front run must be to the left of `red`. Since
                        // `band.reds` is sorted, we won't find any more red
                        // tiles that fall within that front run, so we can drop
                        // it.
                        runs = rest;
                    }
                    [] => {
                        unreachable!(
                            "red after last run;\
                                      every red tile in a band should fall in run from that band"
                        );
                    }
                }
            }
        }

        log::debug!("new active: {new_active:?}");
        log::debug!("considering rectangles:");

        // See what rectangles we can form from tiles already in the active list
        // using newly active tiles.
        for a in &active {
            for b in &new_active {
                log::debug!("    considering {:?} .. {:?}", a.red, b.red);
                // If the lower is visible to the upper, then the upper should
                // be visible to the lower as well. But the reverse is not
                // necessarily true, since upper tiles could have been
                // constrained by prior bands.
                assert!(!a.visible.contains(&b.red.1) || b.visible.contains(&a.red.1));
                if a.visible.contains(&b.red.1) {
                    log::debug!("    rectangle {:?} .. {:?} is contained", a.red, b.red);
                    body(a.red, b.red);
                }
            }
        }

        // See what rectangles we can form using only newly active tiles.
        let mut iter = &new_active[..];
        while let [a, rest @ ..] = iter {
            for b in rest {
                log::debug!("    considering {:?} .. {:?}", a.red, b.red);
                // Within the new band, visibility should always be mutual.
                assert!(a.visible.contains(&b.red.1) == b.visible.contains(&a.red.1));
                if a.visible.contains(&b.red.1) {
                    log::debug!("    rectangle {:?} .. {:?} is contained", a.red, b.red);
                    body(a.red, b.red);
                }
            }
            iter = rest;
        }

        // Finally, add the new active tiles to the active set.
        active.append(&mut new_active);
        active.sort_by_key(|tile| tile.red.1);
    }
}

fn intersection<E>(a: &Range<E>, b: &Range<E>) -> Option<Range<E>>
where
    E: std::cmp::Ord + Copy,
{
    let candidate = max(a.start, b.start)..min(a.end, b.end);
    (!candidate.is_empty()).then_some(candidate)
}

fn area(a: &Point, b: &Point) -> u64 {
    let top = min(a.0, b.0);
    let bottom = max(a.0, b.0);
    let left = min(a.1, b.1);
    let right = max(a.1, b.1);
    (bottom - top + 1) * (right - left + 1)
}

pub fn part2(problem: &Problem) -> u64 {
    let mut largest = 1; // surely there is at least one tile

    for_each_contained_rectangle(problem, |a, b| {
        let area = area(&a, &b);
        log::debug!("        area {area}");
        if area > largest {
            largest = area;
        }
    });

    largest
}

#[cfg(test)]
fn collect_rectangles(edges: &[crate::edge::Edge]) -> Vec<crate::edge::Edge> {
    let problem = Problem {
        red: edges.iter().map(|edge| *edge.start()).collect(),
    };

    let mut rectangles = vec![];
    for_each_contained_rectangle(&problem, |a, b| {
        // Omit rectangles that are just edges.
        if a.0 == b.0 || a.1 == b.1 {
            return;
        }
        rectangles.push(a..=b);
    });
    rectangles
}

#[cfg(test)]
use crate::test_data;

#[test]
fn square_simple() {
    assert_eq!(
        collect_rectangles(&test_data::SQUARE_SIMPLE),
        vec![(12, 10)..=(22, 20), (12, 20)..=(22, 10),],
    );
}

#[test]
fn reversed_square() {
    assert_eq!(
        collect_rectangles(&test_data::REVERSED_SQUARE),
        vec![(12, 10)..=(22, 20), (12, 20)..=(22, 10),],
    );
}

#[test]
fn u_shape() {
    assert_eq!(
        collect_rectangles(&test_data::U_SHAPE),
        vec![
            (10, 10)..=(20, 20),
            (10, 40)..=(20, 30),
            (10, 20)..=(30, 10),
            (20, 20)..=(30, 10),
            (20, 20)..=(30, 40),
            (10, 30)..=(30, 40),
            (20, 30)..=(30, 10),
            (20, 30)..=(30, 40)
        ]
    );
}

#[test]
fn example() {
    assert_eq!(
        collect_rectangles(&test_data::EXAMPLE),
        vec![
            (1, 11)..=(3, 7),
            (3, 2)..=(5, 9),
            (1, 7)..=(5, 9),
            (3, 7)..=(5, 2),
            (3, 7)..=(5, 9),
            (1, 11)..=(5, 9),
            (5, 9)..=(7, 11),
            (1, 11)..=(7, 9)
        ]
    );
}

#[test]
fn part2_example() {
    let edges = test_data::EXAMPLE;
    let problem = Problem {
        red: edges.iter().map(|edge| *edge.start()).collect(),
    };

    assert_eq!(part2(&problem), 24);
}
