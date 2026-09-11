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

use kinet_crypto::certificate_signature::PubKey;
use kinet_types::{deserialize_pubkey, serialize_pubkey};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct BlockSyncPeersConfig<P: PubKey> {
    #[serde(bound = "P:PubKey")]
    pub peers: Vec<SyncPeerIdentityConfig<P>>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct StateSyncPeersConfig<P: PubKey> {
    pub expand_to_group: bool,
    /// When false, all inbound statesync requests are refused
    #[serde(default = "default_serve_statesync")]
    pub serve_statesync: bool,
    #[serde(bound = "P:PubKey")]
    pub init_peers: Vec<SyncPeerIdentityConfig<P>>,
}

fn default_serve_statesync() -> bool {
    true
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct SyncPeerIdentityConfig<P: PubKey> {
    #[serde(serialize_with = "serialize_pubkey::<_, P>")]
    #[serde(deserialize_with = "deserialize_pubkey::<_, P>")]
    #[serde(bound = "P:PubKey")]
    pub secp256k1_pubkey: P,
}
