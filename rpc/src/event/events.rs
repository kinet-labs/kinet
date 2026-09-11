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

use std::sync::Arc;

use kinet_exec_events::BlockCommitState;

use crate::types::{eth_json::KinetNotification, serialize::SharedJsonSerialized};

pub type HeaderNotification = SharedJsonSerialized<KinetNotification<Header>>;
pub type Header = SharedJsonSerialized<alloy_rpc_types::eth::Header>;

pub type BlockTransactions = Arc<Box<[BlockTransaction]>>;
pub type BlockTransaction = (Transaction, TransactionReceipt, Box<[LogNotification]>);

pub type Transaction = SharedJsonSerialized<alloy_rpc_types::Transaction>;
pub type TransactionReceipt = SharedJsonSerialized<alloy_rpc_types::TransactionReceipt>;

pub type LogNotification = SharedJsonSerialized<KinetNotification<Log>>;
pub type Log = SharedJsonSerialized<alloy_rpc_types::eth::Log>;

#[derive(Clone, Debug)]
pub enum EventServerEvent {
    Gap,

    Block {
        commit_state: BlockCommitState,
        header: HeaderNotification,
        transactions: BlockTransactions,
    },
}

#[cfg(test)]
mod test {
    use crate::event::EventServerEvent;

    #[test]
    fn size() {
        assert_eq!(std::mem::size_of::<EventServerEvent>(), 24);
    }
}
