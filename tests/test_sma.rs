// -*- coding: utf-8 -*-
//
// Copyright 2021 Michael Büsch <m@bues.ch>
//
// Licensed under the Apache License version 2.0
// or the MIT license, at your option.
// SPDX-License-Identifier: Apache-2.0 OR MIT
//

extern crate movavg;
use movavg::MovAvg;

#[test]
fn test_sma() {
    // Integers
    let mut avg: MovAvg<i32, i32, 3> = MovAvg::new(); // window size = 3
    assert_eq!(avg.feed(10), 10);
    assert_eq!(avg.feed(20), 15);
    assert_eq!(avg.feed(30), 20);
    assert_eq!(avg.feed(40), 30);
    assert_eq!(avg.get(), 30);

    // Floats
    let mut avg: MovAvg<f64, f64, 3> = MovAvg::new();
    assert_eq!(avg.feed(10.0), 10.0);
    assert_eq!(avg.feed(20.0), 15.0);
    assert_eq!(avg.feed(30.0), 20.0);
    assert_eq!(avg.feed(40.0), 30.0);
    assert_eq!(avg.get(), 30.0);

    // Bigger accumulator
    let mut avg: MovAvg<i8, i32, 3> = MovAvg::new();
    assert_eq!(avg.feed(100), 100);
    assert_eq!(avg.feed(100), 100); // This would overflow an i8 accumulator
}

// vim: ts=4 sw=4 expandtab
