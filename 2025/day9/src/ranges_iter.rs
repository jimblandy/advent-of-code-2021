/*! [`RangesIter`]: An iterator over possibly-overlapping ranges. */

use binary_heap_plus::{BinaryHeap, PeekMut};
use std::ops::Range;

/// A type that can extract a [`std::ops::Range`] from a `T` value.
pub trait RangeGetter<T> {
    /// The type of the obtained ranges' endpoints.
    type End;
    fn range_of(&self, value: &T) -> Range<Self::End>;
}

/// A type for iterating over overlapping ranges.
///
/// A `RangesIter` holds a collection of values of some type `T`, each of which
/// has some [`Range`] associated with it. The elements' ranges may overlap. The
/// `RangesIter` iterator produces a sorted series of non-overlapping ranges
/// that covers all the elements, but where no element starts or ends in the
/// midst of a produced range. Each produced range is accompanied by the set of
/// values whose ranges overlap it.
///
/// For example, suppose we have a `RangesIter` holding the following items with
/// the given ranges:
///
/// - A: (20,30)
/// - B: (5, 10)
/// - C: (15, 25)
/// - D: (15, 20)
///
/// This `RangesIter` would produce the following series of ranges and element sets:
///
/// - 5..10: { B }
/// - 15..15: { B, D }
/// - 15..20: { C, D }
/// - 20..25: { C, A }
/// - 25..30: { A }
/// 
///
/// To create a `RangesIter`, you must provide an iterator that produces all
/// the elements of the collection, together with a [`RangeGetter`] implementation
/// that can extract a range from an element.
/// sets of those values
///
/// The
///
/// together with
/// a [`RangeGetter`] implementor `G` that can extract ranges from such values,
/// where the ranges' endpoints are ordered (they must implement `Ord`).
pub struct RangesIter<T, G> {
    /// All the elements that will appear in the next set of ranges.
    next: BinaryHeap<T, ByStart<G>>,

    /// All the elements whose ranges we have not yet reached.
    unreached: BinaryHeap<T, ByEnd<G>>,
}

struct ByStart<G> {
    getter: G,
}

impl<T, G> compare::Compare<T> for ByStart<G>
where G: RangeGetter<T>,
      G::End: std::cmp::Ord,
{
    fn compare(&self, l: &T, r: &T) -> std::cmp::Ordering {
        self.getter.range_of(l).start.cmp(&self.getter.range_of(r).start)
    }
}

struct ByEnd<G> {
    getter: G,
}

impl<T, G> compare::Compare<T> for ByEnd<G>
where G: RangeGetter<T>,
      G::End: std::cmp::Ord,
{
    fn compare(&self, l: &T, r: &T) -> std::cmp::Ordering {
        self.getter.range_of(l).end.cmp(&self.getter.range_of(r).end)
    }
}
