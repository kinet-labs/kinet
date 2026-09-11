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
use kinet_triedb_utils::triedb_env::Triedb;
use serde::{Deserialize, Serialize};
use tracing::trace;

use crate::{
    data::DataProvider,
    types::{
        eth_json::{
            BlockTagOrHash, BlockTags, EthHash, KinetBlock, KinetTransactionReceipt, Quantity,
        },
        jsonrpc::{ChainStateResultMap, JsonRpcResult},
    },
};

#[rpc(method = "eth_blockNumber")]
#[allow(non_snake_case)]
#[tracing::instrument(level = "debug", skip_all)]
/// Returns the number of most recent block.
pub async fn kinet_eth_blockNumber<T: Triedb>(
    data_provider: &DataProvider<T>,
) -> JsonRpcResult<Quantity> {
    trace!("kinet_eth_blockNumber");

    let block_num = data_provider.get_latest_block_number();
    Ok(Quantity(block_num))
}

#[rpc(method = "eth_chainId", ignore = "chain_id")]
#[allow(non_snake_case)]
#[tracing::instrument(level = "debug")]
/// Returns the chain ID of the current network.
pub async fn kinet_eth_chainId(chain_id: u64) -> JsonRpcResult<Quantity> {
    trace!("kinet_eth_chainId");

    Ok(Quantity(chain_id))
}

#[derive(Deserialize, Debug, schemars::JsonSchema)]
pub struct KinetEthGetBlockByHashParams {
    block_hash: EthHash,
    return_full_txns: bool,
}

#[derive(Serialize, Debug, schemars::JsonSchema)]
pub struct KinetEthGetBlock {
    #[serde(flatten)]
    block: KinetBlock,
}

#[rpc(method = "eth_getBlockByHash")]
#[allow(non_snake_case)]
#[tracing::instrument(level = "debug", skip_all)]
/// Returns information about a block by hash.
pub async fn kinet_eth_getBlockByHash<T: Triedb>(
    data_provider: &DataProvider<T>,
    params: KinetEthGetBlockByHashParams,
) -> JsonRpcResult<Option<KinetEthGetBlock>> {
    trace!("kinet_eth_getBlockByHash: {params:?}");
    data_provider
        .get_block(
            BlockTagOrHash::Hash(params.block_hash),
            params.return_full_txns,
        )
        .await
        .map_present_and_no_err(|block| KinetEthGetBlock {
            block: KinetBlock(block),
        })
}

#[derive(Deserialize, Debug, schemars::JsonSchema)]
pub struct KinetEthGetBlockByNumberParams {
    block_number: BlockTags,
    return_full_txns: bool,
}

#[rpc(method = "eth_getBlockByNumber")]
#[allow(non_snake_case)]
#[tracing::instrument(level = "debug", skip_all)]
/// Returns information about a block by number.
pub async fn kinet_eth_getBlockByNumber<T: Triedb>(
    data_provider: &DataProvider<T>,
    params: KinetEthGetBlockByNumberParams,
) -> JsonRpcResult<Option<KinetEthGetBlock>> {
    trace!("kinet_eth_getBlockByNumber: {params:?}");
    data_provider
        .get_block(
            BlockTagOrHash::BlockTags(params.block_number),
            params.return_full_txns,
        )
        .await
        .map_present_and_no_err(|block| KinetEthGetBlock {
            block: KinetBlock(block),
        })
}

#[derive(Deserialize, Debug, schemars::JsonSchema)]
pub struct KinetEthGetBlockTransactionCountByHashParams {
    block_hash: EthHash,
}

#[rpc(method = "eth_getBlockTransactionCountByHash")]
#[allow(non_snake_case)]
#[tracing::instrument(level = "debug", skip_all)]
/// Returns the number of transactions in a block from a block matching the given block hash.
pub async fn kinet_eth_getBlockTransactionCountByHash<T: Triedb>(
    data_provider: &DataProvider<T>,
    params: KinetEthGetBlockTransactionCountByHashParams,
) -> JsonRpcResult<Option<String>> {
    trace!("kinet_eth_getBlockTransactionCountByHash: {params:?}");
    data_provider
        .get_block(BlockTagOrHash::Hash(params.block_hash), true)
        .await
        .map_present_and_no_err(|block| format!("0x{:x}", block.transactions.len()))
}

#[derive(Deserialize, Debug, schemars::JsonSchema)]
pub struct KinetEthGetBlockTransactionCountByNumberParams {
    block_tag: BlockTags,
}

#[rpc(method = "eth_getBlockTransactionCountByNumber")]
#[allow(non_snake_case)]
#[tracing::instrument(level = "debug", skip_all)]
/// Returns the number of transactions in a block matching the given block number.
pub async fn kinet_eth_getBlockTransactionCountByNumber<T: Triedb>(
    data_provider: &DataProvider<T>,
    params: KinetEthGetBlockTransactionCountByNumberParams,
) -> JsonRpcResult<Option<String>> {
    trace!("kinet_eth_getBlockTransactionCountByNumber: {params:?}");
    data_provider
        .get_block(BlockTagOrHash::BlockTags(params.block_tag), true)
        .await
        .map_present_and_no_err(|block| format!("0x{:x}", block.transactions.len()))
}

#[derive(Deserialize, Debug, schemars::JsonSchema)]
pub struct KinetEthGetBlockReceiptsParams {
    block: BlockTagOrHash,
}

#[derive(Serialize, Debug, schemars::JsonSchema)]
pub struct KinetEthGetBlockReceiptsResult(Vec<KinetTransactionReceipt>);

#[rpc(method = "eth_getBlockReceipts")]
#[allow(non_snake_case)]
#[tracing::instrument(level = "debug", skip_all)]
/// Returns the receipts of a block by number or hash.
pub async fn kinet_eth_getBlockReceipts<T: Triedb>(
    data_provider: &DataProvider<T>,
    params: KinetEthGetBlockReceiptsParams,
) -> JsonRpcResult<Option<KinetEthGetBlockReceiptsResult>> {
    trace!("kinet_eth_getBlockReceipts: {params:?}");

    data_provider
        .get_block_receipts(params.block)
        .await
        .map_present_and_no_err(KinetEthGetBlockReceiptsResult)
}
