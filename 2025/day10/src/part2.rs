/*! Solve AoC 2025 Day 10 Part 2.

Since the effect of each button is strictly additive, it doesn't
matter what order we press them in, only how many times we press each
one. This means that Part 2 is a linear algebra problem in the
natural numbers: what linear combination of the buttons adds up to
the desired joltages?

Given `b` buttons and `v` joltages, we can treat the buttons as
columns of a `v ✕ b` matrix `B`, and the desired joltages as a
`v`-element vector `j`. We are then searching for a `b`-element vector
`p` of natural numbers representing the number of times each button is
pressed, such that `j = B p`.

But that alone isn't an adequate specification of what we need. In
general, a machine's buttons are not linearly independent, and there
will be multiple solutions. We are searching for a solution that
minimizes the sum of `p`'s elements. Since we only actually care about
the number of button presses, we needn't distinguish between minimal
solutions.

*/

use crate::Machine;

impl Machine {
    pub fn part2(&self) -> u64 {
        todo!()
    }
}
