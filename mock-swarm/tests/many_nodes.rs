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
    verifier::MockSwarmVerifier,
};
use kinet_router_scheduler::{NoSerRouterConfig, RouterSchedulerBuilder};
use kinet_transformer::{
    DropTransformer, GenericTransformer, LatencyTransformer, RandLatencyTransformer, ID,
};
use kinet_types::{NodeId, Round, SeqNum};
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
fn many_nodes_noser() {
    // block commits every 2∆ on happy path; 2 * 20ms * 1024 + (vote_pace*1024) = 47s
    // and consensus starts with a timeout
    let runtime = Duration::from_secs(47);
    let delta = Duration::from_millis(20);
    let num_expected_blocks = 1024;
    let state_configs = make_state_configs::<NoSerSwarm>(
        40, // num_nodes
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
    swarm.batch_step_until(&mut UntilTerminator::new().until_tick(runtime));
    swarm_ledger_verification(&swarm, num_expected_blocks);

    let mut verifier = MockSwarmVerifier::default().tick_range(runtime, delta);
    let node_ids = swarm.states().keys().copied().collect_vec();
    verifier.metrics_happy_path(&node_ids, &swarm);

    assert!(verifier.verify(&swarm));
}

#[test]
fn many_nodes_noser_one_offline() {
    let delta = Duration::from_millis(20);
    let num_nodes = 10;
    let num_offline_nodes = 1;

    let state_configs = make_state_configs::<NoSerSwarm>(
        num_nodes,
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
                    vec![
                        GenericTransformer::Latency(LatencyTransformer::new(
                            Duration::from_millis(1),
                        )),
                        GenericTransformer::RandLatency(RandLatencyTransformer::new(
                            0,
                            delta - Duration::from_millis(1),
                        )),
                        GenericTransformer::Drop(
                            DropTransformer::new().drop_only_from(*all_peers.first().unwrap()),
                        ),
                    ],
                    vec![],
                    TimestamperConfig::default(),
                    seed.try_into().unwrap(),
                )
            })
            .collect(),
    );

    let mut swarm = swarm_config.build();

    let num_rounds = 100;
    swarm.batch_step_until(&mut UntilTerminator::new().until_round(Round(num_rounds)));
    let mut max_observed_local_timeouts = swarm
        .states()
        .values()
        .map(|node| node.state.metrics().consensus_events.local_timeout.get())
        .max()
        .unwrap();
    // subtract 1 for the initial timeout on startup
    max_observed_local_timeouts -= 1;

    let max_allowed_local_timeouts =
        (num_rounds * num_offline_nodes as u64).div_ceil(num_nodes.into());

    assert!(max_observed_local_timeouts <= max_allowed_local_timeouts);
}
