// -*- coding: utf-8 -*-
//
// Copyright 2021-2026 Michael Büsch <m@bues.ch>
//
// Licensed under the Apache License version 2.0
// or the MIT license, at your option.
// SPDX-License-Identifier: Apache-2.0 OR MIT
//

/// Basic arithmetic operations.
pub trait Arith: Copy {
    fn from_usize(v: usize) -> Self;
    fn zero() -> Self;
    fn one() -> Self;
    fn add(self, other: Self) -> Self;
    fn sub(self, other: Self) -> Self;
    fn div(self, other: Self) -> Self;
}

macro_rules! impl_arith {
    (
        $(
            ($ty:ty, $zero:literal, $one:literal $(,)?)
        ),* $(,)?
    ) => {
        $(
            impl Arith for $ty {
                fn from_usize(v: usize) -> Self {
                    v as Self
                }

                fn zero() -> Self {
                    $zero
                }

                fn one() -> Self {
                    $one
                }

                fn add(self, other: Self) -> Self {
                    self + other
                }

                fn sub(self, other: Self) -> Self {
                    self - other
                }

                fn div(self, other: Self) -> Self {
                    self / other
                }
            }
        )*
    }
}

impl_arith!(
    (f32, 0.0, 1.0),
    (f64, 0.0, 1.0),
    (u8, 0, 1),
    (i8, 0, 1),
    (u16, 0, 1),
    (i16, 0, 1),
    (u32, 0, 1),
    (i32, 0, 1),
    (u64, 0, 1),
    (i64, 0, 1),
    (usize, 0, 1),
    (isize, 0, 1),
);

#[cfg(has_i128)]
impl_arith!((u128, 0, 1), (i128, 0, 1),);

// vim: ts=4 sw=4 expandtab
