static PART1_TEST: &[&[u64]] = &[
    &[9, 8, 7, 6, 5, 4, 3, 2, 1, 1, 1, 1, 1, 1, 1],
    &[8, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 9],
    &[2, 3, 4, 2, 3, 4, 2, 3, 4, 2, 3, 4, 2, 7, 8],
    &[8, 1, 8, 1, 8, 1, 9, 1, 1, 1, 1, 2, 1, 1, 1],
];

fn max_joltage(bank: &[u64]) -> u64 {
    let best_second_after = bank.iter().rev().fold(0, |j, best|)
    todo!()
}

fn main() {
    println!("Hello, world!");
}
