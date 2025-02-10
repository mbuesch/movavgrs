movavg - Generic Moving Average calculation
===========================================

`Project home <https://bues.ch/>`_

`Git repository <https://bues.ch/cgit/movavgrs.git>`_

`Github repository <https://github.com/mbuesch/movavgrs>`_

Generic `Moving Average <https://en.wikipedia.org/wiki/Moving_average>`_ calculation for the integer types

* i8, i16, i32, i64, i128, isize
* u8, u16, u32, u64, u128, usize

and float types

* f32, f64


Example Cargo.toml dependencies
===============================

Add this to your Cargo.toml:

.. code:: toml

	[dependencies]
	movavg = "2"


Example usage
=============

.. code:: rust

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


Cargo Feature selections
========================

no_std
------

If you want to use movavg without the `std` library (often called `no_std`), then use the following Cargo.toml dependency to disable the `std` feature:

.. code:: toml

	[dependencies]
	movavg = { version = "2", default-features = false }

Currently the `no_std` variant supports all functionality that the default `std` variant supports. But that may change in future.

fastfloat
---------

The `fastfloat` feature can be used to enable much faster, but less accurate floating point calculations. Enabling this feature leads to bigger floating point rounding and cancellation errors.

.. code:: toml

	[dependencies]
	movavg = { version = "2", features = ["fastfloat"] }

This feature may also be used together with disabled `std` feature (see `no_std`).


Rust compiler version
=====================

Requires Rust compiler version 1.61 or later.


License
=======

Copyright (c) 2021-2025 Michael Büsch <m@bues.ch>

Licensed under the Apache License version 2.0 or the MIT license, at your option.
