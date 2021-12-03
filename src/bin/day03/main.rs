#![allow(unused_variables)]

mod input;

fn main() {
    part1(input::SAMPLE_1);
    part2(input::INPUT_1);
}

fn bit_counts(input: &[u32]) -> (usize, [usize; 32]) {
    let mut ones = [0; 32];
    let mut width = 0;
    for &n in input {
        for bit in 0..32 {
            if n & (1 << bit) != 0 {
                ones[bit] += 1;
                if bit + 1 > width {
                    width = bit + 1;
                }
            }
        }
    }

    (width, ones)
}

fn gamma_epsilon(width: usize, ones: &[usize], len: usize) -> (u32, u32) {
    let gamma = ones[..width].iter().enumerate().fold(0_u32, |a, (i, &ones)| {
        a | if ones >= len / 2 {
            1 << i
        } else {
            0
        }
    });
    let top_bit = 1 << (width - 1);
    let epsilon = gamma ^ (top_bit | (top_bit - 1));

    (gamma, epsilon)
}

fn part1(input: &[u32]) {
    let (width, ones) = bit_counts(input);
    let (gamma, epsilon) = gamma_epsilon(width, &ones, input.len());

    println!("gamma = {:#b}, epsilon = {:#b}, product = {}", gamma, epsilon, gamma * epsilon);
}


fn refine(sorted: &[u32], majority: bool, width: usize) -> u32 {
    let mut range = 0..sorted.len();
    for bit in (0..width).rev() {
        let mask = 1 << bit;

        // Find the position of the first number in the range with a one bit in
        // the current position, if any. Since the list is sorted, this tells us
        // the relative frequency of each digit.
        let zeros = sorted[range.clone()].iter().position(|&n| (n & mask) != 0)
            .unwrap_or(range.len());
        let ones = range.len() - zeros;

        if (zeros > ones) == majority {
            // keep the zeros
            range.end = range.start + zeros;
        } else {
            // keep the ones
            range.start += zeros;
        }

        if range.len() == 1 {
            return sorted[range.start];
        }
    }

    panic!("duplicate numbers?");
}

fn part2(input: &[u32]) {
    let mut input = input.to_vec();
    input.sort();
    let width = 32 - input.last().unwrap().leading_zeros() as usize;

    let oxygen = refine(&input, true, width);
    println!("oxygen = {}", oxygen);

    let co2 = refine(&input, false, width);
    println!("co2 = {}", co2);

    println!("life support rating: {}", oxygen * co2);
}
