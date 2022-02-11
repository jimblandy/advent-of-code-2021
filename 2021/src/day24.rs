#![allow(unused_variables)]
use aoc_runner_derive::{aoc, aoc_generator};
use anyhow::Result;
use std::{cmp, fmt};
use std::ops::RangeInclusive;

macro_rules! log {
    { $format:literal $( , $arg:expr )* } => {
        ()
//        println!( $format $( , $arg )* )
    }
}

// My input consists of 14 rounds (repetitions) of a block of code equivalent
// to:
//
//     w = input;
//     x = z % 26 + a0 != w;
//     if a0 <= 0 {
//         z /= 26;
//     }
//     if x {
//         z = z * 26 + w + a1;
//     }
//
// where `input` is one input value, in the range `1..=9`, and `a0` and `a1` are
// constants that vary from one round to the next.
//
// The only state passed from one round to the next is `z`; all other registers
// can be treated as temporaries local to the round. We also know the following
// about `z`, in any run whose input is a valid model number:
//
// - It is positive upon entry to every round.
// - It is zero upon entry to the first round.
// - It is zero upon exit from the last round.
//
// If you look at the value of z as a number in base 26, then `z % 26` returns
// its last digit, `z /= 26` deletes its last digit, and `z * 26 + m` appends
// the digit `m`, if `m` is in the range 0..26 (and, given the values of a1 in
// my input, and the permitted input values, it always is).
//
// So it may make sense to look at `z` as a stack of numbers in the range 0..26,
// and where an empty stack is considered to hold zeros:
//
// - `z % 26` is the top of the stack
//
// - `z /= 26` pops the stack, and
//
// - `z = z * 26 + m` pushes `m` on the stack.
//
// Rewritten in these terms, the code for a round is:
//
//     w = input;
//     x = top() + a0 != w;
//     if a0 <= 0 {
//         pop();
//     }
//     if x {
//         push(w + a1);
//     }
//
// where `top`, `pop`, and `push` do the obvious things to the stack represented
// by `z`.
//
// For every round, either a0 > 9 or a0 <= 0. This tells us a lot about how the
// round will behave:
//
// - In a round where a0 > 9:
//   - top() + a0 > 9
//   - since w is in 1..=9, x is always 1
//   - Since a0 > 0 and x is 1, we push w + a1 on the stack.
//   - Since w is in 1..=9 and a1 is in 2 ..=14, w + a1 is in `3 ..= 23`,
//     so the stack always grows, with a non-zero top.
//
// - On any round where a0 <= 0:
//   - If input == top() + a0 (that is, x is 0):
//     - top() + a0 must be in 1..=9, so top() must be in 1-a0 ..= 9-a0
//     - We pop the stack.
//   - Otherwise, input != top() + a0 (that is, x is 1):
//     - The inbound top of stack could potentially be any value.
//     - We replace the top of the stack with `w + a1`.
//     - Since w is in 1..=9 and a1 is in 2..=11 for these rounds, w + a1 is in
//       3..=20, so the stack definitely becomes non-empty, with a non-zero top.
//
// Working through a few rounds by hand:
//
// In discussing a particular round, let `z[i]` be the value of `z` upon entry
// to round `i`, and by extension, let `z[14]` be the value of `z` upon exit
// from the last round (round 13). If the model number is valid, both `z[0]` and
// `z[14]` are zero.
//
// Similarly, let `w[i]` be the input read by round `i`.
//
// The model number validates if `z[14] == 0`. Since the last round has `a0 <=
// 0`, `x` must be 0, and we popped the stack. (If `x` were 1, then the top of
// the stack would be in 3..=20, and we wouldn't have validated.) Thus `w` must
// be `incoming top - 2`, so the incoming stack must be a single value in 3..=11.
//
// Round 12 has `a0 <= 0`. Since `z[13]` is in `3..=11`, we must have replaced
// the top of the stack, not popped it, so `x` is true, and `z[13] = w[12] + 5`,
// so `w[12]` is in the range `-2..=6`. But we know every `w` is in `1..=9`, so
// `w[12]`'s range is `1..=6` and `z[13]` must actually be in `6..=11`, and thus
// `w[13]` must be in `4..=9`. And since we replaced the top of the stack,
// `z[12]` could be anything in `0..26`. Note that w[12] != z[12] - 11.
//
// Round 11 has `a0 <= 0`. Since `z[12]` is unconstrained, we could have either
// replaced or popped the stack. If we replaced it, `z[12] = w[11] + 11`, so
// z[12] would have to be in `12..=20`. If we popped it, `z[12]` would have to
// be 0. So `z[12]` must actually be either 0 or in 12..=20. In either case,
// `z[11]` is anything in `0..26`, and `w[11]` is anything in 1..=9.
//
// Round 10 has `a0 <= 0`. Since `z[11]` is unconstrained, we could have either
// replaced or popped the stack.

#[derive(Copy, Clone, Debug)]
struct Round {
    a0: i64,
    a1: i64,
}

type Problem = &'static [Round];

static PART1_INPUT: Problem = &[
    Round { a0:  10, a1:  2 }, // 0
    Round { a0:  10, a1:  4 }, // 1
    Round { a0:  14, a1:  8 }, // 2
    Round { a0:  11, a1:  7 }, // 3
    Round { a0:  14, a1: 12 }, // 4
    Round { a0: -14, a1:  7 }, // 5
    Round { a0:   0, a1: 10 }, // 6
    Round { a0:  10, a1: 14 }, // 7
    Round { a0: -10, a1:  2 }, // 8
    Round { a0:  13, a1:  6 }, // 9
    Round { a0: -12, a1:  8 }, // 10
    Round { a0: - 3, a1: 11 }, // 11
    Round { a0: -11, a1:  5 }, // 12
    Round { a0: - 2, a1: 11 }, // 13
];


/// Given a solution from `next_round` forwards, find the constraints necessary to pass earlier rounds.
///
/// This uses recursive backtracking to find all sets of constraints that ensure
/// a valid model number. When we find a set of executions that validate the
/// model number, we pass its constraints to `report`.
///
/// At each recursive call, the contents of `xs[next_round..]` and
/// `zs[next_round..]` are valid. Earlier elements are dead.
///
/// Each `xs[i]` is the value `x` must have in round `i`.
///
/// Each `zs[i]` is a `Vec<BitSet>`, representing the constraints on the "stack"
/// represented by `z` upon entry to round `i`. If you write `z` in base 26,
/// then the `d`'th digit must be present in the `d`'th `BitSet` in the vector.
/// The length of the vector must match the number of base-26 digits in `z`.
fn solve_back(next_round: usize, xs: &mut [i64], zs: &mut [Vec<BitSet>], rounds: Problem, report: &mut impl FnMut(&[i64], &[Vec<BitSet>])) {
    if next_round == 0 {
        report(xs, zs);
        return;
    }

    let round = next_round - 1;
    let Round { a0, a1 } = rounds[round];

    // Indent two stops for each round we've solved.
    let indent = Indent(2 * (rounds.len() - round));
    log!("{}Round {}: a0={}, outbound stack: {:?}", indent, round, a0, zs[next_round]);
    let indent = indent.next();

    if a0 > 9 {
        log!("{}This always pushes input{:+}", indent, a1);
        // These rounds always leave a number in 1 + a1 ..= 9 + a1 on the top of
        // the stack. Is that possible?
        match zs[next_round].last() {
            Some(set) => {
                if !set.overlaps(1 + a1 ..= 9 + a1) {
                    log!("{}FAIL: outbound top of stack cannot accommodate anything in {}",
                             indent, BitSet::from(1 + a1 ..= 9 + a1));
                    return;
                }
            }
            None => {
                log!("{}FAIL: outbound stack must be empty", indent);
                return;
            }
        }

        // On these rounds, x may only be 1.
        xs[round] = 1;

        // We push an element on the stack, so our incoming stack must lack that value.
        zs[round] = zs[next_round].clone();
        zs[round].pop();

        solve_back(round, xs, zs, rounds, report);
    } else if a0 <= 0 {
        // Could x be zero? That would mean that input == top() + a0, and we pop
        // the stack. No possible outgoing stack rules this out.
        log!("{}Suppose input == incoming top(){:+} (a0):", indent, a0);
        {
            log!("{}This always pops the stack.", indent);

            xs[round] = 0;

            // Since input = top() + a0, top() = input - a0.
            // We pop the stack, so our incoming stack must have that value.
            zs[round] = zs[next_round].clone();
            zs[round].push(BitSet::from(1 - a0 ..= 9 - a0));

            solve_back(round, xs, zs, rounds, report);
        }

        // Could x be one? That would mean that input != top() + a0, and we
        // replace the top of the stack with input + a1, which is never zero.
        // The outgoing stack needs to be non-empty, and permit a value in this
        // range.
        match zs[next_round].last() {
            None => {
                log!("{}Input can't be != incoming top(){:+} (a0):", indent, a0);
                log!("{}that replaces tos, but outgoing stack is empty", indent);

            }
            Some(top) => {
                // outgoing top is input + a1
                if !top.overlaps(1 + a1 ..= 9 + a1) {
                    log!("{}Input can't be != incoming top(){:+} (a0)", indent, a0);
                    log!("{}top of outgoing stack doesn't permit anything in {} (input + a1)",
                             indent, BitSet::from(1 + a1 ..= 9 + a1));
                }

                log!("{}Suppose input != incoming top(){:+} (a0):", indent, a0);
                log!("{}This replaces the top of the stack.", indent);
                xs[round] = 1;
                zs[round] = zs[next_round].clone();
                *zs[round].last_mut().unwrap() = BitSet::from(0..=25);

                solve_back(round, xs, zs, rounds, report);
            }
        }
    } else {
        unreachable!()
    }
}

/// Find the largest/smallest valid model number, Given values of `x` and constraints on
/// `z` for all rounds.
fn solve_forward(largest: bool, xs: &[i64], zs: &[Vec<BitSet>], rounds: Problem) -> Vec<i64> {
    let mut inputs = vec![];
    let mut z = 0;

    for (i, ((&Round { a0, a1 }, &x), allowed_z)) in rounds.iter().zip(xs).zip(zs[1..].iter()).enumerate() {
        log!("Round {}: a0 = {}, a1 = {}, incoming z = {}, x = {}", i, a0, a1, z, x);
        let w = match x {
            0 => z % 26 + a0,
            1 => {
                let mut allowed = allowed_z.last().expect("needed to push stack").clone();
                log!("  allowed outgoing top: {}", allowed);
                allowed.subtract(a1);
                log!("  less a1: {}", allowed);
                allowed.restrict_to(&BitSet::from(1..=9));
                log!("  constrained to valid inputs: {}", allowed);
                allowed.remove(z % 26 + a0);
                log!("  without incoming z: {}", allowed);
                if largest {
                    allowed.highest()
                } else {
                    allowed.lowest()
                }
            }
            _ => unreachable!(),
        };
        log!("  Best input: {}", w);
        inputs.push(w);
        if a0 <= 0 {
            z /= 26;
        }
        if x == 1 {
            z = z * 26 + w + a1;
        }
        if z > 0 {
            let mut digits = z;
            for set in allowed_z.iter().rev() {
                log!("    checking stack: {} versus allowed: {}", digits % 26, set);
                assert!(set.contains(digits % 26));
                digits /= 26;
            }
        }
    }

    log!("Final z: {}", z);

    inputs
}

#[aoc_generator(day24, part1, jimb)]
#[aoc_generator(day24, part2, jimb)]
fn generate(_input: &str) -> Problem {
    PART1_INPUT
}

#[aoc(day24, part1, jimb)]
fn part1(rounds: Problem) -> usize {
    let mut xs = vec![0; rounds.len()];
    let mut zs = vec![vec![]; rounds.len() + 1];
    solve_back(14, &mut xs, &mut zs, rounds, &mut |xs, zs| {
        println!("Solution:");
        let input = solve_forward(true, xs, zs, rounds);
        println!("    Highest valid model number: {}",
                 input.iter().map(|&n| char::from_digit(n as u32, 10).unwrap()).collect::<String>());
    });
    0
}

#[aoc(day24, part2, jimb)]
fn part2(rounds: Problem) -> usize {
    let mut xs = vec![0; rounds.len()];
    let mut zs = vec![vec![]; rounds.len() + 1];
    solve_back(14, &mut xs, &mut zs, rounds, &mut |xs, zs| {
        println!("Solution:");
        let input = solve_forward(false, xs, zs, rounds);
        println!("    Lowest valid model number: {}",
                 input.iter().map(|&n| char::from_digit(n as u32, 10).unwrap()).collect::<String>());
    });
    0
}

/// A set of numbers in the range 0..26.
#[derive(Clone, Copy)]
struct BitSet(u32);

impl BitSet {
    fn contains(&self, n: i64) -> bool {
        0 <= n && n < 32 && self.0 & 1 << n != 0
    }

    fn overlaps(&self, range: RangeInclusive<i64>) -> bool {
        self.0 & BitSet::from(range).0 != 0
    }

    fn subtract(&mut self, n: i64) {
        self.0 >>= n;
    }

    fn remove(&mut self, n: i64) {
        if 0 <= n && n < 32 {
            self.0 &= !(1 << n);
        }
    }

    fn restrict_to(&mut self, other: &BitSet) {
        self.0 &= other.0;
    }

    fn highest(&self) -> i64 {
        assert!(self.0 != 0);
        (31 - self.0.leading_zeros()) as _
    }

    fn lowest(&self) -> i64 {
        assert!(self.0 != 0);
        self.0.trailing_zeros() as _
    }
}

impl From<RangeInclusive<i64>> for BitSet {
    fn from(range: RangeInclusive<i64>) -> Self {
        // Convert to exclusive range for bit-shifting, and also so that
        // clamping to non-negative can make the range empty.
        let range = cmp::max(0, *range.start()) .. cmp::max(0, *range.end() + 1);
        BitSet((1 << range.end - range.start) - 1 << range.start)
    }
}

impl From<i64> for BitSet {
    fn from(n: i64) -> Self {
        BitSet::from(n..=n)
    }
}

impl fmt::Display for BitSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{{")?;
        let mut bit = 0;
        while bit < 32 {
            if self.0 & (1 << bit) != 0 {
                let mut end = bit;
                while end < 32 && self.0 & 1 << end != 0 {
                    end += 1;
                }

                if end > bit + 1 {
                    write!(f, " {}..={}", bit, end - 1)?;
                } else {
                    write!(f, " {}", bit)?;
                }

                write!(f, "{}", if self.0 & !((1 << end) - 1) != 0 { "," } else { " " })?;
                bit = end;
            } else {
                bit += 1;
            }
        }
        write!(f, "}}")
    }
}

impl fmt::Debug for BitSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

#[derive(Copy, Clone)]
struct Indent(usize);

impl Indent {
    fn next(self) -> Self {
        Indent(self.0 + 1)
    }
}

impl fmt::Display for Indent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for _ in 0..self.0 {
            write!(f, "  ")?;
        }
        Ok(())
    }
}
