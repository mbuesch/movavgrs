// -*- coding: utf-8 -*-
//
// Copyright 2021-2023 Michael Büsch <m@bues.ch>
//
// Licensed under the Apache License version 2.0
// or the MIT license, at your option.
// SPDX-License-Identifier: Apache-2.0 OR MIT
//

use num_traits::{
    Num,
    NumCast,
};

/// Initialize the accumulator from scratch by summing up all items from the window buffer.
#[inline]
fn initialize_accu<T, A>(window_buffer: &[T]) -> Result<A, &'static str>
where T: Num + NumCast + Copy,
      A: Num + NumCast + Copy,
{
    let mut accu = A::zero();
    for value in window_buffer {
        if let Some(value) = A::from(*value) {
            accu = accu + value;
        } else {
            return Err("Failed to cast value to accumulator type.");
        }
    }
    Ok(accu)
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
                   window_buffer: &[T]) -> Result<Self, &'static str>;
}

macro_rules! impl_int_accu {
    ($($t:ty),*) => {
        $(
            impl<T> MovAvgAccu<T> for $t {
                #[inline]
                fn recalc_accu(self,
                               first_value: Self,
                               input_value: Self,
                               _window_buffer: &[T]) -> Result<Self, &'static str> {
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
            impl<T> MovAvgAccu<T> for $t
            where
                T: Num + NumCast + Copy
            {
                #[inline]
                fn recalc_accu(self,
                               first_value: Self,
                               input_value: Self,
                               window_buffer: &[T]) -> Result<Self, &'static str> {
                    if cfg!(feature="fastfloat") {
                        // Fast calculation, just like the integer variant.
                        Ok((self - first_value) + input_value)
                    } else {
                        // Recalculate the accumulator from scratch.
                        initialize_accu(window_buffer)
                    }
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
/// let mut avg: MovAvg<i32, i32, 3> = MovAvg::new(); // window size = 3
/// assert_eq!(avg.feed(10), 10);
/// assert_eq!(avg.feed(20), 15);
/// assert_eq!(avg.feed(30), 20);
/// assert_eq!(avg.feed(40), 30);
/// assert_eq!(avg.get(), 30);
/// ```
///
/// `MovAvg` also implements `Default`:
///
/// ```
/// use movavg::MovAvg;
///
/// let mut avg: MovAvg<i32, i32, 3> = Default::default();
/// assert_eq!(avg.feed(10), 10);
/// ```
///
/// # Type Generics
///
/// `struct MovAvg<T, A, WINDOW_SIZE>`
///
/// * `T` - The type of the `feed()` input value.
/// * `A` - The type of the internal accumulator.
///         This type must be bigger then or equal to `T`.
/// * `WINDOW_SIZE` - The size of the sliding window.
///                   In number of fed elements.
pub struct MovAvg<T, A, const WINDOW_SIZE: usize> {
    buffer:     [T; WINDOW_SIZE],
    accu:       A,
    nr_items:   usize,
    index:      usize,
}

impl<T, A, const WINDOW_SIZE: usize> MovAvg<T, A, WINDOW_SIZE>
where
    T: Num + NumCast + Copy,
    A: Num + NumCast + Copy + MovAvgAccu<T>,
{
    /// Construct a new Simple Moving Average.
    ///
    /// The internal accumulator defaults to zero.
    ///
    /// # Examples
    ///
    /// ```
    /// use movavg::MovAvg;
    ///
    /// let mut avg: MovAvg<i32, i32, 3> = MovAvg::new(); // window size = 3
    /// assert_eq!(avg.feed(10), 10);
    /// ```
    pub fn new() -> MovAvg<T, A, WINDOW_SIZE> {
        assert!(WINDOW_SIZE > 0);
        Self::new_init([T::one(); WINDOW_SIZE], 0)
    }

    /// Construct a new Simple Moving Average from a pre-allocated buffer
    /// and initialize its internal state.
    ///
    /// * `buffer` - (Partially) pre-populated window buffer. Contains the window values.
    ///              The length of this buffer defines the Moving Average window size.
    /// * `nr_populated` - The number of pre-populated Moving Average window elements in `buffer`.
    ///                    `nr_populated` must be less than or equal to `buffer.len()`.
    ///                    The populated values in `buffer` must begin at index 0.
    ///                    The values of unpopulated elements in `buffer` does not matter.
    ///
    /// # Panics
    ///
    /// Panics, if:
    /// * `nr_populated` is bigger than `buffer.len()`.
    /// * The initial accumulator calculation fails. (e.g. due to overflow).
    ///
    /// # Examples
    ///
    /// ```
    /// use movavg::MovAvg;
    ///
    /// let mut buf = [10, 20, 30,  // populated
    ///                0, 0];       // unpopulated
    ///
    /// let mut avg: MovAvg<i32, i32, 5> =
    ///     MovAvg::new_init(buf,   // Pass buffer ownership.
    ///                      3);    // The first three elements of buf are pre-populated.
    ///
    /// assert_eq!(avg.get(), 20);
    /// assert_eq!(avg.feed(60), 30);
    /// assert_eq!(avg.feed(30), 30);
    /// assert_eq!(avg.feed(60), 40);
    /// ```
    pub fn new_init(buffer: [T; WINDOW_SIZE],
                    nr_populated: usize) -> MovAvg<T, A, WINDOW_SIZE> {
        let size = buffer.len();
        assert!(WINDOW_SIZE > 0);
        assert!(size == WINDOW_SIZE);

        let nr_items = nr_populated;
        assert!(nr_items <= size);

        let index = nr_items % size;

        let accu = initialize_accu(&buffer[0..nr_items])
            .expect("Failed to initialize the accumulator.");

        MovAvg {
            buffer,
            accu,
            nr_items,
            index,
        }
    }

    /// Reset the Moving Average.
    ///
    /// This resets the number of accumulated items to 0,
    /// as if this instance was re-created with [new].
    ///
    /// Note: This does not actually overwrite the buffered items in memory.
    pub fn reset(&mut self) {
        self.accu = A::zero();
        self.nr_items = 0;
        self.index = 0;
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
        let size = self.buffer.len();
        debug_assert!(self.nr_items <= size);

        // Get the first element from the moving window state.
        let first_value = if self.nr_items >= size {
            A::from(self.buffer[self.index])
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
        let orig_item = self.buffer[self.index];
        self.buffer[self.index] = value;

        // Recalculate the accumulator.
        match self.accu.recalc_accu(first_value,
                                    a_value,
                                    &self.buffer[0..new_nr_items]) {
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
                        self.buffer[self.index] = orig_item;
                        Err("Failed to cast result to item type.")
                    },
                }
            },
            Err(e) => {
                // Restore the original moving window state.
                self.buffer[self.index] = orig_item;
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

impl<A, T, const WINDOW_SIZE: usize> Default for MovAvg<T, A, WINDOW_SIZE>
where
    T: Num + NumCast + Copy,
    A: Num + NumCast + Copy + MovAvgAccu<T>,
{
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_u8() {
        let mut a: MovAvg<u8, u8, 3> = MovAvg::new();
        assert_eq!(a.feed(10), 10);
        assert_eq!(a.feed(20), (10 + 20) / 2);
        assert_eq!(a.feed(2), (10 + 20 + 2) / 3);
        assert_eq!(a.feed(100), (20 + 2 + 100) / 3);
        assert_eq!(a.feed(111), (2 + 100 + 111) / 3);
    }

    #[test]
    fn test_i8() {
        let mut a: MovAvg<i8, i8, 5> = MovAvg::new();
        assert_eq!(a.feed(10), 10);
        assert_eq!(a.feed(20), (10 + 20) / 2);
        assert_eq!(a.feed(2), (10 + 20 + 2) / 3);
        assert_eq!(a.feed(-4), (10 + 20 + 2 - 4) / 4);
        assert_eq!(a.feed(-19), (10 + 20 + 2 - 4 - 19) / 5);
        assert_eq!(a.feed(-20), (20 + 2 - 4 - 19 - 20) / 5);
    }

    #[test]
    fn test_u16() {
        let mut a: MovAvg<u16, u16, 5> = MovAvg::new();
        assert_eq!(a.feed(10), 10);
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
        let mut a: MovAvg<i16, i16, 5> = MovAvg::new();
        assert_eq!(a.feed(10), 10);
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
        let mut a: MovAvg<u32, u32, 5> = MovAvg::new();
        assert_eq!(a.feed(10), 10);
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
        let mut a: MovAvg<i32, i32, 5> = MovAvg::new();
        assert_eq!(a.feed(10), 10);
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
        let mut a: MovAvg<u64, u64, 5> = MovAvg::new();
        assert_eq!(a.feed(10), 10);
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
        let mut a: MovAvg<i64, i64, 5> = MovAvg::new();
        assert_eq!(a.feed(10), 10);
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
        let mut a: MovAvg<u128, u128, 5> = MovAvg::new();
        assert_eq!(a.feed(10), 10);
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
        let mut a: MovAvg<i128, i128, 5> = MovAvg::new();
        assert_eq!(a.feed(10), 10);
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
        let mut a: MovAvg<usize, usize, 5> = MovAvg::new();
        assert_eq!(a.feed(10), 10);
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
        let mut a: MovAvg<isize, isize, 5> = MovAvg::new();
        assert_eq!(a.feed(10), 10);
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
        let mut a: MovAvg<f32, f32, 5> = MovAvg::new();
        let e = 0.001;
        assert!((a.feed(10.0) - 10.0).abs() < e);
        assert!((a.feed(20.0) - ((10.0 + 20.0) / 2.0)).abs() < e);
        assert!((a.feed(2.0) - ((10.0 + 20.0 + 2.0) / 3.0)).abs() < e);
        assert!((a.feed(100.0) - ((10.0 + 20.0 + 2.0 + 100.0) / 4.0)).abs() < e);
        assert!((a.feed(111.0) - ((10.0 + 20.0 + 2.0 + 100.0 + 111.0) / 5.0)).abs() < e);
        assert!((a.feed(200.0) - ((20.0 + 2.0 + 100.0 + 111.0 + 200.0) / 5.0)).abs() < e);
        assert!((a.feed(250.0) - ((2.0 + 100.0 + 111.0 + 200.0 + 250.0) / 5.0)).abs() < e);
        assert!((a.feed(-25.0) - ((100.0 + 111.0 + 200.0 + 250.0 - 25.0) / 5.0)).abs() < e);
        assert!((a.feed(-100000.0) - ((111.0 + 200.0 + 250.0 - 25.0 - 100000.0) / 5.0)).abs() < e);
    }

    macro_rules! gen_test_float_extra {
        ($ty:ty) => {
            const PI: $ty = std::f64::consts::PI as $ty;
            let mut a: MovAvg<$ty, $ty, 200> = MovAvg::new();
            let mut prev = -0.1;
            for i in 0..1000 {
                let val = (PI * (i as $ty / 100.0)).sin();
                let res = a.feed(val);
                if i < 75 {
                    assert!(res > prev);
                } else if i < 200 {
                    assert!(res < prev);
                } else {
                    assert!(res < 1e-6);
                }
                prev = res;
            }
        }
    }

    #[test]
    fn test_f32_extra() {
        gen_test_float_extra!(f32);
    }

    #[test]
    fn test_f64() {
        let mut a: MovAvg<f64, f64, 5> = MovAvg::new();
        let e = 0.000001;
        assert!((a.feed(10.0) - 10.0).abs() < e);
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
    fn test_f64_extra() {
        gen_test_float_extra!(f64);
    }

    #[test]
    fn test_single() {
        let mut a: MovAvg<i32, i32, 1> = MovAvg::new();
        assert_eq!(a.feed(10), 10);
        assert_eq!(a.feed(20), 20);
        assert_eq!(a.feed(2), 2);
    }

    #[test]
    fn test_accu_overflow() {
        let mut a: MovAvg<u8, u8, 3> = MovAvg::new();
        a.feed(200);
        assert!(a.try_feed(200).is_err());
    }

    #[test]
    #[should_panic(expected="Accumulator type add overflow")]
    fn test_accu_overflow_panic() {
        let mut a: MovAvg<u8, u8, 3> = MovAvg::new();
        a.feed(200);
        a.feed(200); // this panics
    }

    #[test]
    fn test_accu_underflow() {
        let mut a: MovAvg<i8, i8, 3> = MovAvg::new();
        a.feed(-100);
        assert!(a.try_feed(-100).is_err());
    }

    #[test]
    #[should_panic(expected="Accumulator type add overflow")]
    fn test_accu_underflow_panic() {
        let mut a: MovAvg<i8, i8, 3> = MovAvg::new();
        a.feed(-100);
        a.feed(-100); // this panics
    }

    #[test]
    fn test_init() {
        let mut a: MovAvg<i32, i32, 3> = MovAvg::new_init([10, 99, 99], 1);
        assert_eq!(a.feed(20), 15);
        assert_eq!(a.feed(102), 44);
        assert_eq!(a.feed(178), 100);

        let mut a: MovAvg<i32, i32, 3> = MovAvg::new_init([10, 20, 0], 2);
        assert_eq!(a.feed(102), 44);
        assert_eq!(a.feed(178), 100);

        let mut a: MovAvg<u16, u16, 3> = MovAvg::new_init([10, 20, 30], 0);
        assert!(a.try_get().is_err());
        assert_eq!(a.feed(50), 50);
        assert_eq!(a.feed(60), (50 + 60) / 2);
        assert_eq!(a.feed(70), (50 + 60 + 70) / 3);
        assert_eq!(a.feed(80), (60 + 70 + 80) / 3);

        let mut a: MovAvg<u16, u16, 3> = MovAvg::new_init([10, 20, 30], 2);
        assert_eq!(a.get(), 15);
        assert_eq!(a.feed(50), (10 + 20 + 50) / 3);
        assert_eq!(a.feed(60), (20 + 50 + 60) / 3);
    }

    #[test]
    fn test_reset() {
        let mut a: MovAvg<i32, i32, 5> = MovAvg::new();
        assert_eq!(a.feed(10), 10);
        assert_eq!(a.feed(20), (10 + 20) / 2);
        assert_eq!(a.feed(2), (10 + 20 + 2) / 3);
        assert_eq!(a.feed(100), (10 + 20 + 2 + 100) / 4);
        assert_eq!(a.feed(111), (10 + 20 + 2 + 100 + 111) / 5);
        assert_eq!(a.feed(200), (20 + 2 + 100 + 111 + 200) / 5);
        assert_eq!(a.feed(250), (2 + 100 + 111 + 200 + 250) / 5);
        assert_eq!(a.feed(-25), (100 + 111 + 200 + 250 - 25) / 5);
        a.reset();
        assert_eq!(a.feed(250), 250);
        assert_eq!(a.feed(-25), (250 - 25) / 2);
        assert_eq!(a.feed(100), (250 - 25 + 100) / 3);
        assert_eq!(a.feed(111), (250 - 25 + 100 + 111) / 4);
        assert_eq!(a.feed(200), (250 - 25 + 100 + 111 + 200) / 5);
        assert_eq!(a.feed(250), (-25 + 100 + 111 + 200 + 250) / 5);
        assert_eq!(a.feed(20), (100 + 111 + 200 + 250 + 20) / 5);
    }

    #[test]
    fn test_get() {
        let mut a: MovAvg<i32, i32, 3> = MovAvg::new_init([10, 20, 0], 2);
        assert_eq!(a.get(), 15);
        assert_eq!(a.feed(102), 44);
        assert_eq!(a.get(), 44);
        assert_eq!(a.feed(178), 100);
        assert_eq!(a.get(), 100);
    }

    #[test]
    fn test_get_empty() {
        let a: MovAvg<i32, i32, 3> = MovAvg::new();
        assert!(a.try_get().is_err());
    }

    #[test]
    #[should_panic(expected="The MovAvg state is empty")]
    fn test_get_empty_panic() {
        let a: MovAvg<i32, i32, 3> = MovAvg::new();
        assert_eq!(a.get(), 42); // this panics
    }

    #[test]
    fn test_initialize_accu() {
        let a: u16 = initialize_accu(&[1_u32, 10_u32, 100_u32, 0_u32, 1000_u32]).unwrap();
        assert_eq!(a, 1111);
    }
}

// vim: ts=4 sw=4 expandtab
