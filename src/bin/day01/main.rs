mod input;

use input::INPUT;

fn main() {
    let mut increases = 0;
    let mut prev = INPUT[0];
    for &next in &INPUT[1..] {
        if next > prev {
            increases += 1;
        }
        prev = next;
    }

    println!("{}", increases);

    part2(input::INPUT);
}

fn part2(depths: &[i32]) {
let mut increases = 0;
let mut window_sums = depths.windows(3).map(|w| w.iter().sum());
let first: i32 = window_sums.next().unwrap();
window_sums.fold(first, |prev, next| {
    if next > prev {
        increases += 1;
    }
    next
});
println!("part2: {}", increases);
}
