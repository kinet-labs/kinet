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

use kinet_consensus_types::{
    quorum_certificate::{QuorumCertificate, Rank},
    voting::Vote,
};
use kinet_crypto::NopSignature;
use kinet_testutil::signing::MockSignatures;
use kinet_types::*;

extern crate kinet_testutil;

#[test]
fn comparison() {
    let v_1 = Vote {
        round: Round(2),
        ..DontCare::dont_care()
    };

    let v_2 = Vote {
        round: Round(3),
        ..DontCare::dont_care()
    };

    let qc_1 = QuorumCertificate::<MockSignatures<NopSignature>>::new(
        v_1,
        MockSignatures::with_pubkeys(&[]),
    );
    let mut qc_2 = QuorumCertificate::<MockSignatures<NopSignature>>::new(
        v_2,
        MockSignatures::with_pubkeys(&[]),
    );

    assert!(Rank(qc_1.info) < Rank(qc_2.info));
    assert!(Rank(qc_2.info) > Rank(qc_1.info));

    qc_2.info.round = Round(2);

    assert_eq!(Rank(qc_1.info), Rank(qc_2.info));
}
