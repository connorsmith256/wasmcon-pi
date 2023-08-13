use serde_json::json;
use wasmbus_rpc::actor::prelude::*;
use wasmcloud_interface_httpserver::{HttpRequest, HttpResponse, HttpServer, HttpServerReceiver};
use waveshareoled_interface::{Waveshareoled, WaveshareoledSender, DrawMessageInput};

#[derive(Debug, Default, Actor, HealthResponder)]
#[services(Actor, HttpServer)]
struct WaveshareoledActorActor {}

/// Implementation of HttpServer trait methods
#[async_trait]
impl HttpServer for WaveshareoledActorActor {
    async fn handle_request(&self, ctx: &Context, req: &HttpRequest) -> RpcResult<HttpResponse> {
        let resp = WaveshareoledSender::new().draw_message(
            ctx,
            &DrawMessageInput {
                message: req.path.clone(),
            },
        ).await;

        let (body, status_code) = match resp {
            Ok(v) => (json!({ "response": v }), 200),
            // Ensure we properly return database errors as server errors
            Err(e) => (json!({ "error": e.to_string() }), 500),
        };

        HttpResponse::json(body, status_code)
    }
}

