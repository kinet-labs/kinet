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

use kinet_crypto::certificate_signature::{
    CertificateKeyPair, CertificateSignaturePubKey, CertificateSignatureRecoverable,
};
use kinet_types::{NodeId, Stake};
use kinet_validator::{
    signature_collection::{SignatureCollection, SignatureCollectionKeyPairType},
    validator_mapping::ValidatorMapping,
    validator_set::ValidatorSetTypeFactory,
};

use crate::signing::{create_certificate_keys, create_keys};

pub fn create_keys_w_validators<ST, SCT, VTF>(
    num_nodes: u32,
    validator_set_factory: VTF,
) -> (
    Vec<ST::KeyPairType>,
    Vec<SignatureCollectionKeyPairType<SCT>>,
    VTF::ValidatorSetType,
    ValidatorMapping<CertificateSignaturePubKey<ST>, SignatureCollectionKeyPairType<SCT>>,
)
where
    ST: CertificateSignatureRecoverable,
    SCT: SignatureCollection<NodeIdPubKey = CertificateSignaturePubKey<ST>>,
    VTF: ValidatorSetTypeFactory<NodeIdPubKey = CertificateSignaturePubKey<ST>>,
{
    let keys = create_keys::<ST>(num_nodes);
    let certificate_keys = create_certificate_keys::<SCT>(num_nodes);
    let (validators, validator_mapping) =
        complete_keys_w_validators::<ST, SCT, VTF>(&keys, &certificate_keys, validator_set_factory);
    (keys, certificate_keys, validators, validator_mapping)
}

pub fn complete_keys_w_validators<ST, SCT, VTF>(
    keys: &[ST::KeyPairType],
    certificate_keys: &[SignatureCollectionKeyPairType<SCT>],
    validator_set_factory: VTF,
) -> (
    VTF::ValidatorSetType,
    ValidatorMapping<CertificateSignaturePubKey<ST>, SignatureCollectionKeyPairType<SCT>>,
)
where
    ST: CertificateSignatureRecoverable,
    SCT: SignatureCollection<NodeIdPubKey = CertificateSignaturePubKey<ST>>,
    VTF: ValidatorSetTypeFactory<NodeIdPubKey = CertificateSignaturePubKey<ST>>,
{
    let staking_list = keys
        .iter()
        .map(|k| NodeId::new(k.pubkey()))
        .zip(std::iter::repeat(Stake::ONE))
        .collect::<Vec<_>>();

    let voting_identity = keys
        .iter()
        .map(|k| NodeId::new(k.pubkey()))
        .zip(certificate_keys.iter().map(|k| k.pubkey()))
        .collect::<Vec<_>>();

    let validators = validator_set_factory
        .create(staking_list)
        .expect("create validator set");
    let validator_mapping = ValidatorMapping::new(voting_identity);

    (validators, validator_mapping)
}
