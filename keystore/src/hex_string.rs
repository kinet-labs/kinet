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

use hex;
use serde::{de::Error, Deserialize, Deserializer, Serializer};

pub fn serialize_bytes_to_hex_string<S>(bytes: &Vec<u8>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let hex_string = hex::encode(bytes);
    serializer.serialize_str(&hex_string)
}

pub fn deserialize_bytes_from_hex_string<'d, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
where
    D: Deserializer<'d>,
{
    let hex_string: &str = Deserialize::deserialize(deserializer)?;
    hex::decode(hex_string).map_err(D::Error::custom)
}
