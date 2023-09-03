//! Implementation for cosmonic:waveshareoled
//!

mod phony;

use std::convert::Infallible;
use std::path::PathBuf;

use tracing::{debug, instrument};

use wasmbus_rpc::{core::LinkDefinition, provider::prelude::*};
use waveshareoled_interface::{DrawMessageInput, Waveshareoled, WaveshareoledReceiver};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let hd = load_host_data()?;

    let lib_path = match phony::setup() {
        Ok(path) => path,
        Err(e) => {
            eprintln!("Error setting up lib: {}", e);
            return Ok(());
        }
    };
    eprintln!("wrote lib to {lib_path:?}");

    provider_start(
        WaveshareoledProvider { lib_path },
        hd,
        Some("Waveshareoled Provider".to_string()),
    )?;

    eprintln!("Waveshareoled provider exiting");
    Ok(())
}

/// Implementation for cosmonic:waveshareoled
#[derive(Clone, Provider)]
#[services(Waveshareoled)]
struct WaveshareoledProvider {
    lib_path: PathBuf,
}
// use default implementations of provider message handlers
impl ProviderDispatch for WaveshareoledProvider {}

/// Handle provider control commands
/// put_link (new actor link command), del_link (remove link command), and shutdown
#[async_trait]
impl ProviderHandler for WaveshareoledProvider {
    /// Provider should perform any operations needed for a new link,
    /// including setting up per-actor resources, and checking authorization.
    /// If the link is allowed, return true, otherwise return false to deny the link.
    #[instrument(level = "info", skip(self))]
    async fn put_link(&self, ld: &LinkDefinition) -> RpcResult<bool> {
        debug!("putting link for actor {:?}", ld);

        Ok(true)
    }

    /// Handle notification that a link is dropped: close the connection
    #[instrument(level = "info", skip(self))]
    async fn delete_link(&self, actor_id: &str) {
        debug!("deleting link for actor {}", actor_id);
    }

    /// Handle shutdown request with any cleanup necessary
    async fn shutdown(&self) -> std::result::Result<(), Infallible> {
        Ok(())
    }
}

/// Handle Messaging methods
#[async_trait]
impl Waveshareoled for WaveshareoledProvider {
    async fn draw_message(
        &self,
        _ctx: &wasmbus_rpc::provider::prelude::Context,
        input: &DrawMessageInput,
    ) -> RpcResult<()> {
        debug!("Drawing message: {:?}", input);

        phony::draw_message(&self.lib_path.to_string_lossy(), &input.message)
            .await
            .map_err(|e| RpcError::Other(e.to_string()))
    }
}
