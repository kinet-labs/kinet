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

use std::path::PathBuf;

pub struct EthTxPoolIpcConfig {
    pub bind_path: PathBuf,
    /// Number of txs per batch
    pub tx_batch_size: usize,
    /// Max number of batches to queue
    pub max_queued_batches: usize,
    /// Warn if number of queued batches exceeds this
    pub queued_batches_watermark: usize,
}
