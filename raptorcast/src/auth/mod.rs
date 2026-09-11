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

pub mod framing;
pub mod metrics;
pub mod protocol;
pub mod socket;

pub use framing::{AuthPacketFramer, LeanUdpFramer, LeanUdpFramingError, NopScore};
pub use metrics::{
    GAUGE_RAPTORCAST_AUTH_AUTHENTICATED_UDP_BYTES_READ,
    GAUGE_RAPTORCAST_AUTH_AUTHENTICATED_UDP_BYTES_WRITTEN,
    GAUGE_RAPTORCAST_AUTH_NON_AUTHENTICATED_UDP_BYTES_READ,
    GAUGE_RAPTORCAST_AUTH_NON_AUTHENTICATED_UDP_BYTES_WRITTEN,
};
pub use protocol::{AuthenticationProtocol, NoopAuthProtocol, NoopHeader, WireAuthProtocol};
pub use socket::{
    AuthRecvMsg, AuthenticatedSocketHandle, DualSocketHandle, FramedAuthenticatedSocketHandle,
    FramedRecvError,
};
