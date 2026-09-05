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
    fn from_usize(v: usize) -> Option<Self>;
    fn zero() -> Self;
    fn one() -> Self;
    fn add(self, other: Self) -> Self;
    fn sub(self, other: Self) -> Self;
    fn div(self, other: Self) -> Self;
}

macro_rules! impl_arith_int {
    ($( $ty:ty ),* $(,)?) => {
        $(
            impl Arith for $ty {
                #[inline(always)]
                fn from_usize(v: usize) -> Option<Self> {
                    v.try_into().ok()
                }

                #[inline(always)]
                fn zero() -> Self {
                    0
                }

                #[inline(always)]
                fn one() -> Self {
                    1
                }

                #[inline(always)]
                fn add(self, other: Self) -> Self {
                    self + other
                }

                #[inline(always)]
                fn sub(self, other: Self) -> Self {
                    self - other
                }

                #[inline(always)]
                fn div(self, other: Self) -> Self {
                    self / other
                }
            }
        )*
    }
}

macro_rules! impl_arith_float {
    ($( $ty:ty ),* $(,)?) => {
        $(
            impl Arith for $ty {
                #[inline(always)]
                fn from_usize(v: usize) -> Option<Self> {
                    Some(v as Self)
                }

                #[inline(always)]
                fn zero() -> Self {
                    0.0
                }

                #[inline(always)]
                fn one() -> Self {
                    1.0
                }

                #[inline(always)]
                fn add(self, other: Self) -> Self {
                    #[cfg(any(not(feature = "fastfloat"), not(rustc_1_98)))]
                    {
                        self + other
                    }
                    #[cfg(all(feature = "fastfloat", rustc_1_98))]
                    {
                        self.algebraic_add(other)
                    }
                }

                #[inline(always)]
                fn sub(self, other: Self) -> Self {
                    #[cfg(any(not(feature = "fastfloat"), not(rustc_1_98)))]
                    {
                        self - other
                    }
                    #[cfg(all(feature = "fastfloat", rustc_1_98))]
                    {
                        self.algebraic_sub(other)
                    }
                }

                #[inline(always)]
                fn div(self, other: Self) -> Self {
                    #[cfg(any(not(feature = "fastfloat"), not(rustc_1_98)))]
                    {
                        self / other
                    }
                    #[cfg(all(feature = "fastfloat", rustc_1_98))]
                    {
                        self.algebraic_div(other)
                    }
                }
            }
        )*
    }
}

impl_arith_float!(f32, f64);
impl_arith_int!(u8, u16, u32, u64, usize);
impl_arith_int!(i8, i16, i32, i64, isize);
#[cfg(has_i128)]
impl_arith_int!(u128, i128);

// vim: ts=4 sw=4 expandtab
