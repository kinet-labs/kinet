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

use std::{pin::pin, time::Duration};

use alloy_consensus::{Transaction as _, TxEnvelope};
use alloy_eips::Decodable2718;
use alloy_primitives::{Address, FixedBytes};
use alloy_rpc_types::Filter;
use kinet_exec_events::BlockCommitState;
use kinet_rpc_docs::rpc;
use kinet_triedb_utils::triedb_env::Triedb;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, trace, warn};

use crate::{
    data::DataProvider,
    event::{EventServerClient, EventServerEvent},
    txpool::{EthTxPoolBridgeClient, TxStatus},
    types::{
        eth_json::{
            BlockTagOrHash, BlockTags, EthHash, KinetLog, KinetTransaction,
            KinetTransactionReceipt, Quantity, UnformattedData,
        },
        jsonrpc::{ChainStateResultMap, ErrorCode, JsonRpcError, JsonRpcResult},
    },
};

pub enum FilterError {
    InvalidBlockRange,
    RangeTooLarge,
}

impl From<FilterError> for JsonRpcError {
    fn from(e: FilterError) -> Self {
        match e {
            FilterError::InvalidBlockRange => {
                JsonRpcError::with_message(ErrorCode::InvalidParams, "invalid block range")
            }
            FilterError::RangeTooLarge => {
                JsonRpcError::with_message(ErrorCode::InvalidParams, "block range too large")
            }
        }
    }
}

#[derive(Serialize, Debug, schemars::JsonSchema)]
pub struct KinetEthGetLogsResult(pub Vec<KinetLog>);

#[derive(Debug, Deserialize, JsonSchema)]
pub struct KinetEthGetLogsParams {
    #[schemars(schema_with = "schema_for_filter")]
    filters: Filter,
}

fn schema_for_filter(_: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
    schemars::schema_for_value!(Filter::new().from_block(0).to_block(1).address(
        "0xAc4b3DacB91461209Ae9d41EC517c2B9Cb1B7DAF"
            .parse::<Address>()
            .unwrap()
    ))
    .schema
    .into()
}

#[rpc(
    method = "eth_getLogs",
    ignore = "max_response_size,max_block_range,use_eth_get_logs_index,dry_run_get_logs_index,max_finalized_block_cache_len"
)]
#[allow(non_snake_case)]
/// Returns an array of all logs matching filter with given id.
#[tracing::instrument(level = "debug", skip_all)]
pub async fn kinet_eth_getLogs<T: Triedb>(
    data_provider: &DataProvider<T>,
    max_response_size: u32,
    max_block_range: u64,
    p: KinetEthGetLogsParams,
    use_eth_get_logs_index: bool,
    dry_run_get_logs_index: bool,
    max_finalized_block_cache_len: u64,
) -> JsonRpcResult<KinetEthGetLogsResult> {
    trace!("kinet_eth_getLogs: {p:?}");

    let KinetEthGetLogsParams { filters } = p;

    let logs = data_provider
        .get_logs(
            filters,
            max_response_size,
            max_block_range,
            use_eth_get_logs_index,
            dry_run_get_logs_index,
            max_finalized_block_cache_len,
        )
        .await?;

    Ok(KinetEthGetLogsResult(logs))
}

#[derive(Deserialize, Debug, schemars::JsonSchema)]
pub struct KinetEthSendRawTransactionParams {
    hex_tx: UnformattedData,
}

// TODO: need to support EIP-4844 transactions
#[rpc(
    method = "eth_sendRawTransaction",
    ignore = "tx_pool,ipc,chain_id,allow_unprotected_txs"
)]
#[allow(non_snake_case)]
#[tracing::instrument(level = "debug", skip_all)]
/// Submits a raw transaction. For EIP-4844 transactions, the raw form must be the network form.
/// This means it includes the blobs, KZG commitments, and KZG proofs.
pub async fn kinet_eth_sendRawTransaction(
    txpool_bridge_client: &EthTxPoolBridgeClient,
    params: KinetEthSendRawTransactionParams,
    chain_id: u64,
    allow_unprotected_txs: bool,
) -> JsonRpcResult<String> {
    trace!("kinet_eth_sendRawTransaction: {params:?}");

    let tx = validate_and_decode_tx(
        &params.hex_tx.0,
        chain_id,
        allow_unprotected_txs,
        JsonRpcError::txn_decode_error,
    )?;

    let tx_hash = *tx.tx_hash();
    debug!(name = "sendRawTransaction", txn_hash = ?tx_hash);
    submit_to_txpool(txpool_bridge_client, tx).await?;

    Ok(tx_hash.to_string())
}

fn validate_and_decode_tx(
    hex_tx: &[u8],
    chain_id: u64,
    allow_unprotected_txs: bool,
    decode_error_fn: impl FnOnce() -> JsonRpcError,
) -> Result<TxEnvelope, JsonRpcError> {
    let tx = TxEnvelope::decode_2718_exact(hex_tx).map_err(|err| {
        debug!(?err, "eth txn decode failed");
        decode_error_fn()
    })?;

    // drop pre EIP-155 transactions if disallowed by the rpc (for user protection purposes)
    if !allow_unprotected_txs && tx.chain_id().is_none() {
        return Err(JsonRpcError::with_message(
            ErrorCode::ServerError,
            "Unprotected transactions (pre-EIP155) are not allowed over RPC".to_string(),
        ));
    }

    if let Some(tx_chain_id) = tx.chain_id() {
        if tx_chain_id != chain_id {
            return Err(JsonRpcError::invalid_chain_id(chain_id, tx_chain_id));
        }
    }

    Ok(tx)
}

async fn submit_to_txpool(
    txpool_bridge_client: &EthTxPoolBridgeClient,
    tx: TxEnvelope,
) -> Result<(), JsonRpcError> {
    let Some(_tx_inflight_guard) = txpool_bridge_client.acquire_tx_inflight_guard() else {
        warn!("txpool overloaded");
        return Err(JsonRpcError::overloaded());
    };

    let (tx_status_recv_send, tx_status_recv_recv) =
        tokio::sync::oneshot::channel::<tokio::sync::watch::Receiver<TxStatus>>();

    if let Err(err) = txpool_bridge_client.try_send(tx, tx_status_recv_send) {
        error!(
            ?err,
            "txpool bridge try_send error after acquiring tx_inflight_guard"
        );
        return Err(JsonRpcError::overloaded());
    }

    let mut tx_status_recv =
        match tokio::time::timeout(Duration::from_secs(1), tx_status_recv_recv).await {
            Ok(Ok(tx_status_recv)) => tx_status_recv,
            Ok(Err(_)) | Err(_) => {
                warn!("txpool bridge not responding, tx status receiver was not sent");
                return Err(JsonRpcError::overloaded());
            }
        };

    match tokio::time::timeout(Duration::from_secs(1), tx_status_recv.changed()).await {
        Ok(Ok(())) => {}
        Ok(Err(_)) => {
            // If the tx_status_send was dropped, then the tx was evicted from RPC state
            return match tx_status_recv.borrow().to_owned() {
                TxStatus::Unknown => Err(JsonRpcError::overloaded()),
                TxStatus::Tracked
                | TxStatus::Dropped { .. }
                | TxStatus::Evicted { .. }
                | TxStatus::Committed => Err(JsonRpcError::with_message(
                    ErrorCode::ServerError,
                    "rpc no longer tracking tx".to_string(),
                )),
            };
        }
        Err(_) => {
            // If the changed future times out, RPC should still try returning whatever status it
            // currently has, even if it might be stale.
            warn!("txpool bridge not responding, tx status has not changed");
        }
    }

    let latest_tx_status = tx_status_recv.borrow_and_update().to_owned();

    match latest_tx_status {
        TxStatus::Evicted { reason: _ } => Err(JsonRpcError::with_message(
            ErrorCode::ServerError,
            "rejected".to_string(),
        )),
        TxStatus::Dropped { reason } => Err(JsonRpcError::with_message(
            ErrorCode::ServerError,
            reason.as_user_string(),
        )),
        TxStatus::Tracked | TxStatus::Committed => Ok(()),
        TxStatus::Unknown => {
            warn!("txpool tx status last value was unknown");
            Err(JsonRpcError::overloaded())
        }
    }
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct KinetEthSendRawTransactionSyncParams {
    hex_tx: UnformattedData,
    #[serde(default)]
    timeout_ms: Option<u64>,
}

#[rpc(
    method = "eth_sendRawTransactionSync",
    ignore = "txpool_bridge_client,event_server_client,chain_id,allow_unprotected_txs,eth_send_raw_transaction_sync_default_timeout_ms,eth_send_raw_transaction_sync_max_timeout_ms"
)]
#[allow(non_snake_case)]
#[tracing::instrument(level = "debug", skip_all)]
pub async fn kinet_eth_sendRawTransactionSync(
    txpool_bridge_client: &EthTxPoolBridgeClient,
    event_server_client: &EventServerClient,
    params: KinetEthSendRawTransactionSyncParams,
    chain_id: u64,
    allow_unprotected_txs: bool,
    eth_send_raw_transaction_sync_default_timeout_ms: u64,
    eth_send_raw_transaction_sync_max_timeout_ms: u64,
) -> JsonRpcResult<KinetTransactionReceipt> {
    trace!("kinet_eth_sendRawTransactionSync: {params:?}");

    let timeout_ms = params
        .timeout_ms
        .filter(|&t| t > 0 && t <= eth_send_raw_transaction_sync_max_timeout_ms)
        .unwrap_or(eth_send_raw_transaction_sync_default_timeout_ms);

    let tx = validate_and_decode_tx(&params.hex_tx.0, chain_id, allow_unprotected_txs, || {
        JsonRpcError::new(ErrorCode::TransactionUnready)
    })?;

    let Ok(mut event_server_subscription) = event_server_client.subscribe() else {
        return Err(JsonRpcError::overloaded());
    };

    let tx_hash = *tx.tx_hash();
    debug!(name = "sendRawTransactionSync", txn_hash = ?tx_hash);
    submit_to_txpool(txpool_bridge_client, tx).await?;

    let mut timeout = pin!(tokio::time::sleep(Duration::from_millis(timeout_ms)));

    loop {
        let result = tokio::select! {
            result = event_server_subscription.recv() => result,

            () = &mut timeout => {
                // EIP-7966: Error code 4 with tx hash in data
                return Err(JsonRpcError::tx_sync_timeout(
                    tx_hash.to_string(),
                    timeout_ms,
                ));
            }
        };

        let Ok(event) = result else {
            return Err(JsonRpcError::overloaded());
        };

        match event {
            EventServerEvent::Gap => {
                return Err(JsonRpcError::overloaded());
            }
            EventServerEvent::Block {
                commit_state,
                header: _,
                transactions,
            } => {
                if commit_state != BlockCommitState::Proposed {
                    continue;
                }

                for (tx, tx_receipt, _) in transactions.iter() {
                    if tx.value().inner.tx_hash() != &tx_hash {
                        continue;
                    }

                    return Ok(KinetTransactionReceipt(tx_receipt.value().clone()));
                }
            }
        }
    }
}

#[derive(Deserialize, Debug, schemars::JsonSchema)]
pub struct KinetEthGetTransactionReceiptParams {
    tx_hash: EthHash,
}

#[rpc(method = "eth_getTransactionReceipt")]
#[allow(non_snake_case)]
/// Returns the receipt of a transaction by transaction hash.
#[tracing::instrument(level = "debug", skip_all)]
pub async fn kinet_eth_getTransactionReceipt<T: Triedb>(
    data_provider: &DataProvider<T>,
    params: KinetEthGetTransactionReceiptParams,
) -> JsonRpcResult<Option<KinetTransactionReceipt>> {
    trace!("kinet_eth_getTransactionReceipt: {params:?}");

    data_provider
        .get_transaction_receipt(&FixedBytes(params.tx_hash.0))
        .await
        .map_present_and_no_err(KinetTransactionReceipt)
}

#[derive(Deserialize, Debug, schemars::JsonSchema)]
pub struct KinetEthGetTransactionByHashParams {
    tx_hash: EthHash,
}

#[rpc(method = "eth_getTransactionByHash")]
#[allow(non_snake_case)]
/// Returns the information about a transaction requested by transaction hash.
#[tracing::instrument(level = "debug", skip_all)]
pub async fn kinet_eth_getTransactionByHash<T: Triedb>(
    data_provider: &DataProvider<T>,
    params: KinetEthGetTransactionByHashParams,
) -> JsonRpcResult<Option<KinetTransaction>> {
    trace!("kinet_eth_getTransactionByHash: {params:?}");

    data_provider
        .get_transaction(&FixedBytes(params.tx_hash.0))
        .await
        .map_present_and_no_err(KinetTransaction)
}

#[derive(Deserialize, Debug, schemars::JsonSchema)]
pub struct KinetEthGetTransactionByBlockHashAndIndexParams {
    block_hash: EthHash,
    index: Quantity,
}

#[rpc(method = "eth_getTransactionByBlockHashAndIndex")]
#[allow(non_snake_case)]
#[tracing::instrument(level = "debug", skip_all)]
/// Returns information about a transaction by block hash and transaction index position.
pub async fn kinet_eth_getTransactionByBlockHashAndIndex<T: Triedb>(
    data_provider: &DataProvider<T>,
    params: KinetEthGetTransactionByBlockHashAndIndexParams,
) -> JsonRpcResult<Option<KinetTransaction>> {
    trace!("kinet_eth_getTransactionByBlockHashAndIndex: {params:?}");

    data_provider
        .get_transaction_with_block_and_index(
            BlockTagOrHash::Hash(params.block_hash),
            params.index.0,
        )
        .await
        .map_present_and_no_err(KinetTransaction)
}

#[derive(Deserialize, Debug, schemars::JsonSchema)]
pub struct KinetEthGetTransactionByBlockNumberAndIndexParams {
    block_tag: BlockTags,
    index: Quantity,
}

#[rpc(method = "eth_getTransactionByBlockNumberAndIndex")]
#[allow(non_snake_case)]
#[tracing::instrument(level = "debug", skip_all)]
/// Returns information about a transaction by block number and transaction index position.
pub async fn kinet_eth_getTransactionByBlockNumberAndIndex<T: Triedb>(
    data_provider: &DataProvider<T>,
    params: KinetEthGetTransactionByBlockNumberAndIndexParams,
) -> JsonRpcResult<Option<KinetTransaction>> {
    trace!("kinet_eth_getTransactionByBlockNumberAndIndex: {params:?}");

    data_provider
        .get_transaction_with_block_and_index(
            crate::types::eth_json::BlockTagOrHash::BlockTags(params.block_tag),
            params.index.0,
        )
        .await
        .map_present_and_no_err(KinetTransaction)
}

#[cfg(test)]
mod tests {
    use alloy_consensus::{SignableTransaction, TxEip1559, TxEnvelope};
    use alloy_eips::eip2718::Encodable2718;
    use alloy_primitives::{Address, FixedBytes, TxKind};
    use alloy_rlp::Encodable;
    use alloy_signer::SignerSync;
    use alloy_signer_local::PrivateKeySigner;
    use kinet_eth_types::EthAccount;
    use kinet_event_ring::SnapshotEventRing;
    use kinet_triedb_utils::mock_triedb::MockTriedb;

    use super::{
        kinet_eth_sendRawTransaction, kinet_eth_sendRawTransactionSync,
        KinetEthSendRawTransactionParams, KinetEthSendRawTransactionSyncParams,
    };
    use crate::{
        event::EventServer, txpool::EthTxPoolBridgeClient, types::eth_json::UnformattedData,
    };

    fn serialize_tx(tx: impl Encodable + Encodable2718) -> UnformattedData {
        let mut rlp_encoded_tx = Vec::new();
        tx.encode_2718(&mut rlp_encoded_tx);
        UnformattedData(rlp_encoded_tx)
    }

    fn make_tx(
        sender: FixedBytes<32>,
        max_fee_per_gas: u128,
        max_priority_fee_per_gas: u128,
        gas_limit: u64,
        nonce: u64,
        chain_id: u64,
    ) -> TxEnvelope {
        let transaction = TxEip1559 {
            chain_id,
            nonce,
            gas_limit,
            max_fee_per_gas,
            max_priority_fee_per_gas,
            to: TxKind::Call(Address::repeat_byte(0u8)),
            value: Default::default(),
            access_list: Default::default(),
            input: vec![].into(),
        };

        let signer = PrivateKeySigner::from_bytes(&sender).unwrap();
        let signature = signer
            .sign_hash_sync(&transaction.signature_hash())
            .unwrap();
        transaction.into_signed(signature).into()
    }

    #[tokio::test]
    async fn eth_send_raw_transaction() {
        let mut triedb = MockTriedb::default();
        let sender = FixedBytes::<32>::from([1u8; 32]);
        let signer = PrivateKeySigner::from_bytes(&sender).unwrap();

        triedb.set_account(
            signer.address().0.into(),
            EthAccount {
                nonce: 10,
                ..Default::default()
            },
        );

        let expected_failures = [
            KinetEthSendRawTransactionParams {
                hex_tx: serialize_tx(make_tx(sender, 1000, 1000, 21_000, 11, 1337)), // invaid chain id
            },
            KinetEthSendRawTransactionParams {
                hex_tx: serialize_tx(make_tx(sender, 1000, 1000, 1_000, 11, 1)), // intrinsic gas too low
            },
            KinetEthSendRawTransactionParams {
                hex_tx: serialize_tx(make_tx(sender, 1000, 1000, 400_000_000_000, 11, 1)), // gas too high
            },
            KinetEthSendRawTransactionParams {
                hex_tx: serialize_tx(make_tx(sender, 1000, 1000, 21_000, 1, 1)), // nonce too low
            },
            KinetEthSendRawTransactionParams {
                hex_tx: serialize_tx(make_tx(sender, 1000, 12000, 21_000, 11, 1)), // max priority fee too high
            },
        ];

        for (idx, case) in expected_failures.into_iter().enumerate() {
            assert!(
                kinet_eth_sendRawTransaction(&EthTxPoolBridgeClient::for_testing(), case, 1, true)
                    .await
                    .is_err(),
                "Expected error for case: {:?}",
                idx + 1
            );
        }
    }

    #[tokio::test]
    async fn eth_send_raw_transaction_sync() {
        let mut triedb = MockTriedb::default();
        let sender = FixedBytes::<32>::from([1u8; 32]);
        let signer = PrivateKeySigner::from_bytes(&sender).unwrap();

        triedb.set_account(
            signer.address().0.into(),
            EthAccount {
                nonce: 10,
                ..Default::default()
            },
        );

        let snapshot_event_ring = SnapshotEventRing::new_from_zstd_bytes(
            "TEST",
            include_bytes!(
                "../../../../kinet-execution/rust/crates/kinet-exec-events/test/data/exec-events-emn-30b-15m/snapshot.zst"
            ),
            None,
        )
        .unwrap();

        let event_server_client = EventServer::start_for_testing(snapshot_event_ring);

        // Test the same validation failures as eth_sendRawTransaction
        // to ensure both methods have consistent validation
        let expected_failures = [
            KinetEthSendRawTransactionSyncParams {
                hex_tx: serialize_tx(make_tx(sender, 1000, 1000, 21_000, 11, 1337)), // invalid chain id
                timeout_ms: Some(2000),
            },
            KinetEthSendRawTransactionSyncParams {
                hex_tx: serialize_tx(make_tx(sender, 1000, 1000, 1_000, 11, 1)), // intrinsic gas too low
                timeout_ms: Some(2000),
            },
            KinetEthSendRawTransactionSyncParams {
                hex_tx: serialize_tx(make_tx(sender, 1000, 1000, 400_000_000_000, 11, 1)), // gas too high
                timeout_ms: Some(2000),
            },
            KinetEthSendRawTransactionSyncParams {
                hex_tx: serialize_tx(make_tx(sender, 1000, 1000, 21_000, 1, 1)), // nonce too low
                timeout_ms: Some(2000),
            },
            KinetEthSendRawTransactionSyncParams {
                hex_tx: serialize_tx(make_tx(sender, 1000, 12000, 21_000, 11, 1)), // max priority fee too high
                timeout_ms: Some(2000),
            },
        ];

        for (idx, case) in expected_failures.into_iter().enumerate() {
            assert!(
                kinet_eth_sendRawTransactionSync(
                    &EthTxPoolBridgeClient::for_testing(),
                    &event_server_client,
                    case,
                    1,
                    true,
                    2000,
                    30000,
                )
                .await
                .is_err(),
                "Expected error for case: {:?}",
                idx + 1
            );
        }
    }
}
