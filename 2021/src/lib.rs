use aoc_runner_derive::aoc_lib;

pub mod index;
pub mod astar_weighted;

//mod day05;
//mod day06;
//mod day07;
//mod day08;
//mod day09;
//mod day10;
//mod day11;
//mod day12;
//mod day13;
//mod day14;
//mod day15;
//mod day16;
//mod day17;
//mod day18;
//mod day18_sed;
//mod day18_heap;
mod day19;

pub fn cartesian_product<A, B>(a: A, b: B) -> impl Iterator<Item = (A::Item, B::Item)> + Clone
where
    A: IntoIterator,
    B: IntoIterator,
    A::Item: Clone,
    A::IntoIter: Clone,
    B::IntoIter: Clone,
{
    let a = a.into_iter();
    let b = b.into_iter();
    a.flat_map(move |i| b.clone().map(move |j| (i.clone(), j)))
}

/// Like `cartesian_product`, but the inner iterator can depend on the outer.
///
/// For example, to generate unordered pairs:
///
///     triangular_product(0..n, |i| i+1 .. n)
pub fn triangular_product<A, B, F>(a: A, mut make_b: F) -> impl Iterator<Item = (A::Item, B::Item)>
where
    A: IntoIterator,
    A::Item: Clone,
    F: for<'a> FnMut(&'a A::Item) -> B,
    B: IntoIterator,
{
    let a = a.into_iter();
    a.flat_map(move |i| make_b(&i).into_iter().map(move |j| (i.clone(), j)))
}

pub fn conway() -> impl Iterator<Item = (isize, isize)> + Clone {
    cartesian_product(-1..=1, -1..=1).filter(|&(dx, dy)| dx != 0 || dy != 0)
}

pub fn compass() -> impl Iterator<Item = (isize, isize)> + Clone {
    std::iter::successors(Some((1, 0)), |&(x, y)| Some((-y, x))).take(4)
}

pub fn around<I>(p: [usize; 2], bounds: (usize, usize), offsets: I) -> impl Iterator<Item = [usize; 2]> + Clone
    where I: IntoIterator<Item = (isize, isize)>,
          I::IntoIter: Clone,
{
    offsets.into_iter().filter_map(move |(dx, dy)| {
        // We're counting on the 'as usize' to wrap around for negative values.
        let x = (p[0] as isize + dx) as usize;
        let y = (p[1] as isize + dy) as usize;

        if x >= bounds.0 || y >= bounds.1 {
            return None;
        }

        Some([x, y])
    })
}

aoc_lib! { year = 2021 }
