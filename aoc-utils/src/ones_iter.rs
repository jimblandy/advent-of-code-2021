use num_traits::int::PrimInt;

/// Return an iterator over the bit numbers of the bits set in `n`.
///
/// The least significant bit is numbered zero.
pub fn set_bit_numbers<N>(mut n: N) -> impl Iterator<Item = u32>
where N: PrimInt,
      N: std::ops::Sub<Output = N>,
      N: std::ops::BitAnd<Output = N>,
      N: std::ops::Not<Output = N>,
{
    std::iter::from_fn(move || {
        if n == N::zero() {
            return None;
        }

        let index = n.trailing_zeros();
        n = n & (n - N::one());
        Some(index)
    })
}
