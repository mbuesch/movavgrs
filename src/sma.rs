// -*- coding: utf-8 -*-
//
// Copyright 2021 Michael Büsch <m@bues.ch>
//
// Licensed under the Apache License version 2.0
// or the MIT license, at your option.
// SPDX-License-Identifier: Apache-2.0 OR MIT
//

use num_traits::{
    Num,
    NumCast,
};

/// Initialize the accumulator from scratch by summing up all items.
#[inline]
fn initialize_accu<T, A>(items: &[T]) -> Result<A, &'static str>
    where T: Num + NumCast + Copy,
          A: Num + NumCast + Copy,
{
    items.iter().fold(
        Ok(A::zero()),
        |acc, x| {
            match acc {
                Ok(acc) => Ok(acc + A::from(*x).ok_or("Failed to cast value to accumulator type.")?),
                Err(e) => Err(e),
            }
        }
    )
}

/// Internal accumulator calculation trait for integers and floats.
///
/// This usually does *not* have to be implemented by the library user.
/// The `movavg` crate implements this trait for all core integers and floats.
///
/// `Self` is the accumulator type `A`.
///
/// `T` is the SMA input value type.
pub trait MovAvgAccu<T>: Copy {
    fn recalc_accu(self,
                   first_value: Self,
                   input_value: Self,
                   items: &[T]) -> Result<Self, &'static str>;
}

macro_rules! impl_int_accu {
    ($($t:ty),*) => {
        $(
            impl<T> MovAvgAccu<T> for $t {
                #[inline]
                fn recalc_accu(self,
                               first_value: Self,
                               input_value: Self,
                               _items: &[T]) -> Result<Self, &'static str> {
                    // Subtract the to be removed value from the sum and add the new value.
                    (self - first_value).checked_add(input_value)
                        .ok_or("Accumulator type add overflow.")
                }
            }
        )*
    }
}

macro_rules! impl_float_accu {
    ($($t:ty),*) => {
        $(
            impl<T: Num + NumCast + Copy> MovAvgAccu<T> for $t {
                #[inline]
                fn recalc_accu(self,
                               _first_value: Self,
                               _input_value: Self,
                               items: &[T]) -> Result<Self, &'static str> {
                    // Recalculate the accumulator from scratch.
                    initialize_accu(items)
                }
            }
        )*
    }
}

impl_int_accu!(i8, i16, i32, i64, isize,
               u8, u16, u32, u64, usize);

#[cfg(has_i128)]
impl_int_accu!(i128, u128);

impl_float_accu!(f32, f64);

/// Simple Moving Average (SMA)
///
/// # Examples
///
/// ```
/// use movavg::MovAvg;
///
/// // Integers
/// let mut avg: MovAvg<i32> = MovAvg::new(3); // window size = 3
/// assert_eq!(avg.feed(10), 10);
/// assert_eq!(avg.feed(20), 15);
/// assert_eq!(avg.feed(30), 20);
/// assert_eq!(avg.feed(40), 30);
/// assert_eq!(avg.get(), 30);
///
/// // Floats
/// let mut avg: MovAvg<f64> = MovAvg::new(3);
/// assert_eq!(avg.feed(10.0), 10.0);
/// assert_eq!(avg.feed(20.0), 15.0);
/// assert_eq!(avg.feed(30.0), 20.0);
/// assert_eq!(avg.feed(40.0), 30.0);
/// assert_eq!(avg.get(), 30.0);
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
///         This type must be bigger then or equal to `T`.
///         By default this is the same type as `T`.
pub struct MovAvg<T, A=T> {
    items:      Vec<T>,
    accu:       A,
    nr_items:   usize,
    index:      usize,
}

impl<T: Num + NumCast + Copy,
     A: Num + NumCast + Copy + MovAvgAccu<T>>
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
    pub fn new(size: usize) -> MovAvg<T, A> {
        Self::new_init(size, vec![])
    }

    /// Construct a new Simple Moving Average and initialize its internal state.
    ///
    /// * `size` - The size of the sliding window. In number of fed elements.
    /// * `items` - Pre-initialized window buffer. Contains the window values.
    ///             The length of this vector must be less than or equal to `size`.
    ///
    /// # Panics
    ///
    /// Panics, if:
    /// * `size` is less than 1.
    /// * `items.len()` is bigger than `size`.
    /// * The initial accumulator calculation fails. (e.g. due to overflow).
    pub fn new_init(size: usize,
                    mut items: Vec<T>) -> MovAvg<T, A> {

        assert!(size > 0);
        let nr_items = items.len();
        assert!(nr_items <= size);
        items.resize(size, T::one());
        let index = nr_items % size;

        let accu = initialize_accu(&items[0..nr_items])
            .expect("Failed to initialize the accumulator.");

        MovAvg {
            items,
            accu,
            nr_items,
            index,
        }
    }

    /// Try to feed a new value into the Moving Average and return the new average.
    ///
    /// * `value` - The new value to feed into the Moving Average.
    ///
    /// On success, returns `Ok(T)` with the new Moving Average result.
    ///
    /// Returns `Err`, if the internal accumulator overflows, or if any value conversion fails.
    /// Value conversion does not fail, if the types are big enough to hold the values.
    pub fn try_feed(&mut self, value: T) -> Result<T, &str> {
        let size = self.items.len();
        debug_assert!(self.nr_items <= size);

        // Get the first element from the moving window state.
        let first_value = if self.nr_items >= size {
            A::from(self.items[self.index])
                .ok_or("Failed to cast first value to accumulator type.")?
        } else {
            A::zero()
        };

        let a_value = A::from(value)
            .ok_or("Failed to cast value to accumulator type.")?;

        // Calculate the new moving window state fill state.
        let new_nr_items = if self.nr_items >= size {
            self.nr_items // Already fully populated.
        } else {
            self.nr_items + 1
        };
        let a_nr_items = A::from(new_nr_items)
            .ok_or("Failed to cast number-of-items to accumulator type.")?;

        // Insert the new value into the moving window state.
        // If en error happens later, orig_item has to be restored.
        let orig_item = self.items[self.index];
        self.items[self.index] = value;

        // Recalculate the accumulator.
        match self.accu.recalc_accu(first_value,
                                    a_value,
                                    &self.items[0..new_nr_items]) {
            Ok(new_accu) => {
                // Calculate the new average.
                match T::from(new_accu / a_nr_items) {
                    Some(avg) => {
                        // Update the state.
                        self.nr_items = new_nr_items;
                        self.index = (self.index + 1) % size;
                        self.accu = new_accu;

                        // Return the end result.
                        Ok(avg)
                    },
                    None => {
                        // Restore the original moving window state.
                        self.items[self.index] = orig_item;
                        Err("Failed to cast result to item type.")
                    },
                }
            },
            Err(e) => {
                // Restore the original moving window state.
                self.items[self.index] = orig_item;
                Err(e)
            }
        }
    }

    /// Feed a new value into the Moving Average and return the new average.
    ///
    /// * `value` - The new value to feed into the Moving Average.
    ///
    /// Returns the new Moving Average result.
    ///
    /// # Panics
    ///
    /// Panics, if the internal accumulator overflows, or if any value conversion fails.
    /// Value conversion does not fail, if the types are big enough to hold the values.
    pub fn feed(&mut self, value: T) -> T {
        self.try_feed(value).expect("MovAvg calculation failed.")
    }

    /// Try to get the current Moving Average value.
    /// This method does not modify the internal state.
    ///
    /// Returns `Err`, if the internal state is empty.
    /// That is if no values have been fed into MovAvg.
    ///
    /// Returns `Err`, if any value conversion fails.
    /// Value conversion does not fail, if the types are big enough to hold the values.
    pub fn try_get(&self) -> Result<T, &str> {
        if let Some(nr_items) = A::from(self.nr_items) {
            if nr_items == A::zero() {
                Err("The MovAvg state is empty.")
            } else {
                T::from(self.accu / nr_items)
                    .ok_or("Failed to cast result to item type.")
            }
        } else {
            Err("Failed to cast number-of-items to accumulator type.")
        }
    }

    /// Get the current Moving Average value.
    /// This method does not modify the internal state.
    ///
    /// # Panics
    ///
    /// Panics, if the internal state is empty.
    /// That is if no values have been fed into MovAvg.
    ///
    /// Panics, if any value conversion fails.
    /// Value conversion does not fail, if the types are big enough to hold the values.
    pub fn get(&self) -> T {
        self.try_get().expect("MovAvg calculation failed.")
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

    #[cfg(has_i128)]
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

    #[cfg(has_i128)]
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
        a.feed(200); // this panics
    }

    #[test]
    #[should_panic(expected="Accumulator type add overflow")]
    fn test_accu_underflow() {
        let mut a: MovAvg<i8> = MovAvg::new(3);
        a.feed(-100);
        a.feed(-100); // this panics
    }

    #[test]
    fn test_init() {
        let mut a: MovAvg<i32> = MovAvg::new_init(3, vec![10]);
        assert_eq!(a.feed(20), 15);
        assert_eq!(a.feed(102), 44);
        assert_eq!(a.feed(178), 100);

        let mut a: MovAvg<i32> = MovAvg::new_init(3, vec![10, 20]);
        assert_eq!(a.feed(102), 44);
        assert_eq!(a.feed(178), 100);
    }

    #[test]
    fn test_get() {
        let mut a: MovAvg<i32> = MovAvg::new_init(3, vec![10, 20]);
        assert_eq!(a.get(), 15);
        assert_eq!(a.feed(102), 44);
        assert_eq!(a.get(), 44);
        assert_eq!(a.feed(178), 100);
        assert_eq!(a.get(), 100);
    }

    #[test]
    #[should_panic(expected="The MovAvg state is empty")]
    fn test_get_empty() {
        let a: MovAvg<i32> = MovAvg::new(3);
        assert_eq!(a.get(), 42); // this panics
    }
}

// vim: ts=4 sw=4 expandtab
