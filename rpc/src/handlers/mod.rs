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

use actix_web::{web, HttpResponse};
use futures::StreamExt;
use kinet_tracing_timing::TimingSpanExtension;
use kinet_triedb_utils::triedb_env::Triedb;
use serde_json::value::RawValue;
use tracing::{debug, trace_span, Instrument, Span};
use tracing_actix_web::RootSpan;

use self::{
    debug::{
        kinet_debug_getRawBlock, kinet_debug_getRawHeader, kinet_debug_getRawReceipts,
        kinet_debug_getRawTransaction, kinet_debug_traceBlockByHash,
        kinet_debug_traceBlockByNumber, kinet_debug_traceTransaction,
    },
    debug_replay::{collect_debug_trace_via_replay, DebugTraceParams},
    eth::{
        account::{
            kinet_eth_getBalance, kinet_eth_getCode, kinet_eth_getStorageAt,
            kinet_eth_getTransactionCount, kinet_eth_syncing,
        },
        block::{
            kinet_eth_blockNumber, kinet_eth_chainId, kinet_eth_getBlockByHash,
            kinet_eth_getBlockByNumber, kinet_eth_getBlockReceipts,
            kinet_eth_getBlockTransactionCountByHash, kinet_eth_getBlockTransactionCountByNumber,
        },
        call::{kinet_admin_ethCallStatistics, kinet_debug_traceCall, kinet_eth_call},
        gas::{
            kinet_eth_estimateGas, kinet_eth_feeHistory, kinet_eth_fillTransaction,
            kinet_eth_gasPrice, kinet_eth_maxPriorityFeePerGas,
        },
        simulate::kinet_simulate_v1,
        txn::{
            kinet_eth_getLogs, kinet_eth_getTransactionByBlockHashAndIndex,
            kinet_eth_getTransactionByBlockNumberAndIndex, kinet_eth_getTransactionByHash,
            kinet_eth_getTransactionReceipt, kinet_eth_sendRawTransaction,
            kinet_eth_sendRawTransactionSync,
        },
    },
    meta::{kinet_net_version, kinet_web3_client_version},
    resources::KinetRpcResources,
    txpool::{kinet_txpool_statusByAddress, kinet_txpool_statusByHash},
};
use crate::{
    handlers::{
        debug::{
            KinetDebugTraceBlockByHashParams, KinetDebugTraceBlockByNumberParams,
            KinetDebugTraceTransactionParams,
        },
        eth::call::kinet_createAccessList,
    },
    middleware::TimingRequestId,
    types::{
        eth_json::serialize_result,
        json_serialized_len::JsonSerializedLen,
        jsonrpc::{
            serialize_with_size_limit, ErrorCode, JsonRpcError, JsonRpcResult, JsonRpcResultExt,
            Request, RequestId, RequestParams, RequestWrapper, Response, ResponseWrapper,
        },
    },
};

pub mod debug;
mod debug_replay;
pub mod eth;
mod meta;
pub mod resources;
mod txpool;

use kinet_chain_config::{
    ETHEREUM_MAINNET_CHAIN_ID, HIVE_CHAIN_ID, KINET_DEVNET_CHAIN_ID, KINET_MAINNET_CHAIN_ID,
    KINET_TESTNET_CHAIN_ID,
};
use kinet_ethcall::ChainId;

pub(crate) fn parse_ethcall_chain_id(chain_id: u64) -> JsonRpcResult<ChainId> {
    match chain_id {
        ETHEREUM_MAINNET_CHAIN_ID => Ok(ChainId::EthereumMainnet),
        KINET_MAINNET_CHAIN_ID => Ok(ChainId::KinetMainnet),
        KINET_TESTNET_CHAIN_ID => Ok(ChainId::KinetTestnet),
        KINET_DEVNET_CHAIN_ID => Ok(ChainId::KinetDevnet),
        HIVE_CHAIN_ID => Ok(ChainId::HiveNet),
        other => Err(JsonRpcError::eth_call_error(
            "unsupported chain id".to_string(),
            Some(other.to_string()),
        )),
    }
}

pub async fn rpc_handler(
    root_span: RootSpan,
    body: bytes::Bytes,
    app_state: web::Data<KinetRpcResources>,
    request_id: TimingRequestId,
) -> HttpResponse {
    let request = match RequestWrapper::from_body_bytes(&body) {
        Ok(req) => req,
        Err(e) => {
            debug!("parse error: {e} {body:?}");
            return HttpResponse::Ok().json(Response::from_error(JsonRpcError::new(
                ErrorCode::ParseError,
            )));
        }
    };

    let response = match request {
        RequestWrapper::Single(json_request) => {
            let Ok(request) = Request::from_raw_value(json_request) else {
                return HttpResponse::Ok().json(Response::from_error(JsonRpcError::new(
                    ErrorCode::InvalidRequest,
                )));
            };
            root_span.record("json_method", &request.method);
            let result = rpc_select(&app_state, &request.method, request.params, request_id).await;
            let response = Response::from_result(request.id.clone(), result);

            if let Some(comparator) = &app_state.rpc_comparator {
                let block_number = if let Some(data_provider) = &app_state.data_provider {
                    data_provider
                        .triedb_env
                        .get_latest_proposed_block_key()
                        .seq_num()
                        .0
                } else {
                    0
                };

                let comparator = comparator.clone();
                let json_request = serde_json::to_value(request).unwrap_or_default();
                let response_value = serde_json::to_value(&response).unwrap_or_default();

                tokio::spawn(async move {
                    comparator
                        .submit_comparison(block_number, json_request, response_value)
                        .await;
                });
            }

            ResponseWrapper::Single(response)
        }
        RequestWrapper::Batch(json_batch_request) => {
            root_span.record("json_method", "batch");

            let json_batch_request_len = json_batch_request.len();

            if json_batch_request_len == 0 {
                return HttpResponse::Ok().json(Response::from_error(JsonRpcError::new(
                    ErrorCode::InvalidRequest,
                )));
            }

            if json_batch_request_len > app_state.batch_request_limit as usize {
                return HttpResponse::Ok().json(Response::from_error(JsonRpcError::with_message(
                    ErrorCode::ServerError,
                    format!(
                        "number of requests in batch request exceeds limit of {}",
                        app_state.batch_request_limit
                    ),
                )));
            }

            let mut batch_response_json_serialized_len = 0usize;

            let batch_response = futures::stream::iter(json_batch_request)
                .map(|json_request| {
                    let app_state = app_state.clone();

                    async move {
                        let Ok(request) = Request::from_raw_value(json_request) else {
                            return Response::from_result(
                                RequestId::Null,
                                Err(JsonRpcError::new(ErrorCode::InvalidRequest)),
                            );
                        };

                        let (state, id, method, params) =
                            (app_state, request.id, request.method, request.params);

                        let result = rpc_select(&state, &method, params, request_id).await;

                        Response::from_result(id, result)
                    }
                })
                .buffered(app_state.batch_concurrent_limit as usize)
                .take_while(|response| {
                    batch_response_json_serialized_len += response.json_serialized_len();

                    let take =
                        batch_response_json_serialized_len <= app_state.max_response_size as usize;

                    async move { take }
                })
                .collect::<Vec<_>>()
                .await;

            if batch_response.len() < json_batch_request_len {
                return HttpResponse::Ok().json(Response::from_error(
                    JsonRpcError::max_response_size_exceeded(),
                ));
            }

            ResponseWrapper::Batch(batch_response)
        }
    };

    let response_raw_value =
        match serialize_with_size_limit(&response, app_state.max_response_size as usize) {
            Ok(raw) => raw,
            Err(e) => {
                debug!("response error: {}", e.message);
                return HttpResponse::Ok().json(Response::from_error(e));
            }
        };

    // log the request and response based on the response content
    match &response {
        ResponseWrapper::Single(resp) => match resp.error {
            Some(_) => debug!(?body, ?response, "rpc_request/response error"),
            None => debug!(
                ?body,
                ?response,
                ?request_id,
                "rpc_request/response successful"
            ),
        },
        _ => debug!(?body, ?response, ?request_id, "rpc_batch_request/response"),
    }

    HttpResponse::Ok().json(response_raw_value)
}

#[allow(non_snake_case)]
async fn admin_ethCallStatistics(
    _: TimingRequestId,
    app_state: &KinetRpcResources,
    _params: RequestParams<'_>,
) -> Result<Box<RawValue>, JsonRpcError> {
    let eth_call_handler = app_state.eth_call_handler.as_ref().method_not_found()?;
    let tracker = eth_call_handler.stats_tracker().method_not_found()?;
    kinet_admin_ethCallStatistics(
        eth_call_handler.config(),
        eth_call_handler.available_permits(),
        tracker,
    )
    .await
    .map(serialize_result)?
}

#[allow(non_snake_case)]
async fn debug_getRawBlock(
    _: TimingRequestId,
    app_state: &KinetRpcResources,
    params: RequestParams<'_>,
) -> Result<Box<RawValue>, JsonRpcError> {
    let data_provider = app_state.data_provider.as_ref().method_not_found()?;
    let params = serde_json::from_str(params.get()).invalid_params()?;
    kinet_debug_getRawBlock(data_provider, app_state.max_response_size as usize, params)
        .await
        .map(serialize_result)?
}

#[allow(non_snake_case)]
async fn debug_getRawHeader(
    _: TimingRequestId,
    app_state: &KinetRpcResources,
    params: RequestParams<'_>,
) -> Result<Box<RawValue>, JsonRpcError> {
    let data_provider = app_state.data_provider.as_ref().method_not_found()?;
    let params = serde_json::from_str(params.get()).invalid_params()?;
    kinet_debug_getRawHeader(data_provider, params)
        .await
        .map(serialize_result)?
}

#[allow(non_snake_case)]
async fn debug_getRawReceipts(
    _: TimingRequestId,
    app_state: &KinetRpcResources,
    params: RequestParams<'_>,
) -> Result<Box<RawValue>, JsonRpcError> {
    let data_provider = app_state.data_provider.as_ref().method_not_found()?;
    let params = serde_json::from_str(params.get()).invalid_params()?;
    kinet_debug_getRawReceipts(data_provider, app_state.max_response_size as usize, params)
        .await
        .map(serialize_result)?
}

#[allow(non_snake_case)]
async fn debug_getRawTransaction(
    _: TimingRequestId,
    app_state: &KinetRpcResources,
    params: RequestParams<'_>,
) -> Result<Box<RawValue>, JsonRpcError> {
    let data_provider = app_state.data_provider.as_ref().method_not_found()?;
    let params = serde_json::from_str(params.get()).invalid_params()?;
    kinet_debug_getRawTransaction(data_provider, params)
        .await
        .map(serialize_result)?
}

#[allow(non_snake_case)]
async fn debug_traceBlockByHash(
    request_id: TimingRequestId,
    app_state: &KinetRpcResources,
    params: RequestParams<'_>,
) -> Result<Box<RawValue>, JsonRpcError> {
    let data_provider = app_state.data_provider.as_ref().method_not_found()?;
    let params: KinetDebugTraceBlockByHashParams =
        serde_json::from_str(params.get()).invalid_params()?;
    if params.requires_replay() {
        return collect_debug_trace_via_replay(request_id, data_provider, app_state, &params)
            .await
            .map(serialize_result)?;
    }
    kinet_debug_traceBlockByHash(data_provider, app_state.max_response_size as usize, params)
        .await
        .map(serialize_result)?
}

#[allow(non_snake_case)]
async fn debug_traceBlockByNumber(
    request_id: TimingRequestId,
    app_state: &KinetRpcResources,
    params: RequestParams<'_>,
) -> Result<Box<RawValue>, JsonRpcError> {
    let data_provider = app_state.data_provider.as_ref().method_not_found()?;
    let params: KinetDebugTraceBlockByNumberParams =
        serde_json::from_str(params.get()).invalid_params()?;
    if params.requires_replay() {
        return collect_debug_trace_via_replay(request_id, data_provider, app_state, &params)
            .await
            .map(serialize_result)?;
    }

    kinet_debug_traceBlockByNumber(data_provider, app_state.max_response_size as usize, params)
        .await
        .map(serialize_result)?
}

#[allow(non_snake_case)]
async fn debug_traceCall(
    request_id: TimingRequestId,
    app_state: &KinetRpcResources,
    params: RequestParams<'_>,
) -> Result<Box<RawValue>, JsonRpcError> {
    let data_provider = app_state.data_provider.as_ref().method_not_found()?;
    let eth_call_handler = app_state.eth_call_handler.as_ref().method_not_found()?;
    let params = serde_json::from_str(params.get()).invalid_params()?;
    let permit = eth_call_handler.acquire(request_id).await?;

    permit
        .execute(|eth_call_handler_config, executor| {
            kinet_debug_traceCall(
                data_provider,
                eth_call_handler_config,
                executor,
                app_state.chain_id,
                app_state.max_response_size as usize,
                params,
            )
        })
        .await
        .map(serialize_result)?
}

#[allow(non_snake_case)]
async fn debug_traceTransaction(
    request_id: TimingRequestId,
    app_state: &KinetRpcResources,
    params: RequestParams<'_>,
) -> Result<Box<RawValue>, JsonRpcError> {
    let data_provider = app_state.data_provider.as_ref().method_not_found()?;
    let params: KinetDebugTraceTransactionParams =
        serde_json::from_str(params.get()).invalid_params()?;
    if params.requires_replay() {
        return collect_debug_trace_via_replay(request_id, data_provider, app_state, &params)
            .await
            .map(serialize_result)?;
    }

    kinet_debug_traceTransaction(data_provider, app_state.max_response_size as usize, params)
        .await
        .map(serialize_result)?
}

#[allow(non_snake_case)]
async fn eth_call(
    request_id: TimingRequestId,
    app_state: &KinetRpcResources,
    params: RequestParams<'_>,
) -> Result<Box<RawValue>, JsonRpcError> {
    let data_provider = app_state.data_provider.as_ref().method_not_found()?;
    let eth_call_handler = app_state.eth_call_handler.as_ref().method_not_found()?;
    let params = serde_json::from_str(params.get()).invalid_params()?;
    let permit = eth_call_handler.acquire(request_id).await?;

    permit
        .execute(|eth_call_handler_config, executor| {
            kinet_eth_call(
                data_provider,
                eth_call_handler_config,
                executor,
                app_state.chain_id,
                params,
            )
        })
        .await
        .map(serialize_result)?
}

#[allow(non_snake_case)]
async fn eth_simulateV1(
    request_id: TimingRequestId,
    app_state: &KinetRpcResources,
    params: RequestParams<'_>,
) -> Result<Box<RawValue>, JsonRpcError> {
    if !app_state.enable_eth_simulate_v1 {
        return Err(JsonRpcError::new(ErrorCode::MethodNotFound));
    }
    let data_provider = app_state.data_provider.as_ref().method_not_found()?;
    let eth_call_handler = app_state.eth_call_handler.as_ref().method_not_found()?;
    let params = serde_json::from_str(params.get()).invalid_params()?;
    let permit = eth_call_handler.acquire(request_id).await?;

    permit
        .execute(|config, executor| {
            kinet_simulate_v1(
                data_provider,
                executor,
                app_state.chain_id,
                // TODO(dhil): We use the eth call gas limit for individual calls within the simulation. We should consider adding more granular gas limits in the future.
                config.provider_gas_limit_eth_call,
                config.provider_gas_limit_eth_simulate,
                config.provider_max_calls_eth_simulate,
                config.provider_max_blocks_eth_simulate,
                params,
            )
        })
        .await
        .map(serialize_result)?
}

#[allow(non_snake_case)]
async fn eth_sendRawTransaction(
    _: TimingRequestId,
    app_state: &KinetRpcResources,
    params: RequestParams<'_>,
) -> Result<Box<RawValue>, JsonRpcError> {
    let txpool_bridge_client = app_state.txpool_bridge_client.as_ref().method_not_found()?;
    let params = serde_json::from_str(params.get()).invalid_params()?;
    kinet_eth_sendRawTransaction(
        txpool_bridge_client,
        params,
        app_state.chain_id,
        app_state.allow_unprotected_txs,
    )
    .await
    .map(serialize_result)?
}

#[allow(non_snake_case)]
async fn eth_sendRawTransactionSync(
    _: TimingRequestId,
    app_state: &KinetRpcResources,
    params: RequestParams<'_>,
) -> Result<Box<RawValue>, JsonRpcError> {
    let txpool_bridge_client = app_state.txpool_bridge_client.as_ref().method_not_found()?;
    let event_server_client = app_state.event_server_client.as_ref().method_not_found()?;
    let params = serde_json::from_str(params.get()).invalid_params()?;

    kinet_eth_sendRawTransactionSync(
        txpool_bridge_client,
        event_server_client,
        params,
        app_state.chain_id,
        app_state.allow_unprotected_txs,
        app_state.eth_send_raw_transaction_sync_default_timeout_ms,
        app_state.eth_send_raw_transaction_sync_max_timeout_ms,
    )
    .await
    .map(serialize_result)?
}

#[allow(non_snake_case)]
async fn eth_fillTransaction(
    request_id: TimingRequestId,
    app_state: &KinetRpcResources,
    params: RequestParams<'_>,
) -> Result<Box<RawValue>, JsonRpcError> {
    let data_provider = app_state.data_provider.as_ref().method_not_found()?;
    let eth_call_handler = app_state.eth_call_handler.as_ref().method_not_found()?;
    let params = serde_json::from_str(params.get()).invalid_params()?;
    let permit = eth_call_handler.acquire(request_id).await?;

    permit
        .execute(|eth_call_handler_config, executor| {
            kinet_eth_fillTransaction(
                data_provider,
                eth_call_handler_config,
                executor,
                app_state.chain_id,
                params,
            )
        })
        .await
        .map(serialize_result)?
}

#[allow(non_snake_case)]
async fn eth_createAccessList(
    request_id: TimingRequestId,
    app_state: &KinetRpcResources,
    params: RequestParams<'_>,
) -> Result<Box<RawValue>, JsonRpcError> {
    let data_provider = app_state.data_provider.as_ref().method_not_found()?;
    let eth_call_handler = app_state.eth_call_handler.as_ref().method_not_found()?;
    let params = serde_json::from_str(params.get()).invalid_params()?;
    let permit = eth_call_handler.acquire(request_id).await?;

    permit
        .execute(|eth_call_handler_config, executor| {
            kinet_createAccessList(
                data_provider,
                eth_call_handler_config,
                executor,
                app_state.chain_id,
                params,
            )
        })
        .await
        .map(serialize_result)?
}

#[allow(non_snake_case)]
async fn eth_getLogs(
    _: TimingRequestId,
    app_state: &KinetRpcResources,
    params: RequestParams<'_>,
) -> Result<Box<RawValue>, JsonRpcError> {
    let data_provider = app_state.data_provider.as_ref().method_not_found()?;
    let params = serde_json::from_str(params.get()).invalid_params()?;
    kinet_eth_getLogs(
        data_provider,
        app_state.max_response_size,
        app_state.logs_max_block_range,
        params,
        app_state.use_eth_get_logs_index,
        app_state.dry_run_get_logs_index,
        app_state.max_finalized_block_cache_len,
    )
    .await
    .map(serialize_result)?
}

#[allow(non_snake_case)]
async fn eth_getTransactionByHash(
    _: TimingRequestId,
    app_state: &KinetRpcResources,
    params: RequestParams<'_>,
) -> Result<Box<RawValue>, JsonRpcError> {
    let data_provider = app_state.data_provider.as_ref().method_not_found()?;
    let params = serde_json::from_str(params.get()).invalid_params()?;
    kinet_eth_getTransactionByHash(data_provider, params)
        .await
        .map(serialize_result)?
}

#[allow(non_snake_case)]
async fn eth_getBlockByHash(
    _: TimingRequestId,
    app_state: &KinetRpcResources,
    params: RequestParams<'_>,
) -> Result<Box<RawValue>, JsonRpcError> {
    let data_provider = app_state.data_provider.as_ref().method_not_found()?;
    let params = serde_json::from_str(params.get()).invalid_params()?;
    kinet_eth_getBlockByHash(data_provider, params)
        .await
        .map(serialize_result)?
}

#[allow(non_snake_case)]
async fn eth_getBlockByNumber(
    _: TimingRequestId,
    app_state: &KinetRpcResources,
    params: RequestParams<'_>,
) -> Result<Box<RawValue>, JsonRpcError> {
    let data_provider = app_state.data_provider.as_ref().method_not_found()?;
    let params = serde_json::from_str(params.get()).invalid_params()?;
    kinet_eth_getBlockByNumber(data_provider, params)
        .await
        .map(serialize_result)?
}

#[allow(non_snake_case)]
async fn eth_getTransactionByBlockHashAndIndex(
    _: TimingRequestId,
    app_state: &KinetRpcResources,
    params: RequestParams<'_>,
) -> Result<Box<RawValue>, JsonRpcError> {
    let data_provider = app_state.data_provider.as_ref().method_not_found()?;
    let params = serde_json::from_str(params.get()).invalid_params()?;
    kinet_eth_getTransactionByBlockHashAndIndex(data_provider, params)
        .await
        .map(serialize_result)?
}

#[allow(non_snake_case)]
async fn eth_getTransactionByBlockNumberAndIndex(
    _: TimingRequestId,
    app_state: &KinetRpcResources,
    params: RequestParams<'_>,
) -> Result<Box<RawValue>, JsonRpcError> {
    let data_provider = app_state.data_provider.as_ref().method_not_found()?;
    let params = serde_json::from_str(params.get()).invalid_params()?;
    kinet_eth_getTransactionByBlockNumberAndIndex(data_provider, params)
        .await
        .map(serialize_result)?
}

#[allow(non_snake_case)]
async fn eth_getBlockTransactionCountByHash(
    _: TimingRequestId,
    app_state: &KinetRpcResources,
    params: RequestParams<'_>,
) -> Result<Box<RawValue>, JsonRpcError> {
    let data_provider = app_state.data_provider.as_ref().method_not_found()?;
    let params = serde_json::from_str(params.get()).invalid_params()?;
    kinet_eth_getBlockTransactionCountByHash(data_provider, params)
        .await
        .map(serialize_result)?
}

#[allow(non_snake_case)]
async fn eth_getBlockTransactionCountByNumber(
    _: TimingRequestId,
    app_state: &KinetRpcResources,
    params: RequestParams<'_>,
) -> Result<Box<RawValue>, JsonRpcError> {
    let data_provider = app_state.data_provider.as_ref().method_not_found()?;
    let params = serde_json::from_str(params.get()).invalid_params()?;
    kinet_eth_getBlockTransactionCountByNumber(data_provider, params)
        .await
        .map(serialize_result)?
}

#[allow(non_snake_case)]
async fn eth_getBalance(
    _: TimingRequestId,
    app_state: &KinetRpcResources,
    params: RequestParams<'_>,
) -> Result<Box<RawValue>, JsonRpcError> {
    let data_provider = app_state.data_provider.as_ref().method_not_found()?;
    let params = serde_json::from_str(params.get()).invalid_params()?;
    kinet_eth_getBalance(data_provider, params)
        .await
        .map(serialize_result)?
}

#[allow(non_snake_case)]
async fn eth_getCode(
    _: TimingRequestId,
    app_state: &KinetRpcResources,
    params: RequestParams<'_>,
) -> Result<Box<RawValue>, JsonRpcError> {
    let data_provider = app_state.data_provider.as_ref().method_not_found()?;
    let params = serde_json::from_str(params.get()).invalid_params()?;
    kinet_eth_getCode(data_provider, params)
        .await
        .map(serialize_result)?
}

#[allow(non_snake_case)]
async fn eth_getStorageAt(
    _: TimingRequestId,
    app_state: &KinetRpcResources,
    params: RequestParams<'_>,
) -> Result<Box<RawValue>, JsonRpcError> {
    let data_provider = app_state.data_provider.as_ref().method_not_found()?;
    let params = serde_json::from_str(params.get()).invalid_params()?;
    kinet_eth_getStorageAt(data_provider, params)
        .await
        .map(serialize_result)?
}

#[allow(non_snake_case)]
async fn eth_getTransactionCount(
    _: TimingRequestId,
    app_state: &KinetRpcResources,
    params: RequestParams<'_>,
) -> Result<Box<RawValue>, JsonRpcError> {
    let data_provider = app_state.data_provider.as_ref().method_not_found()?;
    let params = serde_json::from_str(params.get()).invalid_params()?;
    kinet_eth_getTransactionCount(data_provider, params)
        .await
        .map(serialize_result)?
}

#[allow(non_snake_case)]
async fn eth_blockNumber(
    _: TimingRequestId,
    app_state: &KinetRpcResources,
    _params: RequestParams<'_>,
) -> Result<Box<RawValue>, JsonRpcError> {
    let data_provider = app_state.data_provider.as_ref().method_not_found()?;
    kinet_eth_blockNumber(data_provider)
        .await
        .map(serialize_result)?
}

#[allow(non_snake_case)]
async fn eth_chainId(
    _: TimingRequestId,
    app_state: &KinetRpcResources,
    _params: RequestParams<'_>,
) -> Result<Box<RawValue>, JsonRpcError> {
    kinet_eth_chainId(app_state.chain_id)
        .await
        .map(serialize_result)?
}

#[allow(non_snake_case)]
async fn eth_syncing(
    _: TimingRequestId,
    _app_state: &KinetRpcResources,
    _params: RequestParams<'_>,
) -> Result<Box<RawValue>, JsonRpcError> {
    kinet_eth_syncing().await.map(serialize_result)?
}

#[allow(non_snake_case)]
async fn eth_estimateGas(
    request_id: TimingRequestId,
    app_state: &KinetRpcResources,
    params: RequestParams<'_>,
) -> Result<Box<RawValue>, JsonRpcError> {
    let data_provider = app_state.data_provider.as_ref().method_not_found()?;
    let eth_call_handler = app_state.eth_call_handler.as_ref().method_not_found()?;
    let params = serde_json::from_str(params.get()).invalid_params()?;
    let permit = eth_call_handler.acquire(request_id).await?;

    permit
        .execute(|eth_call_handler_config, executor| {
            kinet_eth_estimateGas(
                data_provider,
                eth_call_handler_config,
                executor,
                app_state.chain_id,
                params,
            )
        })
        .await
        .map(serialize_result)?
}

#[allow(non_snake_case)]
async fn eth_gasPrice(
    _: TimingRequestId,
    app_state: &KinetRpcResources,
    _params: RequestParams<'_>,
) -> Result<Box<RawValue>, JsonRpcError> {
    let data_provider = app_state.data_provider.as_ref().method_not_found()?;
    kinet_eth_gasPrice(data_provider)
        .await
        .map(serialize_result)?
}

#[allow(non_snake_case)]
async fn eth_maxPriorityFeePerGas(
    _: TimingRequestId,
    _app_state: &KinetRpcResources,
    _params: RequestParams<'_>,
) -> Result<Box<RawValue>, JsonRpcError> {
    kinet_eth_maxPriorityFeePerGas()
        .await
        .map(serialize_result)?
}

#[allow(non_snake_case)]
async fn eth_feeHistory(
    _: TimingRequestId,
    app_state: &KinetRpcResources,
    params: RequestParams<'_>,
) -> Result<Box<RawValue>, JsonRpcError> {
    let data_provider = app_state.data_provider.as_ref().method_not_found()?;
    let params = serde_json::from_str(params.get()).invalid_params()?;
    let _permit = app_state.feehistory_limiter.try_acquire().map_err(|_| {
        JsonRpcError::internal_error("exceed feehistory max concurrent requests".into())
    })?;
    kinet_eth_feeHistory(data_provider, params)
        .await
        .map(serialize_result)?
}

#[allow(non_snake_case)]
async fn eth_getTransactionReceipt(
    _: TimingRequestId,
    app_state: &KinetRpcResources,
    params: RequestParams<'_>,
) -> Result<Box<RawValue>, JsonRpcError> {
    let data_provider = app_state.data_provider.as_ref().method_not_found()?;
    let params = serde_json::from_str(params.get()).invalid_params()?;
    kinet_eth_getTransactionReceipt(data_provider, params)
        .await
        .map(serialize_result)?
}

#[allow(non_snake_case)]
async fn eth_getBlockReceipts(
    _: TimingRequestId,
    app_state: &KinetRpcResources,
    params: RequestParams<'_>,
) -> Result<Box<RawValue>, JsonRpcError> {
    let data_provider = app_state.data_provider.as_ref().method_not_found()?;
    let params = serde_json::from_str(params.get()).invalid_params()?;
    kinet_eth_getBlockReceipts(data_provider, params)
        .await
        .map(serialize_result)?
}

#[allow(non_snake_case)]
async fn net_version(
    _: TimingRequestId,
    app_state: &KinetRpcResources,
    _params: RequestParams<'_>,
) -> Result<Box<RawValue>, JsonRpcError> {
    kinet_net_version(app_state.chain_id).map(serialize_result)?
}

#[allow(non_snake_case)]
async fn txpool_statusByHash(
    _: TimingRequestId,
    app_state: &KinetRpcResources,
    params: RequestParams<'_>,
) -> Result<Box<RawValue>, JsonRpcError> {
    let txpool_bridge_client = app_state.txpool_bridge_client.as_ref().method_not_found()?;
    let params = serde_json::from_str(params.get()).invalid_params()?;
    kinet_txpool_statusByHash(txpool_bridge_client, params)
        .await
        .map(serialize_result)?
}

#[allow(non_snake_case)]
async fn txpool_statusByAddress(
    _: TimingRequestId,
    app_state: &KinetRpcResources,
    params: RequestParams<'_>,
) -> Result<Box<RawValue>, JsonRpcError> {
    let txpool_bridge_client = app_state.txpool_bridge_client.as_ref().method_not_found()?;
    let params = serde_json::from_str(params.get()).invalid_params()?;
    kinet_txpool_statusByAddress(txpool_bridge_client, params)
        .await
        .map(serialize_result)?
}

#[allow(non_snake_case)]
async fn web3_clientVersion(
    _: TimingRequestId,
    _app_state: &KinetRpcResources,
    _params: RequestParams<'_>,
) -> Result<Box<RawValue>, JsonRpcError> {
    kinet_web3_client_version().map(serialize_result)?
}

macro_rules! enabled_methods {
    ($($(#[$attr:meta])* $method:ident),* $(,)?) => {

        #[derive(Debug, Clone, Copy)]
        #[allow(non_camel_case_types)]
        enum EnabledMethod {
            $(
                $(#[$attr])*
                $method,
            )*
        }

        impl TryFrom<&str> for EnabledMethod {
            type Error = JsonRpcError;

            fn try_from(method: &str) -> Result<Self, Self::Error> {
                match method {
                    $(
                        stringify!($method) => Ok(EnabledMethod::$method),
                    )*
                    _ => Err(JsonRpcError::new(ErrorCode::MethodNotFound)),
                }
            }
        }

        impl EnabledMethod {
            fn span(&self) -> Span {
                match self {
                    $(
                        EnabledMethod::$method => trace_span!(stringify!($method)),
                    )*
                }
            }

            async fn call(
                &self,
                request_id: TimingRequestId,
                app_state: &KinetRpcResources,
                params: RequestParams<'_>,
            ) -> Result<Box<RawValue>, JsonRpcError> {
                match self {
                    $(
                        EnabledMethod::$method => $method(request_id, app_state, params).await,
                    )*
                }
            }
        }
    };
}

enabled_methods!(
    admin_ethCallStatistics,
    debug_getRawBlock,
    debug_getRawHeader,
    debug_getRawReceipts,
    debug_getRawTransaction,
    debug_traceBlockByHash,
    debug_traceBlockByNumber,
    debug_traceCall,
    debug_traceTransaction,
    eth_call,
    eth_simulateV1,
    eth_sendRawTransaction,
    eth_sendRawTransactionSync,
    eth_createAccessList,
    eth_getLogs,
    eth_getTransactionByHash,
    eth_getBlockByHash,
    eth_getBlockByNumber,
    eth_getTransactionByBlockHashAndIndex,
    eth_getTransactionByBlockNumberAndIndex,
    eth_getBlockTransactionCountByHash,
    eth_getBlockTransactionCountByNumber,
    eth_getBalance,
    eth_getCode,
    eth_getStorageAt,
    eth_getTransactionCount,
    eth_blockNumber,
    eth_chainId,
    eth_syncing,
    eth_estimateGas,
    eth_gasPrice,
    eth_maxPriorityFeePerGas,
    eth_feeHistory,
    eth_getTransactionReceipt,
    eth_getBlockReceipts,
    net_version,
    txpool_statusByHash,
    txpool_statusByAddress,
    web3_clientVersion,
    eth_fillTransaction
);

#[tracing::instrument(level = "debug", skip_all)]
pub async fn rpc_select(
    app_state: &KinetRpcResources,
    method: &str,
    params: RequestParams<'_>,
    request_id: TimingRequestId,
) -> Result<Box<RawValue>, JsonRpcError> {
    let method: EnabledMethod = method.try_into()?;
    let mut span = method.span();
    if let Some(metrics) = &app_state.metrics {
        span = span.with_main_timings(metrics.execution_histogram.clone());
    }
    method
        .call(request_id, app_state, params)
        .instrument(span)
        .await
}

#[cfg(test)]
mod tests {
    use kinet_chain_config::{
        ETHEREUM_MAINNET_CHAIN_ID, HIVE_CHAIN_ID, KINET_DEVNET_CHAIN_ID, KINET_MAINNET_CHAIN_ID,
        KINET_TESTNET_CHAIN_ID,
    };
    use kinet_ethcall::ChainId;

    use super::parse_ethcall_chain_id;

    #[test]
    fn parse_ethcall_chain_id_maps_supported_chains() {
        assert_eq!(
            parse_ethcall_chain_id(ETHEREUM_MAINNET_CHAIN_ID).unwrap(),
            ChainId::EthereumMainnet
        );
        assert_eq!(
            parse_ethcall_chain_id(KINET_MAINNET_CHAIN_ID).unwrap(),
            ChainId::KinetMainnet
        );
        assert_eq!(
            parse_ethcall_chain_id(KINET_TESTNET_CHAIN_ID).unwrap(),
            ChainId::KinetTestnet
        );
        assert_eq!(
            parse_ethcall_chain_id(KINET_DEVNET_CHAIN_ID).unwrap(),
            ChainId::KinetDevnet
        );
        assert_eq!(
            parse_ethcall_chain_id(HIVE_CHAIN_ID).unwrap(),
            ChainId::HiveNet
        );
    }

    #[test]
    fn parse_ethcall_chain_id_rejects_unknown_chain() {
        assert!(parse_ethcall_chain_id(42).is_err());
    }
}
