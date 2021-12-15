//! Array indexing utilities.

#![allow(dead_code)]

use ndarray::Array2;
use std::ops;

/// Wrapper for ndarray `Array2` arrays that allows indexing by i32.
///
/// Out-of-bounds reads return OOB. Out-of-bounds writes are ignored.
///
/// This is probably best used via a local wrapper function for brevity:
///
///     # use ndarray::Array2;
///     # use aoc2021::index::Signed;
///     # let arr = Array2::<u32>::zeros((10, 10));
///     # let (i, j) = (0, 0);
///     fn s<T: Default>(array: &Array2<T>) -> Signed<T> {
///         Signed::new(array, T::default())
///     }
///
///     let value = s(&arr)[[i, j]];
///
pub struct Signed<'a, T> {
    array: &'a Array2<T>,
    def: T,
}

impl<'a, T> Signed<'a, T> {
    pub fn new(array: &'a Array2<T>, def: T) -> Self {
        Signed { array, def }
    }

    fn in_bounds(&self, index: [i32; 2]) -> bool {
        (index[0] as usize) < self.array.nrows() &&
            (index[1] as usize) < self.array.ncols()
    }
}

impl<'a, T> ops::Index<[i32; 2]> for Signed<'a, T> {
    type Output = T;

    fn index(&self, index: [i32; 2]) -> &Self::Output {
        if self.in_bounds(index) {
            &self.array[[index[0] as usize, index[1] as usize]]
        } else {
            &self.def
        }
    }
}

/*
impl<'a, T> ops::IndexMut<[i32; 2]> for Signed<'a, T> {
    fn index_mut(&mut self, index: [i32; 2]) -> &mut Self::Output {
        if self.in_bounds(index) {
            &mut self.array[[index[0] as usize, index[1] as usize]]
        } else {
            &mut self.def
        }
    }
}
*/
