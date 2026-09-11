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
use kinet_chain_config::{revision::ChainParams, MockChainConfig};
use kinet_consensus_types::{block::PassthruBlockPolicy, block_validator::MockValidator};
use kinet_crypto::certificate_signature::CertificateKeyPair;
use kinet_execution_state_read::InMemoryStateInner;
use kinet_mock_swarm::{
    mock::TimestamperConfig,
    mock_swarm::SwarmBuilder,
    node::NodeBuilder,
    swarm::{make_state_configs, swarm_ledger_verification},
    swarm_relation::NoSerSwarm,
    terminator::UntilTerminator,
    verifier::{happy_path_tick_by_block, MockSwarmVerifier},
};
use kinet_router_scheduler::{NoSerRouterConfig, RouterSchedulerBuilder};
use kinet_transformer::{GenericTransformer, LatencyTransformer, ID};
use kinet_types::{NodeId, SeqNum};
use kinet_updaters::{
    ledger::MockLedger, statesync::MockStateSyncExecutor, txpool::MockTxPoolExecutor,
    val_set::MockValSetUpdaterNop,
};
use kinet_validator::{simple_round_robin::SimpleRoundRobin, validator_set::ValidatorSetFactory};

static CHAIN_PARAMS: ChainParams = ChainParams {
    tx_limit: 10_000,
    proposal_gas_limit: 300_000_000,
    proposal_byte_limit: 4_000_000,
    max_reserve_balance: 1_000_000_000_000_000_000,
    vote_pace: Duration::from_millis(5),
};

#[test]
fn two_nodes_noser() {
    let delta = Duration::from_millis(100);
    let state_configs = make_state_configs::<NoSerSwarm>(
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
    let swarm_config = SwarmBuilder::<NoSerSwarm>(
        state_configs
            .into_iter()
            .enumerate()
            .map(|(seed, state_builder)| {
                let state_read = state_builder.state_read.clone();
                let validators = state_builder.locked_epoch_validators[0].clone();
                NodeBuilder::<NoSerSwarm>::new(
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
    while swarm
        .step_until(&mut UntilTerminator::new().until_block(1026))
        .is_some()
    {}

    let node_ids = swarm.states().keys().copied().collect_vec();
    // the calculation is correct with two nodes because NoSerRouterScheduler
    // always sends the message over the network/transformer, even for it's for
    // self. Otherwise the time is shorter with two nodes, like QUIC test
    let mut verifier =
        MockSwarmVerifier::default().tick_range(happy_path_tick_by_block(1026, delta), delta);
    verifier.metrics_happy_path(&node_ids, &swarm);
    assert!(verifier.verify(&swarm));

    swarm_ledger_verification(&swarm, 1024);
}
