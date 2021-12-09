use aoc_runner_derive::{aoc, aoc_generator};
use ndarray::{Array2, Zip};
#[cfg(test)]
use ndarray::array;

#[aoc_generator(day9)]
fn generate(input: &str) -> Array2<i32> {
    let rows = input.lines().count();
    let columns = input.lines().next().unwrap().len();

    let mut array = Array2::zeros((rows, columns));
    for (i, line) in input.lines().enumerate() {
        for (j, ch) in line.chars().enumerate() {
            array[[i, j]] = ch as i32 - '0' as i32;
        }
    }

    array
}

#[cfg(test)]
fn sample() -> Array2<i32> {
    generate(
        "\
2199943210
3987894921
9856789892
8767896789
9899965678
",
    )
}

#[test]
fn test_generate() {
    assert_eq!(
        sample(),
        array![
            [2, 1, 9, 9, 9, 4, 3, 2, 1, 0],
            [3, 9, 8, 7, 8, 9, 4, 9, 2, 1],
            [9, 8, 5, 6, 7, 8, 9, 8, 9, 2],
            [8, 7, 6, 7, 8, 9, 6, 7, 8, 9],
            [9, 8, 9, 9, 9, 6, 5, 6, 7, 8],
        ]
    );
}

fn compass() -> impl Iterator<Item = (isize, isize)> {
    std::iter::successors(Some((1, 0)), |&(x, y)| Some((-y, x))).take(4)
}

fn in_bounds<T>(a: &Array2<T>, r: isize, c: isize) -> Option<(usize, usize)> {
    // Man, but Rust *hates* signed array indices
    if 0 <= r && (r as usize) < a.nrows() && 0 <= c && (c as usize) < a.ncols() {
        Some((r as usize, c as usize))
    } else {
        None
    }
}

fn neighbors<'a, T>(a: &'a Array2<T>, i: usize, j: usize) -> impl 'a + Iterator<Item = (usize, usize)> {
    compass()
        .filter_map(move |(di, dj)| {
            in_bounds(a, i as isize + di, j as isize + dj)
        })
}

#[aoc(day9, part1)]
fn part1(input: &Array2<i32>) -> i32 {
    Zip::indexed(input).fold(0, |acc, (i, j), &elt| {
        acc + if neighbors(input, i, j)
            .all(|(i, j)| input[[i, j]] > elt)
        {
            elt + 1
        } else {
            0
        }
    })
}

#[test]
fn test_part1() {
    assert_eq!(part1(&sample()), 15);
}

#[aoc(day9, part2)]
fn part2(input: &Array2<i32>) -> usize {
    let mut visited = Array2::from_elem(input.dim(), false);

    let mut pending = vec![];
    let mut sizes = vec![];
    Zip::indexed(input)
        .for_each(|(i, j), &elt| {
            if visited[[i, j]] || elt == 9 {
                return;
            }

            let mut area = 0;
            pending.clear();
            pending.push((i, j));
            while let Some((i, j)) = pending.pop() {
                if !visited[[i, j]] {
                    visited[[i, j]] = true;
                    area += 1;
                    pending.extend(neighbors(input, i, j)
                                   .filter(|&(i, j)| input[[i,j]] != 9));
                }
            }

            sizes.push(area)
        });

    sizes.sort();
    sizes[(sizes.len() - 3) ..].iter().product()
}

#[test]
fn test_part2() {
    assert_eq!(part2(&sample()), 1134);
}
