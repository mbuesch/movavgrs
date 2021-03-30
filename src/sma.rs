// -*- coding: utf-8 -*-
//
// Copyright 2021 Michael Büsch <m@bues.ch>
//
// Licensed under the Apache License version 2.0
// or the MIT license, at your option.
// SPDX-License-Identifier: Apache-2.0 OR MIT
//

use std::cmp::min;
use num_traits::{
    Num,
    NumCast,
};

/// Internal checked-add trait for integers and floats.
pub trait MovAvgAdd: Copy {
    fn add_chk(self, other: Self) -> Option<Self>;
}

macro_rules! impl_int_add {
    ($($t:ty),*) => {
        $(
            impl MovAvgAdd for $t {
                #[inline]
                fn add_chk(self, other: Self) -> Option<Self> {
                    self.checked_add(other)
                }
            }
        )*
    }
}

macro_rules! impl_float_add {
    ($($t:ty),*) => {
        $(
            impl MovAvgAdd for $t {
                #[inline]
                fn add_chk(self, other: Self) -> Option<Self> {
                    Some(self + other)
                }
            }
        )*
    }
}

impl_int_add!(i8, i16, i32, i64, i128, isize,
              u8, u16, u32, u64, u128, usize);
impl_float_add!(f32, f64);

/// Simple Moving Average (SMA)
///
/// # Examples
///
/// ```
/// use movavg::MovAvg;
///
/// // Integers
/// let mut avg: MovAvg<i32> = MovAvg::new(3);
/// assert_eq!(avg.feed(10), 10);
/// assert_eq!(avg.feed(20), 15);
/// assert_eq!(avg.feed(30), 20);
/// assert_eq!(avg.feed(40), 30);
///
/// // Floats
/// let mut avg: MovAvg<f64> = MovAvg::new(3);
/// assert_eq!(avg.feed(10.0), 10.0);
/// assert_eq!(avg.feed(20.0), 15.0);
/// assert_eq!(avg.feed(30.0), 20.0);
/// assert_eq!(avg.feed(40.0), 30.0);
///
/// // Bigger accumulator
/// let mut avg: MovAvg<i8, i32> = MovAvg::new(3);
/// assert_eq!(avg.feed(100), 100);
/// assert_eq!(avg.feed(100), 100); // This would overflow an i8 accumulator
/// ```
///
/// # Type Generics
///
/// `struct MovAvg<T, A=T>`
///
/// * `T` - The type of the `feed()` input value.
/// * `A` - The type of the internal accumulator.
///         This type must be bigger then or equal to `T`. By default this is `T`.
pub struct MovAvg<T, A=T> {
    items:      Vec<T>,
    accu:       A,
    size:       usize,
    nr_items:   usize,
    begin:      usize,
    end:        usize,
}

impl<T: Num + NumCast + Copy,
     A: Num + NumCast + Copy + MovAvgAdd>
    MovAvg<T, A> {

    /// Construct a new Simple Moving Average.
    ///
    /// The internal accumulator defaults to zero.
    ///
    /// * `size` - The size of the sliding window. In number of fed elements.
    ///
    /// # Panics
    ///
    /// Panics, if:
    /// * `size` is less than 1.
    /// * `size` is bigger than `usize::MAX - 1`.
    pub fn new(size: usize) -> MovAvg<T, A> {
        Self::new_init(vec![T::zero(); size], 0)
    }

    /// Construct a new Simple Moving Average and initialize its internal state.
    ///
    /// * `items` - Pre-initialized window buffer. Contains the window values.
    ///             The length of this vector must be at least 1.
    ///             The length of this vector will be the size of the sliding window.
    ///             The initialized elements must begin at index 0.
    /// * `nr_items` - The number of valid pre-initialized values in `items`.
    ///                This number must be between `0..=items.len()`.
    ///                The value of the items in `items[nr_items]` or above will be ignored.
    ///
    /// # Panics
    ///
    /// Panics, if:
    /// * `items.len()` is less than 1.
    /// * `nr_items` is bigger than `items.len()`.
    /// * `items.len()` is bigger than `usize::MAX - 1`.
    pub fn new_init(items: Vec<T>,
                    nr_items: usize) -> MovAvg<T, A> {

        let size = items.len();
        assert!(size > 0);
        assert!(nr_items <= size);
        // Avoid overflow on temporary nr_items incrementation.
        assert!(size <= usize::MAX - 1);

        let accu = items.iter().fold(A::zero(),
            |acc, x| acc + A::from(*x).expect("Failed to cast value to accumulator type."));

        let begin = 0;
        let end = nr_items % size;

        MovAvg {
            items,
            accu,
            size,
            nr_items,
            begin,
            end,
        }
    }

    /// Try to feed a new value into the Moving Average and return the new average.
    ///
    /// * `value` - The new value to feed into the Moving Average.
    ///
    /// Returns Err, if the internal accumulator overflows, or if any other value conversion fails.
    /// Value conversion does not fail, if the types are big enough to hold the values.
    pub fn try_feed(&mut self, value: T) -> Result<T, &str> {
        debug_assert!(self.nr_items <= self.size);

        // Get the first element from the moving window state.
        let first_value = if self.nr_items >= self.size {
            self.items[self.begin]
        } else {
            T::zero()
        };

        let a_first_value = A::from(first_value)
            .ok_or("Failed to cast first value to accumulator type.")?;
        let a_value = A::from(value)
            .ok_or("Failed to cast value to accumulator type.")?;

        // Subtract the to be removed value from the sum and add the new value.
        let new_accu = (self.accu - a_first_value).add_chk(a_value)
            .ok_or("Accumulator type add overflow.")?;
        let new_nr_items = min(self.nr_items + 1, self.size);
        let a_nr_items = A::from(new_nr_items)
            .ok_or("Failed to cast number-of-items to accumulator type.")?;

        // Calculate the new average.
        let ret = new_accu / a_nr_items;
        let ret = T::from(ret)
            .ok_or("Failed to cast result to item type.")?;

        // Append the new value to the list and update the moving window state.
        self.items[self.end] = value;
        self.nr_items = new_nr_items;
        self.end = (self.end + 1) % self.size;
        self.begin = (self.begin + 1) % self.size;
        self.accu = new_accu;

        Ok(ret)
    }

    /// Feed a new value into the Moving Average and return the new average.
    ///
    /// * `value` - The new value to feed into the Moving Average.
    ///
    /// # Panics
    ///
    /// Panics, if the internal accumulator overflows, or if any other value conversion fails.
    /// Value conversion does not fail, if the types are big enough to hold the values.
    pub fn feed(&mut self, value: T) -> T {
        self.try_feed(value).expect("MovAvg calculation failed.")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_u8() {
        let mut a: MovAvg<u8> = MovAvg::new(3);
        assert_eq!(a.feed(10), 10 / 1);
        assert_eq!(a.feed(20), (10 + 20) / 2);
        assert_eq!(a.feed(2), (10 + 20 + 2) / 3);
        assert_eq!(a.feed(100), (20 + 2 + 100) / 3);
        assert_eq!(a.feed(111), (2 + 100 + 111) / 3);
    }

    #[test]
    fn test_i8() {
        let mut a: MovAvg<i8> = MovAvg::new(5);
        assert_eq!(a.feed(10), 10 / 1);
        assert_eq!(a.feed(20), (10 + 20) / 2);
        assert_eq!(a.feed(2), (10 + 20 + 2) / 3);
        assert_eq!(a.feed(-4), (10 + 20 + 2 - 4) / 4);
        assert_eq!(a.feed(-19), (10 + 20 + 2 - 4 - 19) / 5);
        assert_eq!(a.feed(-20), (20 + 2 - 4 - 19 - 20) / 5);
    }

    #[test]
    fn test_u16() {
        let mut a: MovAvg<u16> = MovAvg::new(5);
        assert_eq!(a.feed(10), 10 / 1);
        assert_eq!(a.feed(20), (10 + 20) / 2);
        assert_eq!(a.feed(2), (10 + 20 + 2) / 3);
        assert_eq!(a.feed(100), (10 + 20 + 2 + 100) / 4);
        assert_eq!(a.feed(111), (10 + 20 + 2 + 100 + 111) / 5);
        assert_eq!(a.feed(200), (20 + 2 + 100 + 111 + 200) / 5);
        assert_eq!(a.feed(250), (2 + 100 + 111 + 200 + 250) / 5);
        assert_eq!(a.feed(10_000), (100 + 111 + 200 + 250 + 10_000) / 5);
    }

    #[test]
    fn test_i16() {
        let mut a: MovAvg<i16> = MovAvg::new(5);
        assert_eq!(a.feed(10), 10 / 1);
        assert_eq!(a.feed(20), (10 + 20) / 2);
        assert_eq!(a.feed(2), (10 + 20 + 2) / 3);
        assert_eq!(a.feed(100), (10 + 20 + 2 + 100) / 4);
        assert_eq!(a.feed(111), (10 + 20 + 2 + 100 + 111) / 5);
        assert_eq!(a.feed(200), (20 + 2 + 100 + 111 + 200) / 5);
        assert_eq!(a.feed(250), (2 + 100 + 111 + 200 + 250) / 5);
        assert_eq!(a.feed(-25), (100 + 111 + 200 + 250 - 25) / 5);
        assert_eq!(a.feed(-10_000), (111 + 200 + 250 - 25 - 10_000) / 5);
    }

    #[test]
    fn test_u32() {
        let mut a: MovAvg<u32> = MovAvg::new(5);
        assert_eq!(a.feed(10), 10 / 1);
        assert_eq!(a.feed(20), (10 + 20) / 2);
        assert_eq!(a.feed(2), (10 + 20 + 2) / 3);
        assert_eq!(a.feed(100), (10 + 20 + 2 + 100) / 4);
        assert_eq!(a.feed(111), (10 + 20 + 2 + 100 + 111) / 5);
        assert_eq!(a.feed(200), (20 + 2 + 100 + 111 + 200) / 5);
        assert_eq!(a.feed(250), (2 + 100 + 111 + 200 + 250) / 5);
        assert_eq!(a.feed(100_000), (100 + 111 + 200 + 250 + 100_000) / 5);
    }

    #[test]
    fn test_i32() {
        let mut a: MovAvg<i32> = MovAvg::new(5);
        assert_eq!(a.feed(10), 10 / 1);
        assert_eq!(a.feed(20), (10 + 20) / 2);
        assert_eq!(a.feed(2), (10 + 20 + 2) / 3);
        assert_eq!(a.feed(100), (10 + 20 + 2 + 100) / 4);
        assert_eq!(a.feed(111), (10 + 20 + 2 + 100 + 111) / 5);
        assert_eq!(a.feed(200), (20 + 2 + 100 + 111 + 200) / 5);
        assert_eq!(a.feed(250), (2 + 100 + 111 + 200 + 250) / 5);
        assert_eq!(a.feed(-25), (100 + 111 + 200 + 250 - 25) / 5);
        assert_eq!(a.feed(-100_000), (111 + 200 + 250 - 25 - 100_000) / 5);
    }

    #[test]
    fn test_u64() {
        let mut a: MovAvg<u64> = MovAvg::new(5);
        assert_eq!(a.feed(10), 10 / 1);
        assert_eq!(a.feed(20), (10 + 20) / 2);
        assert_eq!(a.feed(2), (10 + 20 + 2) / 3);
        assert_eq!(a.feed(100), (10 + 20 + 2 + 100) / 4);
        assert_eq!(a.feed(111), (10 + 20 + 2 + 100 + 111) / 5);
        assert_eq!(a.feed(200), (20 + 2 + 100 + 111 + 200) / 5);
        assert_eq!(a.feed(250), (2 + 100 + 111 + 200 + 250) / 5);
        assert_eq!(a.feed(10_000_000_000), (100 + 111 + 200 + 250 + 10_000_000_000) / 5);
    }

    #[test]
    fn test_i64() {
        let mut a: MovAvg<i64> = MovAvg::new(5);
        assert_eq!(a.feed(10), 10 / 1);
        assert_eq!(a.feed(20), (10 + 20) / 2);
        assert_eq!(a.feed(2), (10 + 20 + 2) / 3);
        assert_eq!(a.feed(100), (10 + 20 + 2 + 100) / 4);
        assert_eq!(a.feed(111), (10 + 20 + 2 + 100 + 111) / 5);
        assert_eq!(a.feed(200), (20 + 2 + 100 + 111 + 200) / 5);
        assert_eq!(a.feed(250), (2 + 100 + 111 + 200 + 250) / 5);
        assert_eq!(a.feed(-25), (100 + 111 + 200 + 250 - 25) / 5);
        assert_eq!(a.feed(-10_000_000_000), (111 + 200 + 250 - 25 - 10_000_000_000) / 5);
    }

    #[test]
    fn test_u128() {
        let mut a: MovAvg<u128> = MovAvg::new(5);
        assert_eq!(a.feed(10), 10 / 1);
        assert_eq!(a.feed(20), (10 + 20) / 2);
        assert_eq!(a.feed(2), (10 + 20 + 2) / 3);
        assert_eq!(a.feed(100), (10 + 20 + 2 + 100) / 4);
        assert_eq!(a.feed(111), (10 + 20 + 2 + 100 + 111) / 5);
        assert_eq!(a.feed(200), (20 + 2 + 100 + 111 + 200) / 5);
        assert_eq!(a.feed(250), (2 + 100 + 111 + 200 + 250) / 5);
        assert_eq!(a.feed(10_000_000_000_000_000_000_000), (100 + 111 + 200 + 250 + 10_000_000_000_000_000_000_000) / 5);
    }

    #[test]
    fn test_i128() {
        let mut a: MovAvg<i128> = MovAvg::new(5);
        assert_eq!(a.feed(10), 10 / 1);
        assert_eq!(a.feed(20), (10 + 20) / 2);
        assert_eq!(a.feed(2), (10 + 20 + 2) / 3);
        assert_eq!(a.feed(100), (10 + 20 + 2 + 100) / 4);
        assert_eq!(a.feed(111), (10 + 20 + 2 + 100 + 111) / 5);
        assert_eq!(a.feed(200), (20 + 2 + 100 + 111 + 200) / 5);
        assert_eq!(a.feed(250), (2 + 100 + 111 + 200 + 250) / 5);
        assert_eq!(a.feed(-25), (100 + 111 + 200 + 250 - 25) / 5);
        assert_eq!(a.feed(-10_000_000_000_000_000_000_000), (111 + 200 + 250 - 25 - 10_000_000_000_000_000_000_000) / 5);
    }

    #[test]
    fn test_usize() {
        let mut a: MovAvg<usize> = MovAvg::new(5);
        assert_eq!(a.feed(10), 10 / 1);
        assert_eq!(a.feed(20), (10 + 20) / 2);
        assert_eq!(a.feed(2), (10 + 20 + 2) / 3);
        assert_eq!(a.feed(100), (10 + 20 + 2 + 100) / 4);
        assert_eq!(a.feed(111), (10 + 20 + 2 + 100 + 111) / 5);
        assert_eq!(a.feed(200), (20 + 2 + 100 + 111 + 200) / 5);
        assert_eq!(a.feed(250), (2 + 100 + 111 + 200 + 250) / 5);
        assert_eq!(a.feed(100_000), (100 + 111 + 200 + 250 + 100_000) / 5);
    }

    #[test]
    fn test_isize() {
        let mut a: MovAvg<isize> = MovAvg::new(5);
        assert_eq!(a.feed(10), 10 / 1);
        assert_eq!(a.feed(20), (10 + 20) / 2);
        assert_eq!(a.feed(2), (10 + 20 + 2) / 3);
        assert_eq!(a.feed(100), (10 + 20 + 2 + 100) / 4);
        assert_eq!(a.feed(111), (10 + 20 + 2 + 100 + 111) / 5);
        assert_eq!(a.feed(200), (20 + 2 + 100 + 111 + 200) / 5);
        assert_eq!(a.feed(250), (2 + 100 + 111 + 200 + 250) / 5);
        assert_eq!(a.feed(-25), (100 + 111 + 200 + 250 - 25) / 5);
        assert_eq!(a.feed(-100_000), (111 + 200 + 250 - 25 - 100_000) / 5);
    }

    #[test]
    fn test_f32() {
        let mut a: MovAvg<f32> = MovAvg::new(5);
        let e = 0.001;
        assert!((a.feed(10.0) - (10.0 / 1.0)).abs() < e);
        assert!((a.feed(20.0) - ((10.0 + 20.0) / 2.0)).abs() < e);
        assert!((a.feed(2.0) - ((10.0 + 20.0 + 2.0) / 3.0)).abs() < e);
        assert!((a.feed(100.0) - ((10.0 + 20.0 + 2.0 + 100.0) / 4.0)).abs() < e);
        assert!((a.feed(111.0) - ((10.0 + 20.0 + 2.0 + 100.0 + 111.0) / 5.0)).abs() < e);
        assert!((a.feed(200.0) - ((20.0 + 2.0 + 100.0 + 111.0 + 200.0) / 5.0)).abs() < e);
        assert!((a.feed(250.0) - ((2.0 + 100.0 + 111.0 + 200.0 + 250.0) / 5.0)).abs() < e);
        assert!((a.feed(-25.0) - ((100.0 + 111.0 + 200.0 + 250.0 - 25.0) / 5.0)).abs() < e);
        assert!((a.feed(-100000.0) - ((111.0 + 200.0 + 250.0 - 25.0 - 100000.0) / 5.0)).abs() < e);
    }

    #[test]
    fn test_f64() {
        let mut a: MovAvg<f64> = MovAvg::new(5);
        let e = 0.000001;
        assert!((a.feed(10.0) - (10.0 / 1.0)).abs() < e);
        assert!((a.feed(20.0) - ((10.0 + 20.0) / 2.0)).abs() < e);
        assert!((a.feed(2.0) - ((10.0 + 20.0 + 2.0) / 3.0)).abs() < e);
        assert!((a.feed(100.0) - ((10.0 + 20.0 + 2.0 + 100.0) / 4.0)).abs() < e);
        assert!((a.feed(111.0) - ((10.0 + 20.0 + 2.0 + 100.0 + 111.0) / 5.0)).abs() < e);
        assert!((a.feed(200.0) - ((20.0 + 2.0 + 100.0 + 111.0 + 200.0) / 5.0)).abs() < e);
        assert!((a.feed(250.0) - ((2.0 + 100.0 + 111.0 + 200.0 + 250.0) / 5.0)).abs() < e);
        assert!((a.feed(-25.0) - ((100.0 + 111.0 + 200.0 + 250.0 - 25.0) / 5.0)).abs() < e);
        assert!((a.feed(-100000.0) - ((111.0 + 200.0 + 250.0 - 25.0 - 100000.0) / 5.0)).abs() < e);
    }

    #[test]
    fn test_single() {
        let mut a: MovAvg<i32> = MovAvg::new(1);
        assert_eq!(a.feed(10), 10);
        assert_eq!(a.feed(20), 20);
        assert_eq!(a.feed(2), 2);
    }

    #[test]
    #[should_panic(expected="Accumulator type add overflow")]
    fn test_accu_overflow() {
        let mut a: MovAvg<u8> = MovAvg::new(3);
        a.feed(200);
        a.feed(200);
    }

    #[test]
    #[should_panic(expected="Accumulator type add overflow")]
    fn test_accu_underflow() {
        let mut a: MovAvg<i8> = MovAvg::new(3);
        a.feed(-100);
        a.feed(-100);
    }
}

// vim: ts=4 sw=4 expandtab
