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

pub use crate::env::stub::{
    KeyPair, MerkleRoot, NodeId, OpaqueChunkHeader, ProposalSignature, PubKey, Signature,
    SignatureCollection, Stake, ValidatorData, VoteAggregation,
};

// TODO: fill in the actual production implementation for above types

const _: () = crate::spec::assert_env::<
    NodeId,
    Stake,
    PubKey,
    KeyPair,
    Signature,
    ValidatorData,
    SignatureCollection,
    VoteAggregation<'_>,
    MerkleRoot,
    ProposalSignature,
    OpaqueChunkHeader,
>();
