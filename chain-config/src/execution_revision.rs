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

#[allow(non_camel_case_types)]
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum KinetExecutionRevision {
    V_ZERO,
    V_ONE,
    V_TWO,
    V_FOUR,
}

impl KinetExecutionRevision {
    pub const LATEST: Self = Self::V_FOUR;
}

impl KinetExecutionRevision {
    pub fn execution_chain_params(&self) -> &'static ExecutionChainParams {
        match &self {
            Self::V_ZERO => &EXECUTION_CHAIN_PARAMS_V_ZERO,
            Self::V_ONE => &EXECUTION_CHAIN_PARAMS_V_ONE,
            Self::V_TWO => &EXECUTION_CHAIN_PARAMS_V_TWO,
            Self::V_FOUR => &EXECUTION_CHAIN_PARAMS_V_FOUR,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ExecutionChainParams {
    pub max_code_size: usize,
    pub prague_enabled: bool,
    pub validate_system_txs: bool,
}

const EXECUTION_CHAIN_PARAMS_V_ZERO: ExecutionChainParams = ExecutionChainParams {
    max_code_size: 24 * 1024,
    prague_enabled: false,
    validate_system_txs: false,
};

const EXECUTION_CHAIN_PARAMS_V_ONE: ExecutionChainParams = ExecutionChainParams {
    max_code_size: 24 * 1024,
    prague_enabled: false,
    validate_system_txs: false,
};

const EXECUTION_CHAIN_PARAMS_V_TWO: ExecutionChainParams = ExecutionChainParams {
    max_code_size: 128 * 1024,
    prague_enabled: false,
    validate_system_txs: false,
};

const EXECUTION_CHAIN_PARAMS_V_FOUR: ExecutionChainParams = ExecutionChainParams {
    max_code_size: 128 * 1024,
    prague_enabled: true,
    validate_system_txs: true,
};
