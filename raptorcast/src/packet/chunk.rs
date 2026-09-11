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

use bytes::BytesMut;
use kinet_crypto::certificate_signature::PubKey;

use crate::util::{Recipient, UdpMessage};

pub struct Chunk<PT: PubKey> {
    chunk_id: usize,
    recipient: Recipient<PT>,
    payload: BytesMut,
}

impl<PT: PubKey> From<Chunk<PT>> for UdpMessage<PT> {
    fn from(chunk: Chunk<PT>) -> Self {
        Self {
            recipient: chunk.recipient,
            stride: chunk.payload.len(),
            payload: chunk.payload.freeze(),
        }
    }
}

impl<PT: PubKey> Chunk<PT> {
    pub fn new(chunk_id: usize, recipient: Recipient<PT>, payload: BytesMut) -> Self {
        debug_assert!(chunk_id <= u16::MAX as usize);
        Self {
            chunk_id,
            recipient,
            payload,
        }
    }

    pub fn recipient(&self) -> &Recipient<PT> {
        &self.recipient
    }

    pub fn chunk_id(&self) -> usize {
        self.chunk_id
    }

    pub fn payload(&self) -> &[u8] {
        &self.payload
    }

    pub fn payload_mut(&mut self) -> &mut [u8] {
        &mut self.payload
    }
}
