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

use super::*;
use crate::shared::nft_sale::NftSale;

pub struct NftSaleGenerator {
    pub nft_sale: NftSale,
    pub tx_per_sender: usize,
}

impl Generator for NftSaleGenerator {
    fn handle_acct_group(
        &mut self,
        accts: &mut [SimpleAccount],
        ctx: &GenCtx,
    ) -> Vec<(TxEnvelope, Address, crate::shared::private_key::PrivateKey)> {
        let mut txs = Vec::with_capacity(self.tx_per_sender * accts.len());

        // for each sender, buy an nft from the sale
        for sender in accts {
            for _ in 0..self.tx_per_sender {
                let tx = self.nft_sale.construct_buy_tx(
                    sender,
                    ctx.base_fee,
                    ctx.chain_id,
                    ctx.set_tx_gas_limit,
                    ctx.priority_fee,
                );
                txs.push((tx, self.nft_sale.addr, sender.key.clone()));
            }
        }

        txs
    }
}
