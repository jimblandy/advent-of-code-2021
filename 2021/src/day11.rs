use aoc_runner_derive::{aoc, aoc_generator};
use ndarray::Array2;
#[cfg(test)]
use ndarray::array;
use crate::{conway, around};

#[aoc_generator(day11)]
fn generate(input: &str) -> Array2<u32> {
    let mut array = Array2::zeros((10, 10));
    input.lines().enumerate().for_each(|(r, l)| {
        l.chars().enumerate().for_each(|(c, ch)| {
            array[[r, c]] = ch as u32 - '0' as u32;
        });
    });

    array
}

#[cfg(test)]
fn sample() -> Array2<u32> {
    generate(include_str!("sample/day11"))
}

#[test]
fn test_generate() {
    assert_eq!(sample(),
               array![
                   [ 5, 4, 8, 3, 1, 4, 3, 2, 2, 3 ],
                   [ 2, 7, 4, 5, 8, 5, 4, 7, 1, 1 ],
                   [ 5, 2, 6, 4, 5, 5, 6, 1, 7, 3 ],
                   [ 6, 1, 4, 1, 3, 3, 6, 1, 4, 6 ],
                   [ 6, 3, 5, 7, 3, 8, 5, 4, 7, 8 ],
                   [ 4, 1, 6, 7, 5, 2, 4, 6, 4, 5 ],
                   [ 2, 1, 7, 6, 8, 4, 1, 7, 2, 1 ],
                   [ 6, 8, 8, 2, 8, 8, 1, 1, 3, 4 ],
                   [ 4, 8, 4, 6, 8, 4, 8, 5, 5, 4 ],
                   [ 5, 2, 8, 3, 7, 5, 1, 5, 2, 6 ],
               ]);
}



fn step(energy: &mut Array2<u32>) -> usize {
    let mut flash_count = 0;
    let mut flashed = Array2::from_elem(energy.dim(), false);
    let mut flash_list = vec![];

    for i in 0..energy.nrows() {
        for j in 0..energy.ncols() {
            energy[[i, j]] += 1;
            if energy[[i, j]] > 9 {
                flash_list.push([i, j]);
                flashed[[i, j]] = true;
            }
        }
    }

    let mut next = 0;
    while let Some(&ij) = flash_list.get(next) {
        flash_count += 1;

        for nij in around(ij, energy.dim(), conway()) {
            energy[nij] += 1;
            if energy[nij] > 9 && !flashed[nij] {
                flash_list.push(nij);
                flashed[nij] = true;
            }
        }

        next += 1;
    }

    for &ij in &flash_list {
        energy[ij] = 0;
    }

    flash_count
}

#[test]
fn test_step() {
    let mut energy = sample();

    assert_eq!(step(&mut energy), 0);
    assert_eq!(energy, generate(include_str!("sample/day11.step1")));

    assert_eq!(step(&mut energy), 35);
    assert_eq!(energy, generate(include_str!("sample/day11.step2")));

    let mut total = 35;
    for _ in 2..10 {
        total += step(&mut energy);
    }
    assert_eq!(total, 204);
    assert_eq!(energy, generate(include_str!("sample/day11.step10")));

    for _ in 10..100 {
        total += step(&mut energy);
    }
    assert_eq!(total, 1656);
    assert_eq!(energy, generate(include_str!("sample/day11.step100")));
}

#[aoc(day11, part1)]
fn part1(input: &Array2<u32>) -> usize {
    let mut input = input.clone();
    (0..100).map(|_| step(&mut input)).sum()
}

#[aoc(day11, part2)]
fn part2(input: &Array2<u32>) -> usize {
    let mut input = input.clone();
    (1..1000).find(|_| step(&mut input) == 100).unwrap()
}
