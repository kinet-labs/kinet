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
use serde::Deserialize;
use tracing::trace;

use crate::{
    data::{get_block_key_from_tag_or_hash, DataProvider},
    types::{
        eth_json::{BlockTagOrHash, EthAddress, StorageKey},
        jsonrpc::{JsonRpcError, JsonRpcResult},
    },
};

#[derive(Deserialize, Debug, schemars::JsonSchema)]
pub struct KinetEthGetBalanceParams {
    account: EthAddress,
    block_number: BlockTagOrHash,
}

#[rpc(method = "eth_getBalance")]
#[allow(non_snake_case)]
/// Returns the balance of the account of given address.
pub async fn kinet_eth_getBalance<T: Triedb>(
    data_provider: &DataProvider<T>,
    params: KinetEthGetBalanceParams,
) -> JsonRpcResult<String> {
    trace!("kinet_eth_getBalance: {params:?}");

    let block_key = get_block_key_from_tag_or_hash(&data_provider.triedb_env, params.block_number)
        .await
        .ok_or_else(JsonRpcError::block_not_found)?;
    let account = data_provider
        .triedb_env
        .get_account(block_key, params.account.0)
        .await
        .map_err(JsonRpcError::internal_error)?;

    match data_provider
        .triedb_env
        .get_state_availability(block_key)
        .await
        .map_err(JsonRpcError::internal_error)?
    {
        true => Ok(format!("0x{:x}", account.balance)),
        false => Err(JsonRpcError::block_not_found()),
    }
}

#[derive(Deserialize, Debug, schemars::JsonSchema)]
pub struct KinetEthGetCodeParams {
    account: EthAddress,
    block: BlockTagOrHash,
}

#[rpc(method = "eth_getCode")]
#[allow(non_snake_case)]
/// Returns code at a given address.
pub async fn kinet_eth_getCode<T: Triedb>(
    data_provider: &DataProvider<T>,
    params: KinetEthGetCodeParams,
) -> JsonRpcResult<String> {
    trace!("kinet_eth_getCode: {params:?}");

    let block_key = get_block_key_from_tag_or_hash(&data_provider.triedb_env, params.block)
        .await
        .ok_or_else(JsonRpcError::block_not_found)?;
    let account = data_provider
        .triedb_env
        .get_account(block_key, params.account.0)
        .await
        .map_err(JsonRpcError::internal_error)?;

    let code = if let Some(code_hash) = account.code_hash {
        data_provider
            .triedb_env
            .get_code(block_key, code_hash)
            .await
            .map_err(JsonRpcError::internal_error)?
    } else {
        Vec::default()
    };

    match data_provider
        .triedb_env
        .get_state_availability(block_key)
        .await
        .map_err(JsonRpcError::internal_error)?
    {
        true => Ok(format!("0x{}", hex::encode(code))),
        false => Err(JsonRpcError::block_not_found()),
    }
}

#[derive(Deserialize, Debug, schemars::JsonSchema)]
pub struct KinetEthGetStorageAtParams {
    account: EthAddress,
    position: StorageKey,
    block: BlockTagOrHash,
}

#[rpc(method = "eth_getStorageAt")]
#[allow(non_snake_case)]
/// Returns the value from a storage position at a given address.
pub async fn kinet_eth_getStorageAt<T: Triedb>(
    data_provider: &DataProvider<T>,
    params: KinetEthGetStorageAtParams,
) -> JsonRpcResult<String> {
    trace!("kinet_eth_getStorageAt: {params:?}");

    let block_key = get_block_key_from_tag_or_hash(&data_provider.triedb_env, params.block)
        .await
        .ok_or_else(JsonRpcError::block_not_found)?;
    let storage_value = data_provider
        .triedb_env
        .get_storage_at(block_key, params.account.0, params.position.0)
        .await
        .map_err(JsonRpcError::internal_error)?;

    match data_provider
        .triedb_env
        .get_state_availability(block_key)
        .await
        .map_err(JsonRpcError::internal_error)?
    {
        true => Ok(format!("0x{}", hex::encode(storage_value))),
        false => Err(JsonRpcError::block_not_found()),
    }
}

#[derive(Deserialize, Debug, schemars::JsonSchema)]
pub struct KinetEthGetTransactionCountParams {
    account: EthAddress,
    block: BlockTagOrHash,
}

#[rpc(method = "eth_getTransactionCount")]
#[allow(non_snake_case)]
/// Returns the number of transactions sent from an address.
pub async fn kinet_eth_getTransactionCount<T: Triedb>(
    data_provider: &DataProvider<T>,
    params: KinetEthGetTransactionCountParams,
) -> JsonRpcResult<String> {
    trace!("kinet_eth_getTransactionCount: {params:?}");

    let block_key = get_block_key_from_tag_or_hash(&data_provider.triedb_env, params.block)
        .await
        .ok_or_else(JsonRpcError::block_not_found)?;
    let account = data_provider
        .triedb_env
        .get_account(block_key, params.account.0)
        .await
        .map_err(JsonRpcError::internal_error)?;

    match data_provider
        .triedb_env
        .get_state_availability(block_key)
        .await
        .map_err(JsonRpcError::internal_error)?
    {
        true => Ok(format!("0x{:x}", account.nonce)),
        false => Err(JsonRpcError::block_not_found()),
    }
}

#[allow(non_snake_case)]
/// Returns an object with data about the sync status or false.
#[rpc(method = "eth_syncing")]
pub async fn kinet_eth_syncing() -> JsonRpcResult<bool> {
    trace!("kinet_eth_syncing");

    Ok(false)
}

#[cfg(test)]
mod tests {
    use super::KinetEthGetStorageAtParams;
    use crate::types::eth_json::{BlockTags, Quantity};

    #[test]
    fn params_without_eip_1898() {
        let res: KinetEthGetStorageAtParams = serde_json::from_str(
            r#"["0xDeaDbeefdEAdbeefdEadbEEFdeadbeEFdEaDbeeF", "0x0", "latest"]"#,
        )
        .unwrap();
        assert!(matches!(
            res.block,
            super::BlockTagOrHash::BlockTags(BlockTags::Latest)
        ));
        let res: KinetEthGetStorageAtParams =
            serde_json::from_str(r#"["0xDeaDbeefdEAdbeefdEadbEEFdeadbeEFdEaDbeeF", "0x0", "0x1"]"#)
                .unwrap();
        assert!(matches!(
            res.block,
            super::BlockTagOrHash::BlockTags(BlockTags::Number(Quantity(1)))
        ));
    }

    #[test]
    fn eip_1898_blockhash() {
        let res: KinetEthGetStorageAtParams = serde_json::from_str(r#"["0xDeaDbeefdEAdbeefdEadbEEFdeadbeEFdEaDbeeF", "0x0", {"blockHash": "0xd4e56740f876aef8c010b86a40d5f56745a118d0906a34e69aec8c0db1cb8fa3"}]"#).unwrap();
        assert!(matches!(res.block, super::BlockTagOrHash::Hash(_)));
        let res: KinetEthGetStorageAtParams = serde_json::from_str(r#"["0xDeaDbeefdEAdbeefdEadbEEFdeadbeEFdEaDbeeF", "0x0", {"blockHash": "0xd4e56740f876aef8c010b86a40d5f56745a118d0906a34e69aec8c0db1cb8fa3", "requireCanonical": false}]"#).unwrap();
        assert!(matches!(res.block, super::BlockTagOrHash::Hash(_)));
        let res: KinetEthGetStorageAtParams = serde_json::from_str(r#"["0xDeaDbeefdEAdbeefdEadbEEFdeadbeEFdEaDbeeF", "0x0", {"blockHash": "0xd4e56740f876aef8c010b86a40d5f56745a118d0906a34e69aec8c0db1cb8fa3", "requireCanonical": true}]"#).unwrap();
        assert!(matches!(res.block, super::BlockTagOrHash::Hash(_)));
    }

    #[test]
    fn eip_1898_blocknumber() {
        let res: KinetEthGetStorageAtParams = serde_json::from_str(
            r#"["0xDeaDbeefdEAdbeefdEadbEEFdeadbeEFdEaDbeeF", "0x0", {"blockNumber": "0x0"}]"#,
        )
        .unwrap();
        assert!(matches!(
            res.block,
            super::BlockTagOrHash::BlockTags(BlockTags::Number(Quantity(0)))
        ));
        let res: KinetEthGetStorageAtParams = serde_json::from_str(
            r#"["0xDeaDbeefdEAdbeefdEadbEEFdeadbeEFdEaDbeeF", "0x0", {"blockNumber": "latest"}]"#,
        )
        .unwrap();
        assert!(matches!(
            res.block,
            super::BlockTagOrHash::BlockTags(BlockTags::Latest)
        ));
    }

    #[test]
    fn storage_key_valid_and_right_aligned() {
        let mut expected = [0u8; 32];
        expected[31] = 1;

        let res: KinetEthGetStorageAtParams = serde_json::from_str(
            r#"["0xDeaDbeefdEAdbeefdEadbEEFdeadbeEFdEaDbeeF", "0x0000000000000000000000000000000000000000000000000000000000000001", "latest"]"#,
        )
        .unwrap();
        assert_eq!(res.position.0, expected);

        let res: KinetEthGetStorageAtParams = serde_json::from_str(
            r#"["0xDeaDbeefdEAdbeefdEadbEEFdeadbeEFdEaDbeeF", "0x1", "latest"]"#,
        )
        .unwrap();
        assert_eq!(res.position.0, expected);
    }

    #[test]
    fn storage_key_too_many_hex_digits_rejected() {
        assert!(serde_json::from_str::<KinetEthGetStorageAtParams>(
            r#"["0xaa00000000000000000000000000000000000000", "0x00000000000000000000000000000000000000000000000000000000000000000", "latest"]"#,
        )
        .is_err());
    }

    #[test]
    fn storage_key_invalid_hex_rejected() {
        assert!(serde_json::from_str::<KinetEthGetStorageAtParams>(
            r#"["0xaa00000000000000000000000000000000000000", "0xasdf", "latest"]"#,
        )
        .is_err());
    }
}
