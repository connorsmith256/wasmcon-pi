use serde_json::json;
use wasmbus_rpc::actor::prelude::*;
use wasmcloud_interface_httpserver::{HttpRequest, HttpResponse, HttpServer, HttpServerReceiver};
use waveshareoled_interface::{
    DrawMessageInput, Event, WaveshareSubscriber, WaveshareSubscriberReceiver, Waveshareoled,
    WaveshareoledSender,
};

#[derive(Debug, Default, Actor, HealthResponder)]
#[services(Actor, HttpServer, WaveshareSubscriber)]
struct WaveshareoledActor {}

/// Implementation of HttpServer trait methods
#[async_trait]
impl HttpServer for WaveshareoledActor {
    async fn handle_request(&self, ctx: &Context, req: &HttpRequest) -> RpcResult<HttpResponse> {
        let censor_filter = censor::Censor::Standard + censor::Censor::Sex;
        let name = censor_filter.censor(req.path.trim_start_matches('/'));
        let (body, status_code) = match WaveshareoledSender::new()
            .draw_message(
                ctx,
                &DrawMessageInput {
                    message: format!("Hello, {name}, I am Roman!",),
                },
            )
            .await
        {
            Ok(v) => (
                json!({ "response": "Come try out Cosmonic! https://app.cosmonic.com" }),
                200,
            ),
            // Ensure we properly return database errors as server errors
            Err(e) => (json!({ "error": e.to_string() }), 500),
        };

        HttpResponse::json(body, status_code)
    }
}

#[async_trait]
impl WaveshareSubscriber for WaveshareoledActor {
    async fn handle_event(&self, ctx: &Context, evt: &Event) -> RpcResult<()> {
        match evt.as_str() {
            "button2" => {
                WaveshareoledSender::new()
                    .draw_message(
                        ctx,
                        &DrawMessageInput {
                            message: format!("Hello, I'm Roman!"),
                        },
                    )
                    .await
            }
            _ => {
                WaveshareoledSender::new()
                    .draw_message(
                        ctx,
                        &DrawMessageInput {
                            message: format!("Hello, I'm Roman!\nYou pressed {evt}"),
                        },
                    )
                    .await
            }
        }
    }
}
