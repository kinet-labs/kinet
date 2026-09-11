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

use std::{error::Error, fmt::Debug, fs::create_dir_all};

use bytes::Bytes;
use criterion::{criterion_group, Criterion};
use kinet_types::{Deserializable, Serializable};
use kinet_wal::wal::*;
use tempfile::{tempdir, TempDir};

const VOTE_SIZE: usize = 400;
const BLOCK_SIZE: usize = 32 * 10000;
const N_VALIDATORS: usize = 400;

// benchmark file io append only, without serde overhead
#[derive(Debug, Clone)]
struct Datablob {
    data: Vec<u8>,
}

impl Datablob {
    fn new(byte_len: usize) -> Self {
        Datablob {
            data: vec![0xbf; byte_len],
        }
    }
}

#[derive(Debug)]
struct ReadError {}

impl std::fmt::Display for ReadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <Self as Debug>::fmt(self, f)
    }
}

impl Error for ReadError {}

impl Serializable<Bytes> for Datablob {
    fn serialize(&self) -> Bytes {
        self.data.clone().into()
    }
}

impl Deserializable<[u8]> for Datablob {
    type ReadError = ReadError;

    fn deserialize(message: &[u8]) -> Result<Self, Self::ReadError> {
        Ok(Datablob {
            data: message.to_vec(),
        })
    }
}
struct Bencher {
    data: Datablob,
    logger: WALogger<Datablob>,
    _tmpdir: TempDir,
}

impl Bencher {
    fn new(byte_len: usize) -> Bencher {
        let tmpdir = tempdir().unwrap();
        create_dir_all(tmpdir.path()).unwrap();
        let file_path = tmpdir.path().join("wal");
        let config = WALoggerConfig::new(
            file_path, false, // sync
        );
        Bencher {
            data: Datablob::new(byte_len),
            logger: config.build().unwrap(),
            _tmpdir: tmpdir,
        }
    }

    fn append(&mut self) {
        self.logger.push(&self.data).unwrap()
    }
}

fn bench_block(c: &mut Criterion) {
    let mut bencher = Bencher::new(BLOCK_SIZE);

    c.bench_function("block", |b| b.iter(|| bencher.append()));
}

fn bench_vote(c: &mut Criterion) {
    let mut bencher = Bencher::new(VOTE_SIZE);

    c.bench_function("vote", |b| {
        b.iter(|| {
            for _ in 0..N_VALIDATORS {
                bencher.append()
            }
        })
    });
}

criterion_group!(bench, bench_block, bench_vote);

#[cfg(target_os = "linux")]
criterion::criterion_main!(bench);

#[cfg(not(target_os = "linux"))]
fn main() {
    println!("Linux only benchmark");
}
