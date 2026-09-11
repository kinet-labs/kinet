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

use std::task::{Context, Waker};

use tracing::warn;

// Flag used to skip polling the ipc socket until EthTxPool has been reset after state syncing.
// TODO(andr-dev): Remove this once RPC uses execution events to decide if state syncing is done.
#[derive(Default)]
pub struct EthTxPoolResetTrigger {
    has_been_reset: bool,
    waker: Option<Waker>,
}

impl EthTxPoolResetTrigger {
    pub fn set_reset(&mut self) {
        if self.has_been_reset {
            warn!("EthTxPoolResetTrigger set_reset called when already reset");
            return;
        }

        self.has_been_reset = true;

        if let Some(waker) = self.waker.take() {
            waker.wake();
        }
    }

    pub fn poll_is_ready(&mut self, cx: &Context) -> bool {
        if self.has_been_reset {
            return true;
        }

        if self
            .waker
            .as_ref()
            .is_none_or(|waker| !waker.will_wake(cx.waker()))
        {
            self.waker = Some(cx.waker().clone());
        }

        false
    }
}
