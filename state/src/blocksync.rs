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

use std::{collections::BTreeMap, marker::PhantomData, time::Duration};

use kinet_blocksync::blocksync::{
    BlockCache, BlockSync, BlockSyncCommand, BlockSyncSelfRequester, BlockSyncWrapper,
};
use kinet_chain_config::{revision::ChainRevision, ChainConfig};
use kinet_consensus_types::{
    block::BlockPolicy, block_validator::BlockValidator, metrics::Metrics,
};
use kinet_crypto::certificate_signature::{
    CertificateSignaturePubKey, CertificateSignatureRecoverable,
};
use kinet_execution_state_read::ExecutionStateRead;
use kinet_executor_glue::{
    BlockSyncEvent, Command, ConsensusEvent, LedgerCommand, LoopbackCommand, KinetEvent,
    RouterCommand, StateSyncEvent, TimeoutVariant, TimerCommand,
};
use kinet_types::{ExecutionProtocol, NodeId, Round, RouterTarget};
use kinet_validator::{
    epoch_manager::EpochManager, signature_collection::SignatureCollection,
    validator_set::ValidatorSetTypeFactory, validators_epoch_mapping::ValidatorsEpochMapping,
};

use crate::{ConsensusMode, KinetState, VerifiedKinetMessage};

pub(super) struct BlockSyncChildState<'a, ST, SCT, EPT, BPT, ESRT, VTF, LT, BVT, CCT, CRT>
where
    ST: CertificateSignatureRecoverable,
    SCT: SignatureCollection<NodeIdPubKey = CertificateSignaturePubKey<ST>>,
    EPT: ExecutionProtocol,
    BPT: BlockPolicy<ST, SCT, EPT, ESRT, CCT, CRT>,
    ESRT: ExecutionStateRead<ST, SCT>,
    VTF: ValidatorSetTypeFactory<NodeIdPubKey = CertificateSignaturePubKey<ST>>,
    BVT: BlockValidator<ST, SCT, EPT, BPT, ESRT, CCT, CRT>,
    CCT: ChainConfig<CRT>,
    CRT: ChainRevision,
{
    block_sync: &'a mut BlockSync<ST, SCT, EPT>,

    /// BlockSync queries consensus first when receiving BlockSyncRequest
    consensus: &'a ConsensusMode<ST, SCT, EPT, BPT, ESRT, CCT, CRT>,
    epoch_manager: &'a EpochManager,
    val_epoch_map: &'a ValidatorsEpochMapping<VTF, SCT>,
    secondary_raptorcast_peers: &'a BTreeMap<NodeId<CertificateSignaturePubKey<ST>>, Round>,
    delta: &'a Duration,
    nodeid: &'a NodeId<CertificateSignaturePubKey<ST>>,

    metrics: &'a mut Metrics,

    _phantom: PhantomData<(ST, SCT, EPT, BPT, ESRT, VTF, LT, BVT)>,
}

impl<'a, ST, SCT, EPT, BPT, ESRT, VTF, LT, BVT, CCT, CRT>
    BlockSyncChildState<'a, ST, SCT, EPT, BPT, ESRT, VTF, LT, BVT, CCT, CRT>
where
    ST: CertificateSignatureRecoverable,
    SCT: SignatureCollection<NodeIdPubKey = CertificateSignaturePubKey<ST>>,
    EPT: ExecutionProtocol,
    BPT: BlockPolicy<ST, SCT, EPT, ESRT, CCT, CRT>,
    ESRT: ExecutionStateRead<ST, SCT>,
    VTF: ValidatorSetTypeFactory<NodeIdPubKey = CertificateSignaturePubKey<ST>>,
    BVT: BlockValidator<ST, SCT, EPT, BPT, ESRT, CCT, CRT>,
    CCT: ChainConfig<CRT>,
    CRT: ChainRevision,
{
    pub(super) fn new(
        kinet_state: &'a mut KinetState<ST, SCT, EPT, BPT, ESRT, VTF, LT, BVT, CCT, CRT>,
    ) -> Self {
        Self {
            block_sync: &mut kinet_state.block_sync,
            consensus: &kinet_state.consensus,
            epoch_manager: &kinet_state.epoch_manager,
            val_epoch_map: &kinet_state.val_epoch_map,
            secondary_raptorcast_peers: &kinet_state.secondary_raptorcast_peers,
            delta: &kinet_state.consensus_config.delta,
            nodeid: &kinet_state.nodeid,
            metrics: &mut kinet_state.metrics,
            _phantom: PhantomData,
        }
    }

    pub(super) fn update(
        &mut self,
        event: BlockSyncEvent<ST, SCT, EPT>,
    ) -> Vec<WrappedBlockSyncCommand<ST, SCT, EPT>> {
        let block_cache = match self.consensus {
            ConsensusMode::Sync { block_buffer, .. } => {
                BlockCache::BlockBuffer(block_buffer.get_payload_cache())
            }
            ConsensusMode::Live(consensus) => BlockCache::BlockTree(consensus.blocktree()),
        };

        let mut block_sync_wrapper = BlockSyncWrapper {
            block_sync: self.block_sync,
            block_cache,
            metrics: self.metrics,
            nodeid: self.nodeid,
            current_epoch: self.consensus.current_epoch(),
            epoch_manager: self.epoch_manager,
            val_epoch_map: self.val_epoch_map,
            secondary_raptorcast_peers: self.secondary_raptorcast_peers,
        };

        let cmds = match event {
            BlockSyncEvent::Request { sender, request } => {
                block_sync_wrapper.handle_peer_request(sender, request)
            }
            BlockSyncEvent::SelfRequest {
                requester,
                block_range,
            } => block_sync_wrapper.handle_self_request(requester, block_range),
            BlockSyncEvent::SelfCancelRequest {
                requester,
                block_range,
            } => {
                block_sync_wrapper.handle_self_cancel_request(requester, block_range);
                Vec::new()
            }
            BlockSyncEvent::SelfResponse { response } => {
                block_sync_wrapper.handle_ledger_response(response)
            }
            BlockSyncEvent::Response { sender, response } => {
                block_sync_wrapper.handle_peer_response(sender, response)
            }
            BlockSyncEvent::Timeout(request) => block_sync_wrapper.handle_timeout(request),
        };
        cmds.into_iter()
            .map(|command| WrappedBlockSyncCommand {
                // TODO: timeout should be more aggressive for headers request
                request_timeout: *self.delta * 7,
                command,
            })
            .collect()
    }
}

pub(crate) struct WrappedBlockSyncCommand<ST, SCT, EPT>
where
    ST: CertificateSignatureRecoverable,
    SCT: SignatureCollection<NodeIdPubKey = CertificateSignaturePubKey<ST>>,
    EPT: ExecutionProtocol,
{
    request_timeout: Duration,
    command: BlockSyncCommand<ST, SCT, EPT>,
}

impl<ST, SCT, EPT, BPT, ESRT, CCT, CRT> From<WrappedBlockSyncCommand<ST, SCT, EPT>>
    for Vec<
        Command<
            KinetEvent<ST, SCT, EPT>,
            VerifiedKinetMessage<ST, SCT, EPT>,
            ST,
            SCT,
            EPT,
            BPT,
            ESRT,
            CCT,
            CRT,
        >,
    >
where
    ST: CertificateSignatureRecoverable,
    SCT: SignatureCollection<NodeIdPubKey = CertificateSignaturePubKey<ST>>,
    EPT: ExecutionProtocol,
    BPT: BlockPolicy<ST, SCT, EPT, ESRT, CCT, CRT>,
    ESRT: ExecutionStateRead<ST, SCT>,
    CCT: ChainConfig<CRT>,
    CRT: ChainRevision,
{
    fn from(wrapped: WrappedBlockSyncCommand<ST, SCT, EPT>) -> Self {
        match wrapped.command {
            BlockSyncCommand::SendRequest { to, request } => {
                vec![Command::RouterCommand(RouterCommand::Publish {
                    target: RouterTarget::TcpPointToPoint {
                        to,
                        completion: None,
                    },
                    message: VerifiedKinetMessage::BlockSyncRequest(request),
                })]
            }
            BlockSyncCommand::ScheduleTimeout(request) => {
                vec![Command::TimerCommand(TimerCommand::Schedule {
                    duration: wrapped.request_timeout,
                    variant: TimeoutVariant::BlockSync(request),
                    on_timeout: KinetEvent::BlockSyncEvent(BlockSyncEvent::Timeout(request)),
                })]
            }
            BlockSyncCommand::ResetTimeout(block_id) => {
                vec![Command::TimerCommand(TimerCommand::ScheduleReset(
                    TimeoutVariant::BlockSync(block_id),
                ))]
            }
            BlockSyncCommand::SendResponse { to, response } => {
                vec![Command::RouterCommand(RouterCommand::Publish {
                    target: RouterTarget::TcpPointToPoint {
                        to,
                        completion: None,
                    },
                    message: VerifiedKinetMessage::BlockSyncResponse(response),
                })]
            }
            BlockSyncCommand::FetchHeaders(block_range) => {
                vec![Command::LedgerCommand(LedgerCommand::LedgerFetchHeaders(
                    block_range,
                ))]
            }
            BlockSyncCommand::FetchPayload(payload_id) => {
                vec![Command::LedgerCommand(LedgerCommand::LedgerFetchPayload(
                    payload_id,
                ))]
            }
            BlockSyncCommand::Emit(requester, (block_range, full_blocks)) => {
                vec![Command::LoopbackCommand(LoopbackCommand::Forward(
                    match requester {
                        BlockSyncSelfRequester::StateSync => {
                            KinetEvent::StateSyncEvent(StateSyncEvent::BlockSync {
                                block_range,
                                full_blocks,
                            })
                        }
                        BlockSyncSelfRequester::Consensus => {
                            KinetEvent::ConsensusEvent(ConsensusEvent::BlockSync {
                                block_range,
                                full_blocks,
                            })
                        }
                    },
                ))]
            }
        }
    }
}
