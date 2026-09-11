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

// Fuzz runner config:
//
// CORPUS_FILTER=*.parse_message.bin
// TIMEOUT_QUICK=5m
//
// Environments:
//
// AFL_HANG_TMOUT=100
// AFL_EXIT_ON_TIME=300000
// AFL_INPUT_LEN_MAX=1500

use bytes::Bytes;
use kinet_raptorcast::{
    parser::packet_parser::{ChunkValidationEnv, RaptorcastPacket},
    udp::ChunkSignatureVerifier,
};
use kinet_crypto::certificate_signature::CertificateKeyPair;
use kinet_secp::mock::{MockSecpKeyPair, MockSecpSignature};
use kinet_types::NodeId;

fn main() {
    let self_id = NodeId::new(MockSecpKeyPair::from_seed(0).pubkey());

    afl::fuzz!(|data: &[u8]| {
        let mut sig_cache = ChunkSignatureVerifier::<MockSecpSignature>::new().with_cache(1);
        let payload = Bytes::copy_from_slice(data);
        let Ok(packet) = RaptorcastPacket::parse(&payload) else {
            return;
        };

        let _ = packet.validate_chunk(
            &payload,
            ChunkValidationEnv {
                signature_verifier: &mut sig_cache,
                max_age_ms: u64::MAX,
                bypass_rate_limiter: |_| true,
                self_id: &self_id,
            },
        );
    });
}
