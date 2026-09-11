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

use std::{collections::BTreeSet, time::Duration};

use itertools::Itertools;
use kinet_bls::BlsSignatureCollection;
use kinet_chain_config::{
    revision::{ChainParams, MockChainRevision},
    MockChainConfig,
};
use kinet_consensus_types::{
    block::{MockExecutionProtocol, PassthruBlockPolicy},
    block_validator::MockValidator,
};
use kinet_crypto::certificate_signature::CertificateSignaturePubKey;
use kinet_execution_state_read::{InMemoryState, InMemoryStateInner};
use kinet_executor_glue::KinetEvent;
use kinet_mock_swarm::{
    mock::TimestamperConfig,
    mock_swarm::SwarmBuilder,
    node::NodeBuilder,
    swarm::{make_state_configs, swarm_ledger_verification},
    swarm_relation::SwarmRelation,
    terminator::UntilTerminator,
    verifier::{happy_path_tick_by_block, MockSwarmVerifier},
};
use kinet_router_scheduler::{NoSerRouterConfig, NoSerRouterScheduler, RouterSchedulerBuilder};
use kinet_secp::SecpSignature;
use kinet_state::{KinetMessage, VerifiedKinetMessage};
use kinet_transformer::{GenericTransformer, GenericTransformerPipeline, LatencyTransformer, ID};
use kinet_types::{NodeId, SeqNum};
use kinet_updaters::{
    ledger::MockLedger, statesync::MockStateSyncExecutor, txpool::MockTxPoolExecutor,
    val_set::MockValSetUpdaterNop,
};
use kinet_validator::{simple_round_robin::SimpleRoundRobin, validator_set::ValidatorSetFactory};
struct BLSSwarm;
impl SwarmRelation for BLSSwarm {
    type SignatureType = SecpSignature;
    type SignatureCollectionType =
        BlsSignatureCollection<CertificateSignaturePubKey<Self::SignatureType>>;
    type ExecutionProtocolType = MockExecutionProtocol;
    type ExecutionStateReadType = InMemoryState<Self::SignatureType, Self::SignatureCollectionType>;
    type BlockPolicyType = PassthruBlockPolicy;
    type ChainConfigType = MockChainConfig;
    type ChainRevisionType = MockChainRevision;

    type TransportMessage = VerifiedKinetMessage<
        Self::SignatureType,
        Self::SignatureCollectionType,
        Self::ExecutionProtocolType,
    >;

    type BlockValidator = MockValidator;
    type ValidatorSetTypeFactory =
        ValidatorSetFactory<CertificateSignaturePubKey<Self::SignatureType>>;
    type LeaderElection = SimpleRoundRobin<CertificateSignaturePubKey<Self::SignatureType>>;
    type Ledger =
        MockLedger<Self::SignatureType, Self::SignatureCollectionType, Self::ExecutionProtocolType>;

    type RouterScheduler = NoSerRouterScheduler<
        CertificateSignaturePubKey<Self::SignatureType>,
        KinetMessage<
            Self::SignatureType,
            Self::SignatureCollectionType,
            Self::ExecutionProtocolType,
        >,
        VerifiedKinetMessage<
            Self::SignatureType,
            Self::SignatureCollectionType,
            Self::ExecutionProtocolType,
        >,
    >;

    type Pipeline = GenericTransformerPipeline<
        CertificateSignaturePubKey<Self::SignatureType>,
        Self::TransportMessage,
    >;

    type ValSetUpdater = MockValSetUpdaterNop<
        Self::SignatureType,
        Self::SignatureCollectionType,
        Self::ExecutionProtocolType,
    >;
    type TxPoolExecutor = MockTxPoolExecutor<
        Self::SignatureType,
        Self::SignatureCollectionType,
        Self::ExecutionProtocolType,
        Self::BlockPolicyType,
        Self::ExecutionStateReadType,
        Self::ChainConfigType,
        Self::ChainRevisionType,
    >;
    type StateSyncExecutor = MockStateSyncExecutor<
        Self::SignatureType,
        Self::SignatureCollectionType,
        Self::ExecutionProtocolType,
    >;
}

static CHAIN_PARAMS: ChainParams = ChainParams {
    tx_limit: 10_000,
    proposal_gas_limit: 300_000_000,
    proposal_byte_limit: 4_000_000,
    max_reserve_balance: 1_000_000_000_000_000_000,
    vote_pace: Duration::from_millis(5),
};

#[test]
fn two_nodes_bls() {
    tracing_subscriber::fmt::init();

    let delta = Duration::from_millis(20);

    let state_configs = make_state_configs::<BLSSwarm>(
        2, // num_nodes
        ValidatorSetFactory::default,
        SimpleRoundRobin::default,
        || MockValidator,
        || PassthruBlockPolicy,
        || InMemoryStateInner::genesis(SeqNum(4)),
        SeqNum(4),                           // execution_delay
        delta,                               // delta
        MockChainConfig::new(&CHAIN_PARAMS), // chain config
        SeqNum(100),                         // state_sync_threshold
    );
    let all_peers: BTreeSet<_> = state_configs
        .iter()
        .map(|state_config| NodeId::new(state_config.key.pubkey()))
        .collect();
    let swarm_config = SwarmBuilder::<BLSSwarm>(
        state_configs
            .into_iter()
            .enumerate()
            .map(|(seed, state_builder)| {
                let state_read = state_builder.state_read.clone();
                let validators = state_builder.locked_epoch_validators[0].clone();
                NodeBuilder::<BLSSwarm>::new(
                    ID::new(NodeId::new(state_builder.key.pubkey())),
                    state_builder,
                    NoSerRouterConfig::new(all_peers.clone()).build(),
                    MockValSetUpdaterNop::new(validators.validators, SeqNum(2000)),
                    MockTxPoolExecutor::default().with_chain_params(&CHAIN_PARAMS),
                    MockLedger::new(state_read.clone()),
                    MockStateSyncExecutor::new(state_read),
                    vec![GenericTransformer::Latency(LatencyTransformer::new(delta))],
                    vec![],
                    TimestamperConfig::default(),
                    seed.try_into().unwrap(),
                )
            })
            .collect(),
    );

    let mut swarm = swarm_config.build();
    while let Some((_, _, event)) = swarm.step_until(&mut UntilTerminator::new().until_block(100)) {
        // assert that we can round-trip KinetEvents properly
        let event_encoded = alloy_rlp::encode(&event);
        let event_roundtrip: KinetEvent<
            <BLSSwarm as SwarmRelation>::SignatureType,
            <BLSSwarm as SwarmRelation>::SignatureCollectionType,
            <BLSSwarm as SwarmRelation>::ExecutionProtocolType,
        > = alloy_rlp::decode_exact(&event_encoded).unwrap_or_else(|err| {
            panic!("failed to rlp roundtrip event={:?}, err={:?}", event, err)
        });
        assert_eq!(
            serde_json::to_string(&event).unwrap(),
            serde_json::to_string(&event_roundtrip).unwrap(),
            "failed to rlp roundtrip KinetEvent"
        );
    }
    swarm_ledger_verification(&swarm, 98);

    // the calculation is correct with two nodes because NoSerRouterScheduler
    // always sends the message over the network/transformer, even for it's for
    // self
    let mut verifier =
        MockSwarmVerifier::default().tick_range(happy_path_tick_by_block(100, delta), delta);

    let node_ids = swarm.states().keys().copied().collect_vec();
    verifier.metrics_happy_path(&node_ids, &swarm);

    assert!(verifier.verify(&swarm));
}
