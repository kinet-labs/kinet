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

use kinet_rpc_docs::rpc;

use crate::{types::jsonrpc::JsonRpcResult, KINET_RPC_VERSION};

#[rpc(method = "net_version", ignore = "chain_id")]
pub fn kinet_net_version(chain_id: u64) -> JsonRpcResult<String> {
    Ok(chain_id.to_string())
}

#[rpc(method = "web3_clientVersion")]
pub fn kinet_web3_client_version() -> JsonRpcResult<String> {
    Ok(format!("Kinet/{}", KINET_RPC_VERSION.unwrap_or("unknown")))
}
