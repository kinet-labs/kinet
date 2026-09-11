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

use kinet_chain_config::{revision::ChainParams, MockChainConfig};
use kinet_consensus_types::{block::PassthruBlockPolicy, block_validator::MockValidator};
use kinet_crypto::certificate_signature::CertificateKeyPair;
use kinet_execution_state_read::InMemoryStateInner;
use kinet_mock_swarm::{
    mock::TimestamperConfig,
    mock_swarm::{Nodes, SwarmBuilder},
    node::NodeBuilder,
    swarm::{make_state_configs, swarm_ledger_verification},
    swarm_relation::NoSerSwarm,
    terminator::UntilTerminator,
};
use kinet_router_scheduler::{NoSerRouterConfig, RouterSchedulerBuilder};
use kinet_transformer::{GenericTransformer, LatencyTransformer, ID};
use kinet_types::{NodeId, SeqNum};
use kinet_updaters::{
    ledger::MockLedger, statesync::MockStateSyncExecutor, txpool::MockTxPoolExecutor,
    val_set::MockValSetUpdaterNop,
};
use kinet_validator::{simple_round_robin::SimpleRoundRobin, validator_set::ValidatorSetFactory};

static ON_TIME_CHAIN_PARAMS: ChainParams = ChainParams {
    tx_limit: 10_000,
    proposal_gas_limit: 300_000_000,
    proposal_byte_limit: 4_000_000,
    max_reserve_balance: 1_000_000_000_000_000_000,
    vote_pace: Duration::from_millis(500),
};

static LATE_CHAIN_PARAMS: ChainParams = ChainParams {
    tx_limit: 10_000,
    proposal_gas_limit: 300_000_000,
    proposal_byte_limit: 4_000_000,
    max_reserve_balance: 1_000_000_000_000_000_000,
    vote_pace: Duration::from_millis(5),
};

fn build_two_node_swarm(chain_params: &'static ChainParams, delta: Duration) -> Nodes<NoSerSwarm> {
    let chain_config = MockChainConfig::new(chain_params);
    let state_configs = make_state_configs::<NoSerSwarm>(
        2,
        ValidatorSetFactory::default,
        SimpleRoundRobin::default,
        || MockValidator,
        || PassthruBlockPolicy,
        || InMemoryStateInner::genesis(SeqNum(4)),
        SeqNum(4),
        delta,
        chain_config,
        SeqNum(100),
    );
    let all_peers: BTreeSet<_> = state_configs
        .iter()
        .map(|state_config| NodeId::new(state_config.key.pubkey()))
        .collect();

    SwarmBuilder::<NoSerSwarm>(
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
                    MockTxPoolExecutor::default().with_chain_params(chain_params),
                    MockLedger::new(state_read.clone()),
                    MockStateSyncExecutor::new(state_read),
                    vec![GenericTransformer::Latency(LatencyTransformer::new(delta))],
                    vec![],
                    TimestamperConfig::default(),
                    seed.try_into().unwrap(),
                )
            })
            .collect(),
    )
    .build()
}

fn run_until_block(swarm: &mut Nodes<NoSerSwarm>, block: usize) {
    while swarm
        .step_until(&mut UntilTerminator::new().until_block(block))
        .is_some()
    {}
}

#[test]
fn vote_delay_metrics_record_ready_after_timer_start_on_happy_path() {
    let delta = Duration::from_millis(100);
    let mut swarm = build_two_node_swarm(&ON_TIME_CHAIN_PARAMS, delta);

    run_until_block(&mut swarm, 64);
    swarm_ledger_verification(&swarm, 62);

    for node in swarm.states().values() {
        let metrics = node.state.metrics();
        let p50 = metrics.vote_delay.ready_after_timer_start_p50_ms.get();
        let p90 = metrics.vote_delay.ready_after_timer_start_p90_ms.get();
        let p99 = metrics.vote_delay.ready_after_timer_start_p99_ms.get();

        assert!(p50 > 0);
        assert!(p50 <= p90);
        assert!(p90 <= p99);
        assert!(p99 < ON_TIME_CHAIN_PARAMS.vote_pace.as_millis() as u64);
        assert!(metrics.consensus_events.local_timeout.get() <= 1);
    }
}

#[test]
fn vote_delay_metrics_record_large_ready_after_timer_start_when_vote_pace_is_too_small() {
    let delta = Duration::from_millis(100);
    let mut swarm = build_two_node_swarm(&LATE_CHAIN_PARAMS, delta);

    run_until_block(&mut swarm, 64);
    swarm_ledger_verification(&swarm, 62);

    for node in swarm.states().values() {
        let metrics = node.state.metrics();
        let p50 = metrics.vote_delay.ready_after_timer_start_p50_ms.get();
        let p90 = metrics.vote_delay.ready_after_timer_start_p90_ms.get();
        let p99 = metrics.vote_delay.ready_after_timer_start_p99_ms.get();

        assert!(p50 > LATE_CHAIN_PARAMS.vote_pace.as_millis() as u64);
        assert!(p50 <= p90);
        assert!(p90 <= p99);
    }
}
