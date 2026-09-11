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

use kinet_executor::ExecutorMetrics;

kinet_executor::metric_consts! {
    pub COUNTER_PEER_SCORE_RECORD_CONTRIBUTION_TOTAL {
        name: "kinet.peer_score.record_contribution.total",
        help: "Total non-zero contribution records processed by peer-score",
    }
    pub COUNTER_PEER_SCORE_NEWCOMER_ADMITTED {
        name: "kinet.peer_score.newcomer.admitted",
        help: "Total newcomer identities admitted into the bounded newcomer pool",
    }
    pub COUNTER_PEER_SCORE_NEWCOMER_REJECTED {
        name: "kinet.peer_score.newcomer.rejected",
        help: "Total newcomer identities rejected from the bounded newcomer pool",
    }
    pub COUNTER_PEER_SCORE_PROMOTION_SUCCEEDED {
        name: "kinet.peer_score.promotion.succeeded",
        help: "Total newcomer identities promoted into the promoted pool",
    }
    pub COUNTER_PEER_SCORE_PROMOTION_REJECTED {
        name: "kinet.peer_score.promotion.rejected",
        help: "Total promotion attempts rejected by the promoted pool admission policy",
    }
    pub COUNTER_PEER_SCORE_DEMOTION {
        name: "kinet.peer_score.demotion",
        help: "Total promoted identities demoted back into the newcomer pool",
    }
    pub GAUGE_PEER_SCORE_PROMOTED_SIZE {
        name: "kinet.peer_score.pool.promoted_size",
        help: "Current number of identities in the promoted pool",
    }
    pub GAUGE_PEER_SCORE_NEWCOMER_SIZE {
        name: "kinet.peer_score.pool.newcomer_size",
        help: "Current number of identities in the newcomer pool",
    }
    pub GAUGE_PEER_SCORE_TOTAL_SIZE {
        name: "kinet.peer_score.pool.total_size",
        help: "Current total number of identities tracked by peer-score",
    }
}

pub fn init_executor_metrics() -> ExecutorMetrics {
    ExecutorMetrics::with_metric_defs(&[
        COUNTER_PEER_SCORE_RECORD_CONTRIBUTION_TOTAL,
        COUNTER_PEER_SCORE_NEWCOMER_ADMITTED,
        COUNTER_PEER_SCORE_NEWCOMER_REJECTED,
        COUNTER_PEER_SCORE_PROMOTION_SUCCEEDED,
        COUNTER_PEER_SCORE_PROMOTION_REJECTED,
        COUNTER_PEER_SCORE_DEMOTION,
        GAUGE_PEER_SCORE_PROMOTED_SIZE,
        GAUGE_PEER_SCORE_NEWCOMER_SIZE,
        GAUGE_PEER_SCORE_TOTAL_SIZE,
    ])
}
