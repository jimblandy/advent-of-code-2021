use aoc_runner_derive::{aoc, aoc_generator};
use anyhow::Result;
use crate::cartesian_product;
use std::ops::Range;
use std::fmt::Write;
use std::mem::swap;
use std::str::FromStr;

#[aoc_generator(day18, part1, jimb_sed)]
#[aoc_generator(day18, part2, jimb_sed)]
fn generator(input: &str) -> Vec<String> {
    input.lines().map(|s| s.to_owned()).collect()
}

fn explode(num: &str, out: &mut String) -> bool {
    out.clear();

    let mut last_n: Option<Range<usize>> = None;
    let mut depth = 0;
    for (start, ch) in num.char_indices() {
        match ch {
            '[' => {
                depth += 1;
                if depth > 4 {
                    let (left, right, pair_len) = get_num_pair(&num[start..]);
                    if let Some(ref range) = last_n {
                        let value = u64::from_str(&num[range.clone()]).unwrap();
                        write!(out, "{}{}{}",
                               &num[..range.start],
                               value + left,
                               &num[range.end..start])
                            .unwrap();
                    } else {
                        out.push_str(&num[..start]);
                    }
                    out.push_str("0");
                    match num[start + pair_len..].find(|ch: char| ch.is_digit(10)) {
                        Some(right_start) => {
                            let (n, right_len) = get_num(&num[start + pair_len + right_start..]);
                            write!(out, "{}{}{}",
                                   &num[start + pair_len .. start + pair_len + right_start],
                                   right + n,
                                   &num[start + pair_len + right_start + right_len..])
                                .unwrap();
                        }
                        None => {
                            out.push_str(&num[start + pair_len..]);
                        }
                    }

                    return true;
                }
            },
            ']' => depth -= 1,
            '0'..='9' => {
                match last_n {
                    None => {
                        last_n = Some(start .. start + 1);
                    }
                    Some(ref mut range) => {
                        if range.end == start {
                            range.end = start + 1;
                        } else {
                            last_n = Some(start .. start + 1);
                        }
                    }
                }
            }
            ',' => {}
            _ => panic!("unexpected character: {:?}", ch),
        }
    }

    false
}

fn get_num_pair(s: &str) -> (u64, u64, usize) {
    let bytes = s.as_bytes();
    assert_eq!(bytes[0], b'[');
    let mut pos = 1;
    let mut left = 0;
    while let Some(digit) = (bytes[pos] as char).to_digit(10) {
        left = left * 10 + digit as u64;
        pos += 1;
    }
    assert_eq!(bytes[pos], b',');
    pos += 1;
    let mut right = 0;
    while let Some(digit) = (bytes[pos] as char).to_digit(10) {
        right = right * 10 + digit as u64;
        pos += 1;
    }
    assert_eq!(bytes[pos], b']');
    (left, right, pos + 1)
}

fn get_num(s: &str) -> (u64, usize) {
    let bytes = s.as_bytes();
    let mut pos = 0;
    let mut n = 0;
    while let Some(digit) = (bytes[pos] as char).to_digit(10) {
        n = n * 10 + digit as u64;
        pos += 1;
    }
    (n, pos)
}

#[test]
fn test_explode() {
    let mut out = String::new();

    assert!(explode("[[[[[9,8],1],2],3],4]", &mut out));
    assert_eq!(out, "[[[[0,9],2],3],4]");

    assert!(explode("[7,[6,[5,[4,[3,2]]]]]", &mut out));
    assert_eq!(out, "[7,[6,[5,[7,0]]]]");

    assert!(explode("[[6,[5,[4,[3,2]]]],1]", &mut out));
    assert_eq!(out, "[[6,[5,[7,0]]],3]");

    assert!(explode("[[3,[2,[1,[7,3]]]],[6,[5,[4,[3,2]]]]]", &mut out));
    assert_eq!(out, "[[3,[2,[8,0]]],[9,[5,[4,[3,2]]]]]");

    assert!(explode("[[3,[2,[8,0]]],[9,[5,[4,[3,2]]]]]", &mut out));
    assert_eq!(out, "[[3,[2,[8,0]]],[9,[5,[7,0]]]]");
}

fn split(num: &str, out: &mut String) -> bool {
    out.clear();

    let mut rest = 0;
    while let Some(pos) = num[rest..].find(|ch: char| ch.is_digit(10)) {
        let (n, len) = get_num(&num[rest + pos..]);

        if n >= 10 {
            write!(out, "{}[{},{}]{}",
                   &num[..rest + pos],
                   n / 2,
                   n - n / 2,
                   &num[rest + pos + len..])
                .unwrap();
            return true;
        }

        rest += pos + len;
    }

    false
}

#[test]
fn test_split() {
    let mut out = String::new();

    assert!(split("[[[[0,7],4],[15,[0,13]]],[1,1]]", &mut out));
    assert_eq!(out, "[[[[0,7],4],[[7,8],[0,13]]],[1,1]]");

    assert!(split("[[[[0,7],4],[[7,8],[0,13]]],[1,1]]", &mut out));
    assert_eq!(out, "[[[[0,7],4],[[7,8],[0,[6,7]]]],[1,1]]");
}

fn reduce(num: &mut String, temp: &mut String) {
    loop {
        if explode(num, temp) {
            swap(num, temp);
            continue;
        }

        if split(num, temp) {
            swap(num, temp);
            continue;
        }

        break;
    }
}

#[test]
fn test_reduce() {
    let mut num = "[[[[[4,3],4],4],[7,[[8,4],9]]],[1,1]]".to_string();
    let mut temp = String::new();

    reduce(&mut num, &mut temp);
    assert_eq!(num, "[[[[0,7],4],[[7,8],[6,0]]],[8,1]]");
}

fn sum_list<'i, I, T>(list: I, out: &mut String, temp: &mut String)
    where I: IntoIterator<Item = &'i T> + 'i,
          T: AsRef<str> + 'i,
          T: ?Sized,
{
    let mut list = list.into_iter();
    let mut left = list.next().unwrap().as_ref();
    for right in list {
        temp.clear();
        write!(temp, "[{},{}]", left, right.as_ref()).unwrap();
        reduce(temp, out);
        swap(temp, out);
        left = &out;
    }
}

#[test]
fn test_sum_list() {
    let mut out = String::new();
    let mut temp = String::new();
    sum_list(&["[1,1]",
               "[2,2]",
               "[3,3]",
               "[4,4]"],
             &mut out,
             &mut temp);
    assert_eq!(out, "[[[[1,1],[2,2]],[3,3]],[4,4]]");

    sum_list(&["[[[0,[4,5]],[0,0]],[[[4,5],[2,6]],[9,5]]]",
               "[7,[[[3,7],[4,3]],[[6,3],[8,8]]]]",
               "[[2,[[0,8],[3,4]]],[[[6,7],1],[7,[1,6]]]]",
               "[[[[2,4],7],[6,[0,5]]],[[[6,8],[2,8]],[[2,1],[4,5]]]]",
               "[7,[5,[[3,8],[1,4]]]]",
               "[[2,[2,2]],[8,[8,1]]]",
               "[2,9]",
               "[1,[[[9,3],9],[[9,0],[0,7]]]]",
               "[[[5,[7,4]],7],1]",
               "[[[[4,2],2],6],[8,7]]"],
             &mut out,
             &mut temp);
    assert_eq!(out, "[[[[8,7],[7,7]],[[8,6],[7,7]]],[[[0,7],[6,6]],[8,7]]]");
}

fn magnitude(num: &str) -> u64 {
    let mut stack = vec![];
    let mut n = 0;
    for ch in num.chars() {
        match ch {
            '0'..='9' => {
                n = n * 10 + ch.to_digit(10).unwrap() as u64;
            }
            '[' => {},
            ',' => {
                stack.push(n);
                n = 0;
            },
            ']' => {
                let left = stack.pop().unwrap();
                n = 3 * left + 2 * n;
            }
            _ => panic!("unexpected character {:?}", ch),
        }
    }
    n
}

#[test]
fn test_magnitude() {
    assert_eq!(magnitude("[9,1]"), 29);
    assert_eq!(magnitude("[[9,1],[1,9]]"), 129);

    assert_eq!(magnitude("[[1,2],[[3,4],5]]"), 143);
    assert_eq!(magnitude("[[[[0,7],4],[[7,8],[6,0]]],[8,1]]"), 1384);
    assert_eq!(magnitude("[[[[1,1],[2,2]],[3,3]],[4,4]]"), 445);
    assert_eq!(magnitude("[[[[3,0],[5,3]],[4,4]],[5,5]]"), 791);
    assert_eq!(magnitude("[[[[5,0],[7,4]],[5,5]],[6,6]]"), 1137);
    assert_eq!(magnitude("[[[[8,7],[7,7]],[[8,6],[7,7]]],[[[0,7],[6,6]],[8,7]]]"), 3488);
}

#[test]
fn test_homework() {
    let mut out = String::new();
    let mut temp = String::new();
    sum_list(include_str!("sample/day18.homework").lines(),
             &mut out, &mut temp);
    assert_eq!(magnitude(&out), 4140);
}

#[aoc(day18, part1, jimb_sed)]
fn part1(input: &Vec<String>) -> u64 {
    let mut out = String::new();
    let mut temp = String::new();
    sum_list(input, &mut out, &mut temp);
    magnitude(&out)

}

#[aoc(day18, part2, jimb_sed)]
fn part2(input: &Vec<String>) -> u64 {
    let mut sum = String::new();
    let mut temp = String::new();

    cartesian_product(0..input.len(), 0..input.len())
        .filter(|(i, j)| i != j)
        .map(|(i, j)| {
            sum.clear();
            write!(sum, "[{},{}]", &input[i], &input[j]).unwrap();
            reduce(&mut sum, &mut temp);
            magnitude(&sum)
        })
        .max()
        .unwrap()
}
