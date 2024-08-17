// -*- coding: utf-8 -*-
//
// Copyright 2021-2023 Michael Büsch <m@bues.ch>
//
// Licensed under the Apache License version 2.0
// or the MIT license, at your option.
// SPDX-License-Identifier: Apache-2.0 OR MIT
//

//! # Moving Average
//!
//! Generic Moving Average calculation for the integer types
//!
//! * i8, i16, i32, i64, i128, isize
//! * u8, u16, u32, u64, u128, usize
//!
//! and float types
//!
//! * f32, f64
//!
//! # Cargo Features
//!
//! * `std` - If the cargo feature `std` is given, then all features that depend on
//!           the `std` library are enabled. This feature is enabled by default.
//!           Use `default-features = false` in your `Cargo.toml` to disable this feature.
//!           This crate is independent of the `std` library, if this feature is disabled.

#![no_std]
#[cfg(feature = "std")]
extern crate std;

mod sma;

pub use sma::{MovAvg, MovAvgAccu};

// vim: ts=4 sw=4 expandtab
