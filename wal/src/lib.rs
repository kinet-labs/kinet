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

pub mod reader;
pub mod wal;

use std::{error::Error, fmt::Debug, io};

extern crate self as kinet_wal;

pub use kinet_wal_derive::WALLog;

#[derive(Debug)]
pub enum WALError {
    IOError(io::Error),
    DeserError(Box<dyn Error>),
}

impl From<io::Error> for WALError {
    fn from(value: io::Error) -> Self {
        Self::IOError(value)
    }
}

impl std::fmt::Display for WALError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <Self as Debug>::fmt(self, f)
    }
}

impl std::error::Error for WALError {}
