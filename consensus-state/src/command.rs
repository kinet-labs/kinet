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

use std::time::Duration;

use kinet_chain_config::{revision::ChainRevision, ChainConfig};
use kinet_consensus::{
    messages::{
        consensus_message::{ConsensusMessage, ProtocolMessage},
        message::{AdvanceRoundMessage, TimeoutMessage},
    },
    pacemaker::PacemakerCommand,
    validation::signing::{Validated, Verified},
    vote_state::VoteStateCommand,
};
use kinet_consensus_types::{
    block::{BlockPolicy, BlockRange, ConsensusBlockHeader, OptimisticPolicyCommit},
    no_endorsement::FreshProposalCertificate,
    payload::RoundSignature,
    quorum_certificate::{QuorumCertificate, TimestampAdjustment},
    timeout::TimeoutCertificate,
};
use kinet_crypto::certificate_signature::{
    CertificateSignaturePubKey, CertificateSignatureRecoverable,
};
use kinet_execution_state_read::ExecutionStateRead;
use kinet_types::{
    Epoch, ExecutionProtocol, FullnodeBroadcastMode, NodeId, Round, RouterTarget, SeqNum,
};
use kinet_validator::signature_collection::{SignatureCollection, SignatureCollectionKeyPairType};

/// Command type that the consensus state-machine outputs
/// This is converted to a kinet-executor-glue::Command at the top-level kinet-state
#[derive(Debug)]
pub enum ConsensusCommand<ST, SCT, EPT, BPT, ESRT, CCT, CRT>
where
    ST: CertificateSignatureRecoverable,
    SCT: SignatureCollection<NodeIdPubKey = CertificateSignaturePubKey<ST>>,
    EPT: ExecutionProtocol,
    BPT: BlockPolicy<ST, SCT, EPT, ESRT, CCT, CRT>,
    ESRT: ExecutionStateRead<ST, SCT>,
    CCT: ChainConfig<CRT>,
    CRT: ChainRevision,
{
    EnterRound(Epoch, Round),
    /// Attempt to send a message to RouterTarget
    /// Delivery is NOT guaranteed, retry must be handled at the state-machine level
    Publish {
        target: RouterTarget<SCT::NodeIdPubKey>,
        message: Verified<ST, Validated<ConsensusMessage<ST, SCT, EPT>>>,
    },
    PublishToFullNodes {
        epoch: Epoch,
        round: Round,
        broadcast_mode: FullnodeBroadcastMode,
        message: Verified<ST, Validated<ConsensusMessage<ST, SCT, EPT>>>,
    },
    /// Schedule a timeout event for `round` to be emitted in `duration`
    Schedule {
        round: Round,
        duration: Duration,
    },
    /// Cancel scheduled (if exists) timeout event
    ScheduleReset,
    /// Creates a proposal
    CreateProposal {
        node_id: NodeId<CertificateSignaturePubKey<ST>>,
        epoch: Epoch,
        round: Round,
        seq_num: SeqNum,
        high_qc: QuorumCertificate<SCT>,
        round_signature: RoundSignature<SCT::SignatureType>,
        last_round_tc: Option<TimeoutCertificate<ST, SCT, EPT>>,
        fresh_proposal_certificate: Option<FreshProposalCertificate<SCT>>,

        tx_limit: usize,
        proposal_gas_limit: u64,
        proposal_byte_limit: u64,
        beneficiary: [u8; 20],
        timestamp_ns: u128,

        extending_blocks: Vec<BPT::ValidatedBlock>,
        delayed_execution_results: Vec<EPT::FinalizedHeader>,
    },
    /// Commit blocks to ledger
    CommitBlocks(OptimisticPolicyCommit<ST, SCT, EPT, BPT, ESRT, CCT, CRT>),
    /// Requests BlockSync
    /// Serviced by block_sync in KinetState
    RequestSync(BlockRange),
    /// Cancels BlockSync request
    CancelSync(BlockRange),
    /// Too far behind, request StateSync with:
    /// 1. New blocktree root
    /// 2. New high_qc
    ///
    /// TODO we can include blocktree cache if we want
    RequestStateSync {
        root: ConsensusBlockHeader<ST, SCT, EPT>,
        high_qc: QuorumCertificate<SCT>,
    },
    // TODO-2 add command for updating validator_set/round
    // - to handle this command, we need to call message_state.set_round()
    TimestampUpdate(TimestampAdjustment),
    ScheduleVote {
        duration: Duration,
        round: Round,
    },
}

impl<ST, SCT, EPT, BPT, ESRT, CCT, CRT> ConsensusCommand<ST, SCT, EPT, BPT, ESRT, CCT, CRT>
where
    ST: CertificateSignatureRecoverable,
    SCT: SignatureCollection<NodeIdPubKey = CertificateSignaturePubKey<ST>>,
    EPT: ExecutionProtocol,
    BPT: BlockPolicy<ST, SCT, EPT, ESRT, CCT, CRT>,
    ESRT: ExecutionStateRead<ST, SCT>,
    CCT: ChainConfig<CRT>,
    CRT: ChainRevision,
{
    pub fn from_pacemaker_command(
        keypair: &ST::KeyPairType,
        cert_keypair: &SignatureCollectionKeyPairType<SCT>,
        version: u32,
        cmd: PacemakerCommand<ST, SCT, EPT>,
    ) -> Vec<Self> {
        match cmd {
            PacemakerCommand::EnterRound(epoch, round, high_certificate) => {
                let mut cmds = Vec::new();
                cmds.push(ConsensusCommand::EnterRound(epoch, round));
                cmds.push(ConsensusCommand::PublishToFullNodes {
                    epoch,
                    round: high_certificate.round(),
                    broadcast_mode: FullnodeBroadcastMode::Broadcast,
                    message: ConsensusMessage {
                        version,
                        message: ProtocolMessage::AdvanceRound(AdvanceRoundMessage {
                            last_round_certificate: high_certificate,
                        }),
                    }
                    .sign(keypair),
                });
                cmds
            }
            PacemakerCommand::PrepareTimeout(
                timeout,
                high_extend,
                safe_to_vote,
                last_round_certificate,
            ) => {
                vec![ConsensusCommand::Publish {
                    // TODO should this be sent to epoch of next round?
                    target: RouterTarget::Broadcast(timeout.epoch),
                    message: ConsensusMessage {
                        version,
                        message: ProtocolMessage::Timeout(TimeoutMessage::new(
                            cert_keypair,
                            timeout,
                            high_extend,
                            safe_to_vote,
                            last_round_certificate,
                        )),
                    }
                    .sign(keypair),
                }]
            }
            PacemakerCommand::Schedule { round, duration } => {
                vec![ConsensusCommand::Schedule { round, duration }]
            }
            PacemakerCommand::ScheduleReset => vec![ConsensusCommand::ScheduleReset],
        }
    }
}

impl<ST, SCT, EPT, BPT, ESRT, CCT, CRT> From<VoteStateCommand>
    for ConsensusCommand<ST, SCT, EPT, BPT, ESRT, CCT, CRT>
where
    ST: CertificateSignatureRecoverable,
    SCT: SignatureCollection<NodeIdPubKey = CertificateSignaturePubKey<ST>>,
    EPT: ExecutionProtocol,
    BPT: BlockPolicy<ST, SCT, EPT, ESRT, CCT, CRT>,
    ESRT: ExecutionStateRead<ST, SCT>,
    CCT: ChainConfig<CRT>,
    CRT: ChainRevision,
{
    fn from(value: VoteStateCommand) -> Self {
        //TODO-3 VoteStateCommand used for evidence collection
        match value {}
    }
}
