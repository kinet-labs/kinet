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

use kinet_executor::ExecutorMetrics;

kinet_executor::metric_consts! {
    pub GAUGE_RAPTORCAST_AUTH_AUTHENTICATED_UDP_BYTES_WRITTEN {
        name: "kinet.raptorcast.auth.authenticated_udp_bytes_written",
        help: "Bytes written via authenticated UDP",
    }
    pub GAUGE_RAPTORCAST_AUTH_NON_AUTHENTICATED_UDP_BYTES_WRITTEN {
        name: "kinet.raptorcast.auth.non_authenticated_udp_bytes_written",
        help: "Bytes written via non-authenticated UDP",
    }
    pub GAUGE_RAPTORCAST_AUTH_AUTHENTICATED_UDP_BYTES_READ {
        name: "kinet.raptorcast.auth.authenticated_udp_bytes_read",
        help: "Bytes read via authenticated UDP",
    }
    pub GAUGE_RAPTORCAST_AUTH_NON_AUTHENTICATED_UDP_BYTES_READ {
        name: "kinet.raptorcast.auth.non_authenticated_udp_bytes_read",
        help: "Bytes read via non-authenticated UDP",
    }
}

kinet_wireauth::define_metric_names!(UDP_METRICS, "udp");
kinet_wireauth::define_metric_names!(DIRECT_UDP_METRICS, "direct_udp");

pub(crate) fn init_socket_executor_metrics() -> ExecutorMetrics {
    ExecutorMetrics::with_metric_defs(&[
        GAUGE_RAPTORCAST_AUTH_AUTHENTICATED_UDP_BYTES_WRITTEN,
        GAUGE_RAPTORCAST_AUTH_NON_AUTHENTICATED_UDP_BYTES_WRITTEN,
        GAUGE_RAPTORCAST_AUTH_AUTHENTICATED_UDP_BYTES_READ,
        GAUGE_RAPTORCAST_AUTH_NON_AUTHENTICATED_UDP_BYTES_READ,
    ])
}
