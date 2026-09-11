// Copyright (C) 2025 Kinet Labs, Inc.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the Apache-2.0 license as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// Apache-2.0 license for more details.
//
// You should have received a copy of the Apache-2.0 license
// along with this program.  If not, see <http://www.apache.org/licenses//>.

use std::cmp::min;

use kinet_raptor::r10::{CodeParameters, SOURCE_SYMBOLS_MAX, SOURCE_SYMBOLS_MIN};
use rayon::prelude::*;

// Verify that the A matrix is invertible for the given range of num_source_symbols values.
fn check_a_invertible(from: usize, to: usize) {
    (from..=to).into_par_iter().for_each(|num_source_symbols| {
        println!("Testing {}", num_source_symbols);

        let params = CodeParameters::new(num_source_symbols).unwrap();

        let a = params.a_systematic_intermediate_matrix();
        if !a.check_invertible() {
            panic!("Inverting A matrix for K = {} failed", num_source_symbols);
        }
    });
}

#[test]
fn check_a_invertible_small() {
    check_a_invertible(SOURCE_SYMBOLS_MIN, min(128, SOURCE_SYMBOLS_MAX));
}

#[test]
#[ignore]
fn check_a_invertible_all() {
    check_a_invertible(SOURCE_SYMBOLS_MIN, SOURCE_SYMBOLS_MAX);
}
