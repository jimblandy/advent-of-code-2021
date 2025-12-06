#![allow(unused_variables, dead_code)]

mod input;

use std::str::FromStr as _;

struct Problem {
    numbers: Vec<Vec<u64>>,
    operations: Vec<char>,
}

impl Problem {
    fn test_input() -> Self {
        Problem {
            numbers: vec![
                vec![123, 328, 51, 64],
                vec![45, 64, 387, 23],
                vec![6, 98, 215, 314],
            ],
            operations: vec!['*', '+', '*', '+'],
        }
    }
}

static TEST_INPUT_STRING: &str = "\
123 328  51 64 
 45 64  387 23 
  6 98  215 314
*   +   *   +  
";

fn char_to_fn(op: char) -> fn(u64, u64) -> u64 {
    match op {
        '+' => <u64 as std::ops::Add<u64>>::add,
        '*' => <u64 as std::ops::Mul<u64>>::mul,
        _ => unreachable!("unexpected op: {:?}", op),
    }
}

fn part1(problem: &Problem) -> u64 {
    problem
        .operations
        .iter()
        .cloned()
        .enumerate()
        .map(|(col, op)| {
            let op = char_to_fn(op);
            let first = problem.numbers[0][col];
            problem.numbers[1..]
                .iter()
                .map(|row| row[col])
                .fold(first, op)
        })
        .sum()
}

#[test]
fn test_part1() {
    assert_eq!(part1(&Problem::test_input()), 4277556);
}

fn part2(problem: &str) -> u64 {
    // Turn the input text into a grid of characters, for easy indexing.
    let grid: Vec<Vec<char>> = problem.lines().map(|line| line.chars().collect()).collect();

    // Split off the numbers from bottom line containing the operators.
    // We can unwrap because we know there's at least one line.
    let (operators, numbers) = grid.split_last().unwrap();

    // Scan the operator line, producing column numbers and operator functions.
    let operator_columns = operators
        .iter()
        .cloned()
        .enumerate()
        .filter(|&(_column, op)| op != ' ')
        .map(|(column, op)| (column, char_to_fn(op)));

    // Produce the results for each column.
    let column_results = operator_columns
        .map(|(op_column, operator)| {
            // Produce the operands to this operator, considering columns from
            // left to right until we reach a column containing only spaces.
            let operands =
                (op_column..)
                .map(|column| {
                    // Produce the characters in `column`, substituting spaces
                    // if we go off the end of the line.
                    let column_chars = numbers
                        .iter()
                        .map(|row| row.get(column).cloned().unwrap_or(' '));
                    // Turn the column into a string, squeezing out spaces. This
                    // means that the blank column that separates problems turns
                    // into an empty string.
                    column_chars
                        .filter(|&ch| ch != ' ')
                        .collect::<String>()
                })
                // When we reach an empty column, that's the end of the operands
                // for this operator.
                .take_while(|operand_str| !operand_str.is_empty())
                // Parse the column of digits as a number. We can unwrap because
                // we know they're all digits.
                .map(|s| u64::from_str(&s).unwrap());

            // Apply `operator` to all the operands. We can unwrap because we
            // know there's at least one operand.
            operands.reduce(operator).unwrap()
        });

    column_results.sum()
}

#[test]
fn test_part2() {
    assert_eq!(part2(TEST_INPUT_STRING), 3263827);
}

fn main() {
    println!("part 1: {}", part1(&Problem::part1_input()));
    println!("part 2: {}", part2(include_str!("input.txt")));
}
