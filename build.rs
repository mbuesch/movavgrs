// -*- coding: utf-8 -*-
//
// Copyright 2021-2026 Michael Büsch <m@bues.ch>
//
// Licensed under the Apache License version 2.0
// or the MIT license, at your option.
// SPDX-License-Identifier: Apache-2.0 OR MIT
//

fn main() {
    let ac = autocfg::new();

    ac.emit_has_type("i128");
    println!("cargo:rustc-check-cfg=cfg(has_i128)");

    ac.emit_rustc_version(1, 98);
    println!("cargo:rustc-check-cfg=cfg(rustc_1_98)");

    autocfg::rerun_path("build.rs");
}

// vim: ts=4 sw=4 expandtab
