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

pub mod metrics;
mod pool;
mod queue;

use std::fmt::{Debug, Display};

pub use kinet_peer_score::{IdentityScore, PeerStatus, Score};

macro_rules! ensure {
    ($condition:expr, $error:expr $(,)?) => {
        if !$condition {
            return Err($error);
        }
    };
}
pub(crate) use ensure;
pub use queue::{FairQueue, FairQueueBuilder};

#[derive(Debug, Clone, thiserror::Error)]
pub enum PushError<Id: Debug + Display> {
    #[error("per-id limit exceeded for {id}: {limit}")]
    PerIdLimitExceeded { id: Id, limit: usize },
    #[error("queue full: {size}/{max_size}")]
    Full { size: usize, max_size: usize },
}
