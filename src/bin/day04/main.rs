mod input;
mod parse;

use input::{Board, Input};

fn main() {
    part1(&input::INPUT_1);
    part2(&input::INPUT_1);
}

fn part1(input: &Input) {
    let mut marks: Vec<Board<bool>> = input
        .boards
        .iter()
        .map(|_| Board::<bool>::default())
        .collect();

    for &n in input.draws {
        let mut best = 0;
        for (board, marks) in input.boards.iter().zip(marks.iter_mut()) {
            let score = mark(n, board, marks);
            if score > best {
                best = score;
            }
        }

        if best > 0 {
            println!("Best score: {}", best);
            break;
        }
    }
}

fn part2(input: &Input) {
    let mut marks = vec![Board::<bool>::default(); input.boards.len()];
    let mut scores = vec![0; input.boards.len()];

    let mut last = 0;
    for &n in input.draws {
        for ((board, marks), score) in input.boards.iter().zip(marks.iter_mut()).zip(scores.iter_mut()) {
            if *score == 0 {
                *score = mark(n, board, marks);
                if *score > 0 {
                    last = *score;
                }
            }
        }
    }

    println!("{:?}", scores);
    println!("Last board's score: {}", last);
}

fn mark(n: u32, board: &Board<u32>, marks: &mut Board<bool>) -> u32 {
    let mut marked_any = false;
    for row in 0..5 {
        for col in 0..5 {
            if board[row][col] == n {
                marks[row][col] = true;
                marked_any = true;
            }
        }
    }

    if marked_any {
        for row in 0..5 {
            for col in 0..5 {
                if (0..5).all(|j| marks[row][j]) || (0..5).all(|i| marks[i][col]) {
                    let mut sum = 0;
                    for row in 0..5 {
                        for col in 0..5 {
                            if !marks[row][col] {
                                sum += board[row][col];
                            }
                        }
                    }
                    return sum * n;
                }
            }
        }
    }

    0
}
