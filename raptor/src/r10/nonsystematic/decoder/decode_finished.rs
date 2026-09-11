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

use crate::r10::nonsystematic::decoder::{BufferId, Decoder};

impl Decoder {
    pub fn decoding_done(&self) -> bool {
        self.num_source_symbols_paired == self.params.num_source_symbols()
    }

    pub fn source_symbol_to_buffer_id(&self, source_symbol_id: usize) -> Option<BufferId> {
        if source_symbol_id < self.params.num_source_symbols() {
            match self.intermediate_symbol_state[source_symbol_id].is_used_buffer_index() {
                None => None,
                Some(buffer_index) => {
                    let buffer = &self.buffer_state[usize::from(buffer_index)];

                    if buffer.intermediate_symbol_ids.len() == 1 {
                        debug_assert!(buffer.is_paired());

                        Some(self.buffer_index_to_buffer_id(buffer_index))
                    } else {
                        None
                    }
                }
            }
        } else {
            None
        }
    }
}
