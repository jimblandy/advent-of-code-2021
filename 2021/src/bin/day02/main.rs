mod input;

pub const fn up(n: i32) -> (i32, i32) { ( -n, 0 ) }
pub const fn down(n: i32) -> (i32, i32) { ( n, 0 ) }
pub const fn forward(n: i32) -> (i32, i32) { ( 0, n ) }

fn main() {
    let (v, h) = input::INPUT_1.iter().fold((0,0), |a, d| (a.0 + d.0, a.1 + d.1));
    println!("{:?} {}", (v, h), v * h);

    let mut pos = (0, 0);
    let mut aim = 0;
    for (aim_change, forward) in input::INPUT_1 {
        aim += aim_change;
        pos.0 += aim * forward;
        pos.1 += forward;
    }
    println!("{:?} {}", pos, pos.0 * pos.1);
}
