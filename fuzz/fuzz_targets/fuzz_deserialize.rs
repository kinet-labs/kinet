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
// CORPUS_FILTER=*.deserialize.bin
// TIMEOUT_QUICK=5m
//
// Environments:
//
// AFL_HANG_TMOUT=100
// AFL_EXIT_ON_TIME=300000

use bytes::Bytes;
use kinet_bls::BlsSignatureCollection;
use kinet_crypto::certificate_signature::CertificateSignaturePubKey;
use kinet_eth_types::EthExecutionProtocol;
use kinet_raptorcast::message::InboundRouterMessage;
use kinet_secp::SecpSignature;
use kinet_state::KinetMessage;

type SignatureType = SecpSignature;
type SignatureCollection = BlsSignatureCollection<CertificateSignaturePubKey<SignatureType>>;
type ExecutionProtocol = EthExecutionProtocol;
type Message = KinetMessage<SignatureType, SignatureCollection, ExecutionProtocol>;

fn main() {
    afl::fuzz!(|data: &[u8]| {
        let app_message = Bytes::copy_from_slice(data);
        let _ = InboundRouterMessage::<Message, SignatureType>::try_deserialize(&app_message);
    });
}
