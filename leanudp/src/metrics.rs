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
    pub GAUGE_LEANUDP_POOL_PRIORITY_MESSAGES {
        name: "kinet.leanudp.pool.priority_messages",
        help: "Priority pool in-flight message count",
    }
    pub GAUGE_LEANUDP_POOL_REGULAR_MESSAGES {
        name: "kinet.leanudp.pool.regular_messages",
        help: "Regular pool in-flight message count",
    }
    pub GAUGE_LEANUDP_POOL_DEDICATED_MESSAGES {
        name: "kinet.leanudp.pool.dedicated_messages",
        help: "Dedicated per-identity pools in-flight message count",
    }
    pub COUNTER_LEANUDP_DECODE_FRAGMENTS_RECEIVED {
        name: "kinet.leanudp.decode.fragments_received",
        help: "Total LeanUDP fragments received",
    }
    pub COUNTER_LEANUDP_DECODE_BYTES_RECEIVED {
        name: "kinet.leanudp.decode.bytes_received",
        help: "Total fragment payload bytes received, excluding LeanUDP headers",
    }
    pub COUNTER_LEANUDP_DECODE_FRAGMENTS_PRIORITY {
        name: "kinet.leanudp.decode.fragments_priority",
        help: "Total fragments routed to the priority pool",
    }
    pub COUNTER_LEANUDP_DECODE_BYTES_PRIORITY {
        name: "kinet.leanudp.decode.bytes_priority",
        help: "Total fragment payload bytes routed to the priority pool, excluding LeanUDP headers",
    }
    pub COUNTER_LEANUDP_DECODE_FRAGMENTS_REGULAR {
        name: "kinet.leanudp.decode.fragments_regular",
        help: "Total fragments routed to the regular pool",
    }
    pub COUNTER_LEANUDP_DECODE_BYTES_REGULAR {
        name: "kinet.leanudp.decode.bytes_regular",
        help: "Total fragment payload bytes routed to the regular pool, excluding LeanUDP headers",
    }
    pub COUNTER_LEANUDP_DECODE_FRAGMENTS_DEDICATED {
        name: "kinet.leanudp.decode.fragments_dedicated",
        help: "Total fragments routed to dedicated per-identity pools",
    }
    pub COUNTER_LEANUDP_DECODE_BYTES_DEDICATED {
        name: "kinet.leanudp.decode.bytes_dedicated",
        help: "Total fragment payload bytes routed to dedicated per-identity pools, excluding LeanUDP headers",
    }
    pub COUNTER_LEANUDP_DECODE_MESSAGES_COMPLETED {
        name: "kinet.leanudp.decode.messages_completed",
        help: "Total messages fully reassembled by the decoder",
    }
    pub COUNTER_LEANUDP_DECODE_BYTES_COMPLETED {
        name: "kinet.leanudp.decode.bytes_completed",
        help: "Total message payload bytes fully reassembled by the decoder",
    }
    pub COUNTER_LEANUDP_DECODE_EVICTED_TIMEOUT {
        name: "kinet.leanudp.decode.evicted_timeout",
        help: "Total in-flight messages evicted after timing out",
    }
    pub COUNTER_LEANUDP_DECODE_EVICTED_RANDOM {
        name: "kinet.leanudp.decode.evicted_random",
        help: "Total in-flight messages randomly evicted under pool pressure",
    }
    pub COUNTER_LEANUDP_ERROR_INVALID_HEADER {
        name: "kinet.leanudp.error.invalid_header",
        help: "Total packets rejected due to malformed LeanUDP headers",
    }
    pub COUNTER_LEANUDP_ERROR_UNSUPPORTED_VERSION {
        name: "kinet.leanudp.error.unsupported_version",
        help: "Total packets rejected due to an unsupported LeanUDP version",
    }
    pub COUNTER_LEANUDP_ERROR_IDENTITY_LIMIT {
        name: "kinet.leanudp.error.identity_limit",
        help: "Total fragments rejected because the sender exceeded its in-flight message limit",
    }
    pub COUNTER_LEANUDP_ERROR_DUPLICATE_FRAGMENT {
        name: "kinet.leanudp.error.duplicate_fragment",
        help: "Total fragments rejected because the sequence number was already present",
    }
    pub COUNTER_LEANUDP_ERROR_TOO_MANY_FRAGMENTS {
        name: "kinet.leanudp.error.too_many_fragments",
        help: "Total fragments rejected because the message exceeded the fragment count limit",
    }
    pub COUNTER_LEANUDP_ERROR_CONFLICTING_END {
        name: "kinet.leanudp.error.conflicting_end",
        help: "Total fragments rejected because they conflicted with the declared message end",
    }
    pub COUNTER_LEANUDP_ERROR_MESSAGE_TOO_LARGE {
        name: "kinet.leanudp.error.message_too_large",
        help: "Total fragments rejected because the reassembled message would exceed the size limit",
    }
    pub COUNTER_LEANUDP_ERROR_POOL_FULL {
        name: "kinet.leanudp.error.pool_full",
        help: "Total fragments rejected because no pool capacity was available",
    }
    pub COUNTER_LEANUDP_ENCODE_MESSAGES {
        name: "kinet.leanudp.encode.messages",
        help: "Total messages submitted to the LeanUDP encoder",
    }
    pub COUNTER_LEANUDP_ENCODE_FRAGMENTS {
        name: "kinet.leanudp.encode.fragments",
        help: "Total fragments produced by the LeanUDP encoder",
    }
    pub COUNTER_LEANUDP_ENCODE_BYTES {
        name: "kinet.leanudp.encode.bytes",
        help: "Total message payload bytes submitted to the LeanUDP encoder, excluding headers",
    }
    pub COUNTER_LEANUDP_ENCODE_ERROR_TOO_LARGE {
        name: "kinet.leanudp.encode.error.too_large",
        help: "Total messages rejected by the encoder because they exceed the fragment limit",
    }
}

pub fn init_encoder_metrics() -> ExecutorMetrics {
    ExecutorMetrics::with_metric_defs(&[
        COUNTER_LEANUDP_ENCODE_MESSAGES,
        COUNTER_LEANUDP_ENCODE_FRAGMENTS,
        COUNTER_LEANUDP_ENCODE_BYTES,
        COUNTER_LEANUDP_ENCODE_ERROR_TOO_LARGE,
    ])
}

pub fn init_decoder_metrics() -> ExecutorMetrics {
    ExecutorMetrics::with_metric_defs(&[
        GAUGE_LEANUDP_POOL_PRIORITY_MESSAGES,
        GAUGE_LEANUDP_POOL_REGULAR_MESSAGES,
        GAUGE_LEANUDP_POOL_DEDICATED_MESSAGES,
        COUNTER_LEANUDP_DECODE_FRAGMENTS_RECEIVED,
        COUNTER_LEANUDP_DECODE_BYTES_RECEIVED,
        COUNTER_LEANUDP_DECODE_FRAGMENTS_PRIORITY,
        COUNTER_LEANUDP_DECODE_BYTES_PRIORITY,
        COUNTER_LEANUDP_DECODE_FRAGMENTS_REGULAR,
        COUNTER_LEANUDP_DECODE_BYTES_REGULAR,
        COUNTER_LEANUDP_DECODE_FRAGMENTS_DEDICATED,
        COUNTER_LEANUDP_DECODE_BYTES_DEDICATED,
        COUNTER_LEANUDP_DECODE_MESSAGES_COMPLETED,
        COUNTER_LEANUDP_DECODE_BYTES_COMPLETED,
        COUNTER_LEANUDP_DECODE_EVICTED_TIMEOUT,
        COUNTER_LEANUDP_DECODE_EVICTED_RANDOM,
        COUNTER_LEANUDP_ERROR_INVALID_HEADER,
        COUNTER_LEANUDP_ERROR_UNSUPPORTED_VERSION,
        COUNTER_LEANUDP_ERROR_IDENTITY_LIMIT,
        COUNTER_LEANUDP_ERROR_DUPLICATE_FRAGMENT,
        COUNTER_LEANUDP_ERROR_TOO_MANY_FRAGMENTS,
        COUNTER_LEANUDP_ERROR_CONFLICTING_END,
        COUNTER_LEANUDP_ERROR_MESSAGE_TOO_LARGE,
        COUNTER_LEANUDP_ERROR_POOL_FULL,
    ])
}
