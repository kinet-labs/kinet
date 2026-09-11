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

//! Top level exports are available under the specific implementation
//! variants at crate::{stub,prod}::*. For the module system
//! mechanism, refer to the moduledoc at env/mod.rs.

#![allow(clippy::duplicate_mod)]

mod env;

// the specification for types used in cadence.
pub mod spec;

#[cfg(feature = "enable_stub")]
// override to top-level path
#[path = ""]
pub mod stub {
    // cadence implementation with all stub types, intended for testing
    pub use crate::env::stub as env;

    #[path = "top_level.rs"]
    mod top_level;
    pub use top_level::*;
}

// override to top-level path
#[path = ""]
pub mod prod {
    // cadence implementation with production types
    pub use crate::env::prod as env;

    #[path = "top_level.rs"]
    mod top_level;
    pub use top_level::*;
}
