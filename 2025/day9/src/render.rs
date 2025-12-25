use crate::bands::Band;

use std::cmp;
use std::iter::Extend;
use std::ops::Range;

/// An object that accepts a series of [`Band`]s and renders them to a
/// color bitmap.
///
/// The background is drawn black; areas within the bands are drawn in
/// gray; and red tiles, obviously, are red.
///
/// The size of the bitmap produced by `RenderTarget` is chosen at
/// construction time, along with a scale factor that relates "band
/// space" (the 'floor tile' coordinate system in which `Band` rows
/// and columns are interpreted) to "output space" (rows and columns
/// in the output bitmap). For example, if the output bitmap is
/// 120x100 and the scale factor is 10, then the band space covered by
/// the bitmap is 1200x1000 tiles.
///
/// Bands must be presented from top to bottom.
pub struct RenderTarget {
    /// The dimensions of our final output buffer: (rows, columns).
    size: (usize, usize),

    /// The output pixel buffer.
    ///
    /// The length of this vector is `size.0 * size.1`. Pixels appear in
    /// row-major order.
    pixels: Vec<u8>,

    /// The factor by which we scale down bands before rendering.
    ///
    /// Every pixel in the output buffer covers a square of floor tiles whose
    /// sides are `scale` tiles long.
    scale: usize,

    /// The row of `pixels` we are currently buffering.
    next_row: usize,

    /// For each pixel in `next_row`, the number of floor tiles that fall within
    /// the shape.
    covered_tiles: Vec<u64>,

    /// For each pixel in `next_row`, whether it contains any red floor tiles.
    red_tiles: Vec<bool>,
}

impl RenderTarget {
    pub fn new(size: (usize, usize), scale: usize) -> Self {
        Self {
            size,
            pixels: Vec::with_capacity(size.0 * size.1),
            scale,
            next_row: 0,
            covered_tiles: vec![0; size.1],
            red_tiles: vec![false; size.1],
        }
    }

    pub fn render_band(&mut self, band: &Band) {
        let scale = self.scale as u64;
        let rows = *band.rows.start()..*band.rows.end() + 1;
        let top_pixel = rows.start / scale;
        assert!(self.next_row <= top_pixel as usize);

        // Split the band's range of rows into subranges, one for each row of
        // pixels it touches, and handle one pixel row at a time.
        for (rows, pixel_row) in divide_range(rows, scale).zip(top_pixel..) {
            // Advance rendering to the top of this pixel row.
            while (self.next_row as u64) < pixel_row {
                self.flush_row();
            }

            if band.rows.start() / scale == pixel_row {
                for red in &band.reds {
                    self.red_tiles[(red / scale) as usize] = true;
                }
            }

            let height = rows.end - rows.start;
            self.render_runs(height, &band.runs);
        }
    }

    pub fn into_image(mut self) -> image::RgbImage {
        while self.next_row < self.size.0 {
            self.flush_row();
        }

        let width = self.size.1 as u32;
        let height = self.size.0 as u32;
        image::RgbImage::from_vec(width, height, self.pixels).unwrap()
    }

    /// Render a band's covered area for a single row of pixels.
    ///
    /// `height` must be the number of floor tile rows the band overlaps within
    /// the current row of pixels.
    fn render_runs(&mut self, height: u64, runs: &[Range<u64>]) {
        let scale = self.scale as u64;
        for run in runs.iter().cloned() {
            let pixel_col = run.start / scale;
            let covered_tiles = &mut self.covered_tiles[pixel_col as usize..];
            // Split `run` into subranges for each pixel it covers, and add their area
            // to their pixel.
            for (subrun, pixel) in divide_range(run, scale).zip(covered_tiles.iter_mut()) {
                let width = subrun.end - subrun.start;
                let area = width.checked_mul(height).unwrap();
                *pixel = (*pixel).saturating_add(area);
            }
        }
    }

    /// Turn the `covered_tiles` and `red_tiles` buffers into actual pixels, and
    /// prepare for processing the next row.
    fn flush_row(&mut self) {
        assert_eq!(self.pixels.len(), self.next_row * self.size.1 * 3);
        let tiles_per_pixel = u64::try_from(self.scale * self.scale).unwrap();
        self.pixels
            .extend(
                self.covered_tiles
                    .iter()
                    .zip(&self.red_tiles)
                    .flat_map(|(&covered, &red)| {
                        if red {
                            [255, 20, 20]
                        } else {
                            let level = covered * 128 / tiles_per_pixel;
                            let component = u8::try_from(level).unwrap();
                            [component, component, component]
                        }
                    }),
            );
        self.next_row += 1;
        assert_eq!(self.pixels.len(), self.next_row * self.size.1 * 3);

        self.covered_tiles[..].fill(0);
        self.red_tiles[..].fill(false);
    }
}

/// Split `range` up into a series of ranges, splitting at each multiple of `scale`.
///
/// Examples:
///
/// - `divide_range(5..35, 10)` splits the range at each multiple of `10`,
///   producing the ranges `5..10`, `10..20`, `20..30`, and `30..35`.
///
/// - `divide_range(5..7, 10)` produces only the range `5..7`.
///
/// - `divide_range(9..20, 10)` produces the ranges `9..10` and `10..20`.
fn divide_range(range: Range<u64>, scale: u64) -> impl Iterator<Item = Range<u64>> {
    let mut start = range.start - range.start % scale;
    std::iter::from_fn(move || {
        let next = start + scale;
        let chunk = start..next;
        start = next;
        intersection(&chunk, &range)
    })
}

#[test]
fn test_divide_range() {
    assert_eq!(
        divide_range(5..35, 10).collect::<Vec<_>>(),
        vec![5..10, 10..20, 20..30, 30..35]
    );
    assert_eq!(divide_range(5..7, 10).collect::<Vec<_>>(), vec![5..7]);
    assert_eq!(
        divide_range(9..20, 10).collect::<Vec<_>>(),
        vec![9..10, 10..20]
    );
}

fn intersection<T: cmp::Ord + Copy>(a: &Range<T>, b: &Range<T>) -> Option<Range<T>> {
    let candidate = cmp::max(a.start, b.start)..cmp::min(a.end, b.end);
    (candidate.start < candidate.end).then_some(candidate)
}
