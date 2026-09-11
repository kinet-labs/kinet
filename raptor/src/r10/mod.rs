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

// Copies of various lookup tables and implementations of various parameter generation
// functions for the Raptor code described in RFC 5053.

pub mod a;
pub mod degree;
pub mod half;
pub mod ldpc;
pub mod lt;
pub mod matrix;
pub mod nonsystematic;
pub mod parameters;
pub mod rand;
pub mod systematic_index;

pub use degree::{deg, MAX_DEGREE};
pub use parameters::{CodeParameters, SOURCE_SYMBOLS_MAX, SOURCE_SYMBOLS_MIN};
pub use rand::rand;
