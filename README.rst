movavg - Generic Moving Average calculation
===========================================

`https://bues.ch/ <https://bues.ch/>`_

Generic `Moving Average <https://en.wikipedia.org/wiki/Moving_average>`_ calculation for the integer types

* i8, i16, i32, i64, i128, isize
* u8, u16, u32, u64, u128, usize

and float types

* f32, f64


Examples
========

.. code:: rust

	// Integers
	let mut avg: MovAvg<i32> = MovAvg::new(3); // window size = 3
	assert_eq!(avg.feed(10), 10);
	assert_eq!(avg.feed(20), 15);
	assert_eq!(avg.feed(30), 20);
	assert_eq!(avg.feed(40), 30);

	// Floats
	let mut avg: MovAvg<f64> = MovAvg::new(3);
	assert_eq!(avg.feed(10.0), 10.0);
	assert_eq!(avg.feed(20.0), 15.0);
	assert_eq!(avg.feed(30.0), 20.0);
	assert_eq!(avg.feed(40.0), 30.0);

	// Bigger accumulator
	let mut avg: MovAvg<i8, i32> = MovAvg::new(3);
	assert_eq!(avg.feed(100), 100);
	assert_eq!(avg.feed(100), 100); // This would overflow an i8 accumulator


License
=======

Copyright (c) 2021 Michael Buesch <m@bues.ch>

Licensed under the Apache License version 2.0 or the MIT license, at your option.
