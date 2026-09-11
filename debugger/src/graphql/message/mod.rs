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

use std::ops::Deref;

use async_graphql::{Object, Union};
use kinet_blocksync::messages::message::{
    BlockSyncBodyResponse, BlockSyncHeadersResponse, BlockSyncRequestMessage,
    BlockSyncResponseMessage,
};
use kinet_consensus::{
    messages::{
        consensus_message::{ConsensusMessage, ProtocolMessage},
        message::{
            AdvanceRoundMessage, NoEndorsementMessage, ProposalMessage, RoundRecoveryMessage,
            TimeoutMessage, VoteMessage,
        },
    },
    validation::signing::{Validated, Verified},
};
use kinet_consensus_types::{block::BlockRange, timeout::HighExtendVote};
use kinet_state::VerifiedKinetMessage;
use kinet_types::Round;

use crate::graphql::{
    block_body_id_string, block_id_string, ExecutionProtocolType, GraphQLBlockRef, GraphQLRound,
    GraphQLSeqNum, SignatureCollectionType, SignatureType,
};

type KinetMessageType =
    VerifiedKinetMessage<SignatureType, SignatureCollectionType, ExecutionProtocolType>;
type ConsensusMessageType =
    ConsensusMessage<SignatureType, SignatureCollectionType, ExecutionProtocolType>;
type VerifiedConsensusMessageType = Verified<SignatureType, Validated<ConsensusMessageType>>;

impl<'s> From<&'s KinetMessageType> for GraphQLKinetMessage<'s> {
    fn from(message: &'s KinetMessageType) -> Self {
        match message {
            KinetMessageType::Consensus(message) => {
                Self::Consensus(GraphQLConsensusMessage(message))
            }
            KinetMessageType::BlockSyncRequest(_) => {
                Self::BlockSyncRequest(GraphQLBlockSyncRequestMessage(message))
            }
            KinetMessageType::BlockSyncResponse(_) => {
                Self::BlockSyncResponse(GraphQLBlockSyncResponseMessage(message))
            }
            KinetMessageType::ForwardedTx(txs) => Self::ForwardedTx(GraphQLForwardedTxMessage {
                count: txs.len().try_into().unwrap_or(i64::MAX),
            }),
            KinetMessageType::StateSyncMessage(_) => {
                Self::StateSyncMessage(GraphQLStateSyncMessage)
            }
        }
    }
}

#[derive(Union)]
pub(crate) enum GraphQLKinetMessage<'s> {
    Consensus(GraphQLConsensusMessage<'s>),
    BlockSyncRequest(GraphQLBlockSyncRequestMessage<'s>),
    BlockSyncResponse(GraphQLBlockSyncResponseMessage<'s>),
    ForwardedTx(GraphQLForwardedTxMessage),
    StateSyncMessage(GraphQLStateSyncMessage),
}

pub(crate) struct GraphQLBlockSyncRequestMessage<'s>(&'s KinetMessageType);

#[Object]
impl GraphQLBlockSyncRequestMessage<'_> {
    async fn kind(&self) -> &'static str {
        "BlockSyncRequest"
    }

    async fn request_type(&self) -> &'static str {
        match self.request() {
            BlockSyncRequestMessage::Headers(_) => "HEADERS",
            BlockSyncRequestMessage::Payload(_) => "PAYLOAD",
        }
    }

    async fn block_range(&self) -> Option<GraphQLBlockSyncRange> {
        match self.request() {
            BlockSyncRequestMessage::Headers(block_range) => {
                Some(GraphQLBlockSyncRange(*block_range))
            }
            BlockSyncRequestMessage::Payload(_) => None,
        }
    }

    async fn body_id(&self) -> Option<String> {
        match self.request() {
            BlockSyncRequestMessage::Headers(_) => None,
            BlockSyncRequestMessage::Payload(body_id) => Some(block_body_id_string(body_id)),
        }
    }
}

impl GraphQLBlockSyncRequestMessage<'_> {
    fn request(&self) -> &BlockSyncRequestMessage {
        let KinetMessageType::BlockSyncRequest(request) = self.0 else {
            unreachable!("GraphQLBlockSyncRequestMessage must wrap BlockSyncRequest");
        };
        request
    }
}

pub(crate) struct GraphQLBlockSyncResponseMessage<'s>(&'s KinetMessageType);

#[Object]
impl GraphQLBlockSyncResponseMessage<'_> {
    async fn kind(&self) -> &'static str {
        "BlockSyncResponse"
    }

    async fn response_type(&self) -> &'static str {
        match self.response() {
            BlockSyncResponseMessage::HeadersResponse(_) => "HEADERS",
            BlockSyncResponseMessage::PayloadResponse(_) => "PAYLOAD",
        }
    }

    async fn available(&self) -> bool {
        match self.response() {
            BlockSyncResponseMessage::HeadersResponse(BlockSyncHeadersResponse::Found(_))
            | BlockSyncResponseMessage::PayloadResponse(BlockSyncBodyResponse::Found(_)) => true,
            BlockSyncResponseMessage::HeadersResponse(BlockSyncHeadersResponse::NotAvailable(
                _,
            ))
            | BlockSyncResponseMessage::PayloadResponse(BlockSyncBodyResponse::NotAvailable(_)) => {
                false
            }
        }
    }

    async fn block_range(&self) -> Option<GraphQLBlockSyncRange> {
        match self.response() {
            BlockSyncResponseMessage::HeadersResponse(BlockSyncHeadersResponse::Found((
                block_range,
                _,
            )))
            | BlockSyncResponseMessage::HeadersResponse(BlockSyncHeadersResponse::NotAvailable(
                block_range,
            )) => Some(GraphQLBlockSyncRange(*block_range)),
            BlockSyncResponseMessage::PayloadResponse(_) => None,
        }
    }

    async fn blocks(&self) -> Vec<GraphQLBlockRef> {
        match self.response() {
            BlockSyncResponseMessage::HeadersResponse(BlockSyncHeadersResponse::Found((
                _,
                headers,
            ))) => headers.iter().map(GraphQLBlockRef::from_header).collect(),
            BlockSyncResponseMessage::HeadersResponse(BlockSyncHeadersResponse::NotAvailable(
                _,
            ))
            | BlockSyncResponseMessage::PayloadResponse(_) => Vec::new(),
        }
    }

    async fn body_id(&self) -> Option<String> {
        match self.response() {
            BlockSyncResponseMessage::PayloadResponse(BlockSyncBodyResponse::Found(body)) => {
                Some(block_body_id_string(&body.get_id()))
            }
            BlockSyncResponseMessage::PayloadResponse(BlockSyncBodyResponse::NotAvailable(
                body_id,
            )) => Some(block_body_id_string(body_id)),
            BlockSyncResponseMessage::HeadersResponse(_) => None,
        }
    }
}

impl GraphQLBlockSyncResponseMessage<'_> {
    fn response(
        &self,
    ) -> &BlockSyncResponseMessage<SignatureType, SignatureCollectionType, ExecutionProtocolType>
    {
        let KinetMessageType::BlockSyncResponse(response) = self.0 else {
            unreachable!("GraphQLBlockSyncResponseMessage must wrap BlockSyncResponse");
        };
        response
    }
}

struct GraphQLBlockSyncRange(BlockRange);

#[Object]
impl GraphQLBlockSyncRange {
    async fn last_block_id(&self) -> String {
        block_id_string(&self.0.last_block_id)
    }

    async fn num_blocks(&self) -> GraphQLSeqNum {
        GraphQLSeqNum::new(self.0.num_blocks)
    }
}

pub(crate) struct GraphQLForwardedTxMessage {
    count: i64,
}

#[Object]
impl GraphQLForwardedTxMessage {
    async fn kind(&self) -> &'static str {
        "ForwardedTx"
    }

    async fn count(&self) -> i64 {
        self.count
    }
}

pub(crate) struct GraphQLStateSyncMessage;

#[Object]
impl GraphQLStateSyncMessage {
    async fn kind(&self) -> &'static str {
        "StateSyncMessage"
    }
}

pub(crate) struct GraphQLConsensusMessage<'s>(&'s VerifiedConsensusMessageType);

#[Object]
impl<'s> GraphQLConsensusMessage<'s> {
    async fn round(&self) -> GraphQLRound {
        GraphQLRound::new(self.0.get_round())
    }
    async fn message(&self) -> GraphQLConsensusMessageType<'s> {
        self.0.into()
    }
}

impl<'s> From<&'s VerifiedConsensusMessageType> for GraphQLConsensusMessageType<'s> {
    fn from(message: &'s VerifiedConsensusMessageType) -> Self {
        match &message.deref().message {
            ProtocolMessage::Proposal(proposal) => Self::Proposal(GraphQLProposal(proposal)),
            ProtocolMessage::Vote(vote) => Self::Vote(GraphQLVote(vote)),
            ProtocolMessage::Timeout(timeout) => Self::Timeout(GraphQLTimeout(timeout)),
            ProtocolMessage::RoundRecovery(round_recovery) => {
                Self::RoundRecovery(GraphQLRoundRecovery(round_recovery))
            }
            ProtocolMessage::NoEndorsement(no_endorsement) => {
                Self::NoEndorsement(GraphQLNoEndorsement(no_endorsement))
            }
            ProtocolMessage::AdvanceRound(advance_round) => {
                Self::AdvanceRound(GraphQLAdvanceRound(advance_round))
            }
        }
    }
}

#[derive(Union)]
enum GraphQLConsensusMessageType<'s> {
    Proposal(GraphQLProposal<'s>),
    Vote(GraphQLVote<'s>),
    Timeout(GraphQLTimeout<'s>),
    RoundRecovery(GraphQLRoundRecovery<'s>),
    NoEndorsement(GraphQLNoEndorsement<'s>),
    AdvanceRound(GraphQLAdvanceRound<'s>),
}

struct GraphQLProposal<'s>(
    &'s ProposalMessage<SignatureType, SignatureCollectionType, ExecutionProtocolType>,
);
#[Object]
impl<'s> GraphQLProposal<'s> {
    async fn seq_num(&self) -> GraphQLSeqNum {
        GraphQLSeqNum::new(self.0.tip.block_header.seq_num)
    }

    async fn block(&self) -> GraphQLBlockRef {
        GraphQLBlockRef::from_header(&self.0.tip.block_header)
    }
}

struct GraphQLVote<'s>(&'s VoteMessage<SignatureCollectionType>);
#[Object]
impl<'s> GraphQLVote<'s> {
    async fn round(&self) -> GraphQLRound {
        GraphQLRound::new(self.0.vote.round)
    }

    async fn block_id(&self) -> String {
        block_id_string(&self.0.vote.id)
    }
}

struct GraphQLTimeout<'s>(
    &'s TimeoutMessage<SignatureType, SignatureCollectionType, ExecutionProtocolType>,
);
#[Object]
impl<'s> GraphQLTimeout<'s> {
    async fn round(&self) -> GraphQLRound {
        GraphQLRound::new(self.0 .0.tminfo.round)
    }

    async fn high_extend_type(&self) -> &'static str {
        match &self.0 .0.high_extend {
            HighExtendVote::Tip(_, _) => "TIP",
            HighExtendVote::Qc(_) => "QC",
        }
    }

    async fn block_id(&self) -> String {
        match &self.0 .0.high_extend {
            HighExtendVote::Tip(tip, _) => block_id_string(&tip.block_header.get_id()),
            HighExtendVote::Qc(qc) => block_id_string(&qc.get_block_id()),
        }
    }
}

struct GraphQLRoundRecovery<'s>(
    &'s RoundRecoveryMessage<SignatureType, SignatureCollectionType, ExecutionProtocolType>,
);
#[Object]
impl<'s> GraphQLRoundRecovery<'s> {
    async fn round(&self) -> GraphQLRound {
        GraphQLRound::new(self.0.round)
    }

    async fn block_id(&self) -> String {
        block_id_string(&self.0.tc.high_extend.qc().get_block_id())
    }
}

struct GraphQLNoEndorsement<'s>(&'s NoEndorsementMessage<SignatureCollectionType>);
#[Object]
impl<'s> GraphQLNoEndorsement<'s> {
    async fn round(&self) -> GraphQLRound {
        GraphQLRound::new(self.0.msg.round)
    }
}

struct GraphQLAdvanceRound<'s>(
    &'s AdvanceRoundMessage<SignatureType, SignatureCollectionType, ExecutionProtocolType>,
);
#[Object]
impl<'s> GraphQLAdvanceRound<'s> {
    async fn round(&self) -> GraphQLRound {
        GraphQLRound::new(self.0.last_round_certificate.round() + Round(1))
    }

    async fn block_id(&self) -> String {
        block_id_string(&self.0.last_round_certificate.qc().get_block_id())
    }
}

#[cfg(test)]
mod tests {
    use bytes::Bytes;

    use super::{GraphQLKinetMessage, KinetMessageType};

    #[test]
    fn forwarded_tx_message_converts_without_panicking() {
        let message = KinetMessageType::ForwardedTx(vec![Bytes::new()].into());

        assert!(matches!(
            GraphQLKinetMessage::from(&message),
            GraphQLKinetMessage::ForwardedTx(_)
        ));
    }
}
