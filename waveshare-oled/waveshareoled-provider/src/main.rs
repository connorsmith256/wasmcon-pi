//! Implementation for cosmonic:waveshareoled
//!

mod phony;

use std::collections::HashMap;
use std::{convert::Infallible, sync::Arc};

use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use tracing::{debug, error, instrument};

use wasmbus_rpc::{core::LinkDefinition, provider::prelude::*};
use waveshareoled_interface::{
    DrawMessageInput, WaveshareSubscriber, WaveshareSubscriberSender, Waveshareoled,
    WaveshareoledReceiver, WrappedEvent,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let hd = load_host_data()?;

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;
    let (wrapper, sender) = match runtime.block_on(phony::Wrapper::new()) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Error setting up lib: {}", e);
            return Ok(());
        }
    };

    runtime.block_on(async {
        provider_run(
            WaveshareoledProvider {
                wrapper: Arc::new(RwLock::new(wrapper)),
                sender,
                actor_subscribers: Default::default(),
            },
            hd,
            Some("Waveshareoled Provider".to_string()),
        )
        .await
    })?;
    // in the unlikely case there are any stuck threads,
    // close them so the process has a clean exit
    runtime.shutdown_timeout(core::time::Duration::from_secs(10));

    eprintln!("Waveshareoled provider exiting");
    Ok(())
}

/// Implementation for cosmonic:waveshareoled
#[derive(Clone, Provider)]
#[services(Waveshareoled)]
struct WaveshareoledProvider {
    wrapper: Arc<RwLock<phony::Wrapper>>,
    sender: tokio::sync::broadcast::Sender<WrappedEvent>,
    actor_subscribers: Arc<RwLock<HashMap<String, JoinHandle<()>>>>,
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

        let mut actors = self.actor_subscribers.write().await;
        let mut rx = self.sender.subscribe();
        let owned_ld = ld.to_owned();
        actors.insert(
            ld.actor_id.clone(),
            tokio::spawn(async move {
                let default_context = Context::default();
                let sender = WaveshareSubscriberSender::for_actor(&owned_ld);
                loop {
                    match rx.recv().await {
                        Ok(evt) => {
                            debug!("Received event: {:?}", evt);

                            if let Err(e) = sender
                                .handle_event(&default_context, &evt.to_string())
                                .await
                            {
                                error!(error = %e, "Error sending event to actor");
                                continue;
                            }
                        }
                        Err(e) => {
                            debug!("Error receiving event: {:?}", e);
                            break;
                        }
                    }
                }
            }),
        );

        Ok(true)
    }

    /// Handle notification that a link is dropped: close the connection
    #[instrument(level = "info", skip(self))]
    async fn delete_link(&self, actor_id: &str) {
        debug!("deleting link for actor {}", actor_id);
        let handle = self.actor_subscribers.write().await.remove(actor_id);
        if let Some(handle) = handle {
            handle.abort();
        }
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

        self.wrapper
            .write()
            .await
            .draw_message(&input.message)
            .await
            .map_err(|e| RpcError::Other(e.to_string()))
    }

    async fn clear(&self, _ctx: &wasmbus_rpc::provider::prelude::Context) -> RpcResult<()> {
        self.wrapper
            .write()
            .await
            .draw_message("PROIVDER_DISPLAY_CLEAR")
            .await
            .map_err(|e| RpcError::Other(e.to_string()))
    }
}
