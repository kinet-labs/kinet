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

use alloy_rlp::{RlpDecodable, RlpEncodable};
use kinet_crypto::{
    certificate_signature::{CertificateSignaturePubKey, CertificateSignatureRecoverable},
    signing_domain,
};
use kinet_types::ExecutionProtocol;
use kinet_validator::signature_collection::SignatureCollection;
use serde::{Deserialize, Serialize};

use crate::{block::ConsensusBlockHeader, no_endorsement::FreshProposalCertificate};

#[derive(Clone, Debug, PartialEq, Eq, RlpEncodable, RlpDecodable, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
#[rlp(trailing)]
pub struct ConsensusTip<ST, SCT, EPT>
where
    ST: CertificateSignatureRecoverable,
    SCT: SignatureCollection<NodeIdPubKey = CertificateSignaturePubKey<ST>>,
    EPT: ExecutionProtocol,
{
    pub block_header: ConsensusBlockHeader<ST, SCT, EPT>,
    /// block_header.block_round leader signature over block_header
    signature: ST,

    pub fresh_certificate: Option<FreshProposalCertificate<SCT>>,
}

impl<ST, SCT, EPT> ConsensusTip<ST, SCT, EPT>
where
    ST: CertificateSignatureRecoverable,
    SCT: SignatureCollection<NodeIdPubKey = CertificateSignaturePubKey<ST>>,
    EPT: ExecutionProtocol,
{
    pub fn new(
        keypair: &ST::KeyPairType,
        block_header: ConsensusBlockHeader<ST, SCT, EPT>,
        fresh_certificate: Option<FreshProposalCertificate<SCT>>,
    ) -> Self {
        let rlp_block_header = alloy_rlp::encode(&block_header);
        let signature = ST::sign::<signing_domain::Tip>(&rlp_block_header, keypair);
        Self {
            block_header,
            signature,
            fresh_certificate,
        }
    }

    pub fn signature_author(&self) -> Result<CertificateSignaturePubKey<ST>, ST::Error> {
        let rlp_block_header = alloy_rlp::encode(&self.block_header);
        self.signature
            .recover_pubkey::<signing_domain::Tip>(&rlp_block_header)
    }
}
