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

use kinet_crypto::{certificate_signature::PubKey, NopPubKey};
use kinet_types::NodeId;

pub(crate) fn dummy_node_id() -> NodeId<NopPubKey> {
    test_node_id(0)
}

pub(crate) fn test_node_id(byte: u8) -> NodeId<NopPubKey> {
    NodeId::new(NopPubKey::from_bytes(&[byte; 32]).unwrap())
}
