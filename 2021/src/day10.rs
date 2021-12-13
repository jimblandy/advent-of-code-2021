use aoc_runner_derive::{aoc, aoc_generator};

#[aoc_generator(day10)]
fn generate(input: &str) -> Vec<String> {
    input.lines().map(|l| l.to_string()).collect()
}

#[cfg(test)]
fn sample() -> Vec<String> {
    generate(include_str!("sample/day10"))
}

fn score(ch: char) -> usize {
    match ch {
        ')' => 3,
        ']' => 57,
        '}' => 1197,
        '>' => 25137,
        _ => panic!("non-closing bracket: {:?}", ch),
    }
}

fn autocomplete_score(ch: char) -> usize {
    match ch {
        ')' => 1,
        ']' => 2,
        '}' => 3,
        '>' => 4,
        _ => panic!("non-closing bracket: {:?}", ch),
    }
}

#[aoc(day10, part1)]
fn part1(input: &Vec<String>) -> usize {
    let mut stack = vec![];
    let mut total = 0;
    for line in input {
        stack.clear();
        for ch in line.chars() {
            match ch {
                '(' => stack.push(')'),
                '[' => stack.push(']'),
                '{' => stack.push('}'),
                '<' => stack.push('>'),
                ')' | ']' | '}' | '>' => {
                    match stack.pop() {
                        Some(m) => {
                            if ch != m {
                                total += score(ch);
                                break;
                            }
                        }
                        None => {
                            // We don't have any instructions about lines that
                            // are over-closed.
                            panic!("too many close brackets");
                        }
                    }
                }
                _ => panic!("character not in alphabet: {:?}", ch),
            }
        }
        // We'll end up here both for incomplete lines and corrupted lines, but
        // only the latter will contribute to the total score.
    }

    total
}

#[test]
fn test_part1() {
    assert_eq!(part1(&sample()), 26397);
}

fn part2_line(line: &str) -> Option<usize> {
    let mut stack = vec![];
    for ch in line.chars() {
        match ch {
            '(' => stack.push(')'),
            '[' => stack.push(']'),
            '{' => stack.push('}'),
            '<' => stack.push('>'),
            ')' | ']' | '}' | '>' => {
                match stack.pop() {
                    Some(m) => {
                        if ch != m {
                            // ignore corrupted lines for part2
                            return None;
                        }
                    }
                    None => panic!("over-closed"),
                }
            }
            _ => panic!("character not in alphabet: {:?}", ch),
        }
    }

    // The line is incomplete.
    let score = stack.into_iter().rev().fold(0, |a, ch| a * 5 + autocomplete_score(ch));
    Some(score)
}

#[test]
fn test_part2_line() {
    assert_eq!(part2_line("[({(<(())[]>[[{[]{<()<>>"),  Some(288957));
    assert_eq!(part2_line("[(()[<>])]({[<{<<[]>>("),    Some(5566));
    assert_eq!(part2_line("(((({<>}<{<{<>}{[]{[]{}"),   Some(1480781));
    assert_eq!(part2_line("{<[[]]>}<{[{[{[]{()[[[]"),   Some(995444));
    assert_eq!(part2_line("<{([{{}}[<[[[<>{}]]]>[]]"),  Some(294));
}

#[aoc(day10, part2)]
fn part2(input: &Vec<String>) -> usize {
    let mut scores: Vec<usize> = input.iter().map(|s| part2_line(s)).flatten().collect();

    let mid = scores.len() / 2;
    *scores.select_nth_unstable(mid).1
}

#[test]
fn test_part2() {
    assert_eq!(part2(&sample()), 288957);
}
