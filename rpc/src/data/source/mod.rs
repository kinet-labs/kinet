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

use kinet_types::BlockId;

pub use self::{
    archive::ArchiveDataSource,
    historical::{HistoricalDataSource, HistoricalDataSourceExt, HistoricalDataSourceStack},
    stack::DataSourceStack,
    triedb::TriedbDataSource,
};

mod archive;
mod historical;
mod stack;
mod triedb;

pub type DataSourceResult<T> = Result<T, DataSourceError>;

#[derive(Debug, thiserror::Error)]
pub enum DataSourceError {
    #[error("data source error: {0}")]
    Internal(String),
}

#[derive(Clone, Copy, Debug)]
pub enum BlockCommitState {
    Proposed,
    Voted,
    Finalized,
}

/// A resolved block reference that has been successfully located by a data source.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BlockPointer {
    Finalized(u64),
    NonFinalized(u64, BlockId),
}

/// Canonical block representation shared by historical data sources.
#[derive(Clone, Debug)]
pub struct HistoricalBlockData {
    pub header: alloy_consensus::Header,
    pub header_hash_precomputed: Option<alloy_primitives::BlockHash>,

    pub transactions: Vec<kinet_eth_types::TxEnvelopeWithSender>,
}
