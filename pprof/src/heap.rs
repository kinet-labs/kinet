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

use actix_web::{HttpResponse, Responder, web};

pub(crate) async fn handle_get_heap() -> impl Responder {
    #[cfg(not(feature = "jemallocator"))]
    return HttpResponse::NotImplemented()
        .body("Heap profiling is not available: jemallocator feature is not enabled");

    #[cfg(feature = "jemallocator")]
    {
        let mut prof_ctl = jemalloc_pprof::PROF_CTL.as_ref().unwrap().lock().await;
        if prof_ctl.activated() {
            match prof_ctl.dump_pprof() {
                Ok(pprof) => HttpResponse::Ok().body(pprof),
                Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
            }
        } else {
            HttpResponse::Conflict().body("heap profiling not activated")
        }
    }
}

#[derive(serde::Deserialize)]
pub(crate) struct ProfilingConfig {
    active: Option<bool>,
}

pub(crate) async fn handle_update_prof_config(
    config: web::Json<ProfilingConfig>,
) -> impl Responder {
    #[cfg(not(feature = "jemallocator"))]
    return HttpResponse::NotImplemented()
        .body("Profiling configuration is not available: jemallocator feature is not enabled");

    #[cfg(feature = "jemallocator")]
    {
        let mut prof_ctl = jemalloc_pprof::PROF_CTL.as_ref().unwrap().lock().await;
        let mut updates = Vec::new();

        if let Some(active) = config.active {
            if active {
                match prof_ctl.activate() {
                    Ok(_) => updates.push("profiling enabled"),
                    Err(err) => return HttpResponse::InternalServerError().body(err.to_string()),
                }
            } else {
                match prof_ctl.deactivate() {
                    Ok(_) => updates.push("profiling disabled"),
                    Err(err) => return HttpResponse::InternalServerError().body(err.to_string()),
                }
            }
        }

        if updates.is_empty() {
            return HttpResponse::BadRequest()
                .body("No valid configuration parameters provided. Expected JSON with 'active'");
        }

        HttpResponse::Ok().body(format!(
            "Successfully updated profiling configuration: {}",
            updates.join(", ")
        ))
    }
}
