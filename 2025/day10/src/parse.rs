use std::str::FromStr;

use crate::{Machine, Problem};
use anyhow::bail;

impl FromStr for Machine {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let Some(rest) = s.strip_prefix('[') else {
            bail!("Missing '['");
        };
        let Some((lights, rest)) = rest.split_once(']') else {
            bail!("Missing ']'");
        };
        let lights = lights
            .chars()
            .rev()
            .map(|ch| match ch {
                '#' => 1,
                '.' => 0,
                _ => panic!("Bad light character: {ch:?}"),
            })
            .fold(0, |acc, bit| acc << 1 | bit);
        let Some((buttons, rest)) = rest.split_once('{') else {
            bail!("Missing '{{'");
        };
        let buttons: Vec<u64> = buttons
            .split_whitespace()
            .map(|button| {
                let Some(rest) = button.strip_prefix('(') else {
                    panic!("Button doesn't start with '('");
                };
                let Some(lights) = rest.strip_suffix(')') else {
                    panic!("Button doesn't end with ')'");
                };
                comma_list::<u32>(lights).fold(0, |acc, num| acc | (1 << num))
            })
            .collect();
        let mut buttons_by_size = buttons.clone();
        buttons_by_size.sort_by_key(|&button| button.count_ones());
        buttons_by_size.reverse();
        let Some(joltages) = rest.trim_end().strip_suffix('}') else {
            bail!("Missing '}}'");
        };
        let joltages = comma_list::<u64>(joltages).collect();
        Ok(Machine {
            lights,
            buttons,
            buttons_by_size,
            joltages,
        })
    }
}

fn comma_list<T>(s: &str) -> impl Iterator<Item = T>
where
    T: FromStr,
    T::Err: std::fmt::Display,
{
    s.split(',').map(|num| {
        T::from_str(num).unwrap_or_else(|err| panic!("Bad comma list element {num:?}: {err}"))
    })
}

impl FromStr for Problem {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let machines = s
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(|line| Machine::from_str(line))
            .collect::<Result<_, _>>()?;
        Ok(Problem { machines })
    }
}
