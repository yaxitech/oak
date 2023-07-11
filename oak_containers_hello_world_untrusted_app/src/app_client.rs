//
// Copyright 2023 The Project Oak Authors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

mod proto {
    pub mod oak {
        pub mod containers {
            pub mod example {
                tonic::include_proto!("oak.containers.example");
            }
        }
    }
}

use anyhow::Context;
use proto::oak::containers::example::{
    trusted_application_client::TrustedApplicationClient as GrpcTrustedApplicationClient,
    HelloRequest,
};
use tonic::transport::{Endpoint, Uri};
use tower::service_fn;

// Virtio VSOCK does not use URIs, hence this URI will never be used.
// It is defined purely since in order to create a channel a URI has to
// be supplied to create an `Endpoint`.
static IGNORED_ENDPOINT_URI: &str = "file://[::]:0";

/// Utility struct used to interface with the launcher
pub struct TrustedApplicationClient {
    inner: GrpcTrustedApplicationClient<tonic::transport::channel::Channel>,
}

impl TrustedApplicationClient {
    async fn get_stream_with_trusted_app(
        cid: u32,
        port: u32,
    ) -> Result<tokio_vsock::VsockStream, anyhow::Error> {
        let (vsock_stream, _) = tokio_vsock::VsockListener::bind(cid, port)
            .context("failed to bind vsock listener")?
            // The trusted app is the only party that will connect to this listener.
            // Hence the first incoming stream must be the trusted app.
            //
            // Effectively this means that while on the gRPC layer the trusted app
            // listens for invocations from the untrusted app, the inverse is
            // true on the layer of the VSOCK connection. There the untrusted
            // app listens for connections, the trusted app connects to the
            // listener.
            .accept()
            .await
            .context("failed to accept vsock connection")?;

        Ok(vsock_stream)
    }
    pub async fn create(cid: u32, port: u32) -> Result<Self, Box<dyn std::error::Error>> {
        let inner: GrpcTrustedApplicationClient<tonic::transport::channel::Channel> = {
            let channel = Endpoint::try_from(IGNORED_ENDPOINT_URI)
                .context("couldn't form endpoint")?
                .connect_with_connector(service_fn(move |_: Uri| {
                    TrustedApplicationClient::get_stream_with_trusted_app(cid, port)
                }))
                .await
                .context("couldn't connect to untrusted app VSOCK socket")?;
            GrpcTrustedApplicationClient::new(channel)
        };
        Ok(Self { inner })
    }

    pub async fn hello(&mut self, name: &str) -> Result<String, Box<dyn std::error::Error>> {
        let greeting = self
            .inner
            .hello(HelloRequest {
                name: name.to_string(),
            })
            .await?
            .into_inner()
            .greeting;
        Ok(greeting)
    }
}
