use aoc_runner_derive::{aoc, aoc_generator};
use anyhow::{anyhow, Result};

trait Bits {
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool;
    fn get(&mut self, n: usize) -> u64;

    fn by_ref(&mut self) -> &mut Self
        where Self: Sized
    {
        self
    }

    fn take(self, n: usize) -> Take<Self>
        where Self: Sized
    {
        Take {
            inner: self,
            remaining: n
        }
    }
}

impl<'a, B> Bits for &'a mut B
    where B: Bits + ?Sized
{
    #[inline]
    fn len(&self) -> usize {
        (**self).len()
    }

    #[inline]
    fn is_empty(&self) -> bool {
        (**self).is_empty()
    }

    #[inline]
    fn get(&mut self, n: usize) -> u64 {
        (**self).get(n)
    }
}

#[derive(Clone)]
struct Words {
    words: Vec<u64>,
    current: u64,
    next_word: usize,
    remaining_bits: usize,
}

#[inline]
fn mask(n: usize) -> u64 {
    (1 << n) - 1
}

#[inline]
fn insert(a: u64, n: usize, b: u64) -> u64 {
    a << n | b & mask(n)
}

impl Bits for Words {
    #[inline]
    fn len(&self) -> usize {
        (self.words.len() - self.next_word) * 64 + self.remaining_bits
    }

    #[inline]
    fn is_empty(&self) -> bool {
        self.remaining_bits == 0 && self.next_word >= self.words.len()
    }

    #[inline]
    fn get(&mut self, n: usize) -> u64 {
        let unmasked = if n <= self.remaining_bits {
            self.remaining_bits -= n;
            self.current >> self.remaining_bits
        } else {
            let rest = n - self.remaining_bits;
            let top = self.current << rest;
            self.current = self.words[self.next_word];
            self.next_word += 1;
            self.remaining_bits = 64 - rest;
            top | self.current >> self.remaining_bits
        };

        unmasked & mask(n)
    }
}

impl Words {
    fn new(bytes: Vec<u8>) -> Self {
        let words = bytes.chunks(8)
            .map(|chunk| {
                let missing = 8 - chunk.len();
                if missing == 0 {
                    u64::from_be_bytes(chunk.try_into().unwrap())
                } else {
                    chunk.iter().fold(0, |n, &b| n << 8 | b as u64)
                        << 8 * missing
                }
            })
            .collect::<Vec<u64>>();

        if words.is_empty() {
            Words {
                words,
                current: 0,
                next_word: 1,
                remaining_bits: 0
            }
        } else {
            let current = words[0];
            Words {
                words,
                current,
                next_word: 1,
                remaining_bits: 64,
            }
        }
    }

    fn from_hex(hex: &str) -> Result<Self> {
        assert!(hex.len() & 1 == 0);
        let mut bytes = vec![];
        for pair in hex.as_bytes().chunks_exact(2) {
            let s = std::str::from_utf8(pair)
                .map_err(|_| anyhow!("pair of bytes not valid UTF-8: {:?}", hex))?;
            let byte = u8::from_str_radix(s, 16)
                .map_err(|_| anyhow!("pair of bytes not valid hex: {:?}", s))?;
            bytes.push(byte)
        }

        Ok(Words::new(bytes))
    }
}

#[test]
fn test_words() {
    let mut b = Words::new(vec![0xd2, 0xfe, 0x28]);
    assert_eq!(b.get(3), 6);
    assert_eq!(b.get(3), 4);
    assert_eq!(b.get(5), 0b10111);
    assert_eq!(b.get(5), 0b11110);
    assert_eq!(b.get(5), 0b00101);
    assert_eq!(b.get(43), 0);
    assert!(b.is_empty());
}

struct Take<B> {
    inner: B,
    remaining: usize,
}

impl<B: Bits> Bits for Take<B> {
    fn len(&self) -> usize {
        self.remaining
    }

    fn is_empty(&self) -> bool {
        self.remaining == 0
    }

    fn get(&mut self, n: usize) -> u64 {
        assert!(n <= self.remaining);
        self.remaining -= n;
        self.inner.get(n)
    }
}

#[test]
fn test_take() {
    let mut b = Words::from_hex("38006F45291200").unwrap();

    assert_eq!(b.get(3), 1);
    assert_eq!(b.get(3), 6);
    assert_eq!(b.get(1), 0);
    assert_eq!(b.get(15), 27);

    {
        let mut s = b.by_ref().take(27);
        assert_eq!(s.get(3), 6);
        assert_eq!(s.get(3), 4);
        assert_eq!(s.get(5), 10);
        // at 11

        assert_eq!(s.get(3), 2);
        assert_eq!(s.get(3), 4);
        assert_eq!(s.get(5), 0b10001);
        assert_eq!(s.get(5), 0b00100);
        // at 27
        assert!(s.is_empty());
    }

    assert_eq!(b.get(7), 0);
}

fn add_version_numbers(bits: &mut dyn Bits) -> u64 {
    let version = bits.get(3);
    let ty = bits.get(3);
    let mut total = version;
    match ty {
        LITERAL => {
            let mut value = 0;
            loop {
                let chunk = bits.get(5);
                value = insert(value, 4, chunk);
                if chunk & 0b10000 == 0 {
                    break;
                }
            }
            let _ = value;
        }
        _operator => {
            let length_type = bits.get(1);
            if length_type == 0 {
                let bit_length = bits.get(15) as usize;
                let mut sub = bits.take(bit_length);
                while !sub.is_empty() {
                    total += add_version_numbers(&mut sub);
                }
            } else {
                let packet_count = bits.get(11);
                for _ in 0..packet_count {
                    total += add_version_numbers(bits);
                }
            }
        }
    };

    total
}

#[test]
fn test_add_version_numbers() {
    fn v(s: &str) -> u64 {
        let mut words = Words::from_hex(s).unwrap();
        part1(&mut words)
    }

    assert_eq!(v("38006F45291200"), 9);
    assert_eq!(v("EE00D40C823060"), 14);
    assert_eq!(v("8A004A801A8002F478"), 16);
    assert_eq!(v("620080001611562C8802118E34"), 12);
    assert_eq!(v("C0015000016115A2E0802F182340"), 23);
    assert_eq!(v("A0016C880162017C3686B18A3D4780"), 31);
}

#[aoc_generator(day16, part1, jimb)]
#[aoc_generator(day16, part1, jimb_faster)]
#[aoc_generator(day16, part2, jimb)]
#[aoc_generator(day16, part2, jimb_faster)]
fn generate(input: &str) -> Result<Words> {
    Words::from_hex(input)
}

#[aoc(day16, part1, jimb)]
fn part1(input: &Words) -> u64 {
    let mut words = input.clone();
    add_version_numbers(&mut words)
}

fn add_version_numbers_faster(bits: &mut Words) -> u64 {
    let version = bits.get(3);
    let ty = bits.get(3);
    let mut total = version;
    match ty {
        LITERAL => {
            let mut value = 0;
            loop {
                let chunk = bits.get(5);
                value = insert(value, 4, chunk);
                if chunk & 0b10000 == 0 {
                    break;
                }
            }
            let _ = value;
        }
        _operator => {
            let length_type = bits.get(1);
            if length_type == 0 {
                let bit_length = bits.get(15) as usize;
                let following = bits.len() - bit_length;
                while bits.len() > following {
                    total += add_version_numbers(bits);
                }
            } else {
                let packet_count = bits.get(11);
                for _ in 0..packet_count {
                    total += add_version_numbers(bits);
                }
            }
        }
    };

    total
}

#[aoc(day16, part1, jimb_faster)]
fn part1_faster(input: &Words) -> u64 {
    let mut words = input.clone();
    add_version_numbers_faster(&mut words)
}

const LITERAL: u64 = 4;
const SUM: u64 = 0;
const PRODUCT: u64 = 1;
const MIN: u64 = 2;
const MAX: u64 = 3;
const GT: u64 = 5;
const LT: u64 = 6;
const EQ: u64 = 7;

fn evaluate(bits: &mut dyn Bits) -> u64 {
    let _version = bits.get(3);
    let ty = bits.get(3);

    match ty {
        LITERAL => {
            let mut value = 0;
            loop {
                let chunk = bits.get(5);
                value = insert(value, 4, chunk);
                if chunk & 0b10000 == 0 {
                    break;
                }
            }
            value
        }
        operator => {
            let mut operands = vec![];
            let length_type = bits.get(1);
            if length_type == 0 {
                let bit_length = bits.get(15) as usize;
                let mut sub = bits.take(bit_length);
                while !sub.is_empty() {
                    operands.push(evaluate(&mut sub));
                }
            } else {
                let packet_count = bits.get(11);
                for _ in 0..packet_count {
                    operands.push(evaluate(bits));
                }
            };

            match operator {
                SUM => operands.iter().sum(),
                PRODUCT => operands.iter().product(),
                MIN => *operands.iter().min().unwrap(),
                MAX => *operands.iter().max().unwrap(),
                GT => (operands[0] > operands[1]) as u64,
                LT => (operands[0] < operands[1]) as u64,
                EQ => (operands[0] == operands[1]) as u64,
                _ => panic!("Unrecognized opcode: {}", operator),
            }
        }
    }
}

#[test]
fn test_evaluate() {
    fn e(input: &str) -> u64 {
        let mut words = Words::from_hex(input).unwrap();
        evaluate(&mut words)
    }

    assert_eq!(e("C200B40A82"), 3);
    assert_eq!(e("04005AC33890"), 54);
    assert_eq!(e("880086C3E88112"), 7);
    assert_eq!(e("CE00C43D881120"), 9);
    assert_eq!(e("D8005AC2A8F0"), 1);
    assert_eq!(e("F600BC2D8F"), 0);
    assert_eq!(e("9C005AC2F8F0"), 0);
    assert_eq!(e("9C0141080250320F1802104A08"), 1);
}

#[aoc(day16, part2, jimb)]
fn part2(input: &Words) -> u64 {
    let mut words = input.clone();
    evaluate(&mut words)
}

fn evaluate_faster(stack: &mut Vec<u64>, bits: &mut Words) -> u64 {
    let _version = bits.get(3);
    let ty = bits.get(3);

    match ty {
        LITERAL => {
            let mut value = 0;
            loop {
                let chunk = bits.get(5);
                value = insert(value, 4, chunk);
                if chunk & 0b10000 == 0 {
                    break;
                }
            }
            value
        }
        operator => {
            let frame_ptr = stack.len();
            let length_type = bits.get(1);
            if length_type == 0 {
                let bit_length = bits.get(15) as usize;
                let following = bits.len() - bit_length;
                while bits.len() > following {
                    let value = evaluate_faster(stack, bits);
                    stack.push(value);
                }
            } else {
                let packet_count = bits.get(11);
                for _ in 0..packet_count {
                    let value = evaluate_faster(stack, bits);
                    stack.push(value);
                }
            };

            let operands = &stack[frame_ptr..];
            let result = match operator {
                SUM => operands.iter().sum(),
                PRODUCT => operands.iter().product(),
                MIN => *operands.iter().min().unwrap(),
                MAX => *operands.iter().max().unwrap(),
                GT => (operands[0] > operands[1]) as u64,
                LT => (operands[0] < operands[1]) as u64,
                EQ => (operands[0] == operands[1]) as u64,
                _ => panic!("Unrecognized opcode: {}", operator),
            };

            stack.truncate(frame_ptr);

            result
        }
    }
}

#[aoc(day16, part2, jimb_faster)]
fn part2_faster(input: &Words) -> u64 {
    let mut words = input.clone();
    evaluate_faster(&mut Vec::with_capacity(1000), &mut words)
}
