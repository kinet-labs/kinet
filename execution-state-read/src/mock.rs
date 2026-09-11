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

use std::collections::BTreeMap;

use alloy_consensus::Header;
use alloy_primitives::Address;
use kinet_crypto::certificate_signature::{
    CertificateSignaturePubKey, CertificateSignatureRecoverable,
};
use kinet_eth_types::{EthAccount, EthHeader};
use kinet_types::{Balance, BlockId, Nonce, SeqNum, Stake};
use kinet_validator::signature_collection::{SignatureCollection, SignatureCollectionPubKeyType};

use crate::{ExecutionStateRead, ExecutionStateReadError};

#[derive(Debug, Default, Clone)]
pub struct NopExecutionStateRead {
    pub nonces: BTreeMap<Address, Nonce>,
    pub balances: BTreeMap<Address, Balance>,
}

impl<ST, SCT> ExecutionStateRead<ST, SCT> for NopExecutionStateRead
where
    ST: CertificateSignatureRecoverable,
    SCT: SignatureCollection<NodeIdPubKey = CertificateSignaturePubKey<ST>>,
{
    fn get_account_statuses<'a>(
        &mut self,
        _block_id: &BlockId,
        _seq_num: &SeqNum,
        _is_finalized: bool,
        addresses: impl Iterator<Item = &'a Address>,
    ) -> Result<Vec<Option<EthAccount>>, ExecutionStateReadError> {
        Ok(addresses
            .map(|address| {
                Some(EthAccount {
                    balance: self.balances.get(address).cloned().unwrap_or_default(),
                    nonce: self.nonces.get(address).cloned().unwrap_or_default(),
                    code_hash: None,
                    is_delegated: false,
                })
            })
            .collect())
    }

    fn get_execution_result(
        &mut self,
        _block_id: &BlockId,
        _seq_num: &SeqNum,
        _is_finalized: bool,
    ) -> Result<EthHeader, ExecutionStateReadError> {
        Ok(EthHeader(Header::default()))
    }

    /// Fetches earliest block from storage backend
    fn raw_read_earliest_finalized_block(&self) -> Option<SeqNum> {
        None
    }

    /// Fetches latest block from storage backend
    fn raw_read_latest_finalized_block(&self) -> Option<SeqNum> {
        None
    }

    fn read_valset_at_block(
        &mut self,
        block_num: SeqNum,
        requested_epoch: kinet_types::Epoch,
    ) -> Vec<(
        <SCT as SignatureCollection>::NodeIdPubKey,
        SignatureCollectionPubKeyType<SCT>,
        Stake,
    )> {
        vec![]
    }

    fn total_db_lookups(&self) -> u64 {
        0
    }
}
