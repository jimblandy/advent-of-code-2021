mod bands;
mod render;
mod ranges_iter;

use std::io::Write as _;
use std::ops::RangeInclusive;

type Point = (u64, u64); // row, col
type Edge = RangeInclusive<Point>;

fn is_vertical(edge: &Edge) -> bool {
    edge.start().0 != edge.end().0 && edge.start().1 == edge.end().1
}

fn is_horizontal(edge: &Edge) -> bool {
    edge.start().0 == edge.end().0 && edge.start().1 != edge.end().1
}

struct Problem {
    red: Vec<Point>,
}

impl Problem {
    fn from_str(input: &str) -> Self {
        Problem {
            red: input
                .lines()
                .map(|line| {
                    let mut coords = line.split(',').map(|coord| coord.parse().unwrap());
                    (coords.next().unwrap(), coords.next().unwrap())
                })
                .collect(),
        }
    }

    fn edges(&self) -> impl Iterator<Item = Edge> + '_ {
        let backlink = *self.red.last().unwrap()..=self.red[0];
        self.red
            .windows(2)
            .map(|w| {
                let &[from, to] = w else { unreachable!() };
                from..=to
            })
            .chain(std::iter::once(backlink))
            .inspect(|edge| {
                assert!(is_horizontal(edge) || is_vertical(edge));
                assert!(edge.start() != edge.end());
            })
    }
}

fn area(a: Point, b: Point) -> u64 {
    use std::cmp::{max, min};

    let ul = (min(a.0, b.0), min(a.1, b.1));
    let lr = (max(a.0, b.0), max(a.1, b.1));
    (lr.0 + 1 - ul.0) * (lr.1 + 1 - ul.1)
}

fn part1(problem: &Problem) -> u64 {
    problem
        .red
        .iter()
        .enumerate()
        .flat_map(|(i, &a)| problem.red[..i].iter().map(move |&b| (a, b)))
        .map(|(a, b)| area(a, b))
        .max()
        .unwrap()
}

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let problem = Problem::from_str(include_str!("input.txt"));
    println!("part 1: {}", part1(&problem));
    render(&problem, (1000, 1000), 100, "day9.png".as_ref())?;
    Ok(())
}

fn render(
    problem: &Problem,
    size: (usize, usize),
    scale: usize,
    output: &std::path::Path,
) -> anyhow::Result<()> {
    let bands = bands::BandIter::from_edges(problem.edges());
    let mut target = render::RenderTarget::new(size, scale);
    for band in bands {
        target.render_band(&band);
    }
    let image = target.into_image();

    let stream = std::fs::File::create(output)?;
    let mut stream = std::io::BufWriter::new(stream);
    image.write_to(&mut stream, image::ImageFormat::Png)?;
    stream.flush()?;
    Ok(())
}
