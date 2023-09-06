//! Implementation for cosmonic:waveshareoled
//!

use std::collections::HashMap;
use std::convert::Infallible;
use std::sync::Arc;

use anyhow::{anyhow, ensure, Context as _};
use embedded_graphics::mono_font::{ascii, MonoTextStyleBuilder};
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::Point;
use embedded_graphics::text::{Baseline, Text};
use embedded_graphics::Drawable;
use futures::try_join;
use rppal::gpio::{Gpio, OutputPin, Trigger};
use rppal::hal::Delay;
use rppal::spi::{self, Spi};
use sh1106::prelude::*;
use tokio::sync::{broadcast, mpsc, RwLock};
use tokio::task::{spawn, spawn_blocking, JoinHandle};
use tokio::time::{sleep, Duration};
use tracing::{debug, error, instrument};
use wasmbus_rpc::{core::LinkDefinition, provider::prelude::*};
use waveshareoled_interface::{
    DrawMessageInput, WaveshareSubscriber, WaveshareSubscriberSender, Waveshareoled,
    WaveshareoledReceiver, WrappedEvent,
};

enum DisplayCommand {
    Clear,
    Text(String),
}

fn command(spi: &mut Spi, dc: &mut OutputPin, byte: u8) -> anyhow::Result<()> {
    dc.set_low();
    let n = spi.write(&[byte]).context("failed to write byte to SPI")?;
    ensure!(n == 1, "short write");
    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let hd = load_host_data()?;

    let mut spi = Spi::new(
        spi::Bus::Spi0,
        spi::SlaveSelect::Ss0,
        10000000,
        spi::Mode::Mode0,
    )
    .context("failed to connect to SPI")?;

    let gpio = Gpio::new().context("failed to get GPIO")?;

    let _bl = gpio.get(18).context("failed to get BL")?;
    let btn1 = gpio.get(21).context("failed to get BTN1")?;
    let btn2 = gpio.get(20).context("failed to get BTN2")?;
    let btn3 = gpio.get(16).context("failed to get BTN3")?;
    let cs = gpio.get(8).context("failed to get CS")?;
    let dc = gpio.get(24).context("failed to get DC")?;
    let js_down = gpio.get(19).context("failed to get Joystick down")?;
    let js_left = gpio.get(5).context("failed to get Joystick left")?;
    let js_press = gpio.get(13).context("failed to get Joystick press")?;
    let js_right = gpio.get(26).context("failed to get Joystick right")?;
    let js_up = gpio.get(6).context("failed to get Joystick up")?;
    let rst = gpio.get(25).context("failed to get RST")?;

    let mut btn1 = btn1.into_input_pullup();
    let mut btn2 = btn2.into_input_pullup();
    let mut btn3 = btn3.into_input_pullup();
    let cs = cs.into_output();
    let mut dc = dc.into_output_low();
    let mut js_down = js_down.into_input_pullup();
    let mut js_left = js_left.into_input_pullup();
    let mut js_press = js_press.into_input_pullup();
    let mut js_right = js_right.into_input_pullup();
    let mut js_up = js_up.into_input_pullup();
    let mut rst = rst.into_output();

    command(&mut spi, &mut dc, 0xAE)?; // turn off oled panel
    command(&mut spi, &mut dc, 0x02)?; // -set low column address
    command(&mut spi, &mut dc, 0x10)?; // -set high column address
    command(&mut spi, &mut dc, 0x40)?; // set start line address  Set Mapping RAM Display Start Line (0x00~0x3F)
    command(&mut spi, &mut dc, 0x81)?; // set contrast control register
    command(&mut spi, &mut dc, 0xA0)?; // Set SEG/Column Mapping
    command(&mut spi, &mut dc, 0xC0)?; // Set COM/Row Scan Direction
    command(&mut spi, &mut dc, 0xA6)?; // set normal display
    command(&mut spi, &mut dc, 0xA8)?; // set multiplex ratio(1 to 64)
    command(&mut spi, &mut dc, 0x3F)?; // 1/64 duty
    command(&mut spi, &mut dc, 0xD3)?; // set display offset    Shift Mapping RAM Counter (0x00~0x3F)
    command(&mut spi, &mut dc, 0x00)?; // not offset
    command(&mut spi, &mut dc, 0xd5)?; // set display clock divide ratio/oscillator frequency
    command(&mut spi, &mut dc, 0x80)?; // set divide ratio, Set Clock as 100 Frames/Sec
    command(&mut spi, &mut dc, 0xD9)?; // set pre-charge period
    command(&mut spi, &mut dc, 0xF1)?; // Set Pre-Charge as 15 Clocks & Discharge as 1 Clock
    command(&mut spi, &mut dc, 0xDA)?; // set com pins hardware configuration
    command(&mut spi, &mut dc, 0x12)?;
    command(&mut spi, &mut dc, 0xDB)?; // set vcomh
    command(&mut spi, &mut dc, 0x40)?; // Set VCOM Deselect Level
    command(&mut spi, &mut dc, 0x20)?; // Set Page Addressing Mode (0x00/0x01/0x02)
    command(&mut spi, &mut dc, 0x02)?; //
    command(&mut spi, &mut dc, 0xA4)?; //  Disable Entire Display On (0xa4/0xa5)
    command(&mut spi, &mut dc, 0xA6)?; //  Disable Inverse Display On (0xa6/a7)

    sleep(Duration::from_millis(100)).await;
    command(&mut spi, &mut dc, 0xAF)?; // turn on oled panel

    let mut display: GraphicsMode<SpiInterface<Spi, OutputPin, OutputPin>> =
        sh1106::Builder::new().connect_spi(spi, dc, cs).into();
    display
        .reset(&mut rst, &mut Delay::new())
        .expect("failed to reset");
    display.init().unwrap();
    display.flush().unwrap();

    for pin in [
        &mut btn1,
        &mut btn2,
        &mut btn3,
        &mut js_left,
        &mut js_up,
        &mut js_right,
        &mut js_down,
        &mut js_press,
    ] {
        pin.set_interrupt(Trigger::RisingEdge)
            .context("failed to set interrupt")?;
    }

    let (event_tx, _) = broadcast::channel(1000);
    let event_handle: JoinHandle<anyhow::Result<()>> = spawn_blocking({
        let event_tx = event_tx.clone();
        move || loop {
            let (pin, _lvl) = gpio
                .poll_interrupts(
                    &[
                        &btn1, &btn2, &btn3, &js_left, &js_up, &js_right, &js_down, &js_press,
                    ],
                    false,
                    None,
                )
                .context("failed to poll")?
                .context("poll returned unexpectedly")?;
            // TODO: Debounce
            let event = if pin == btn1 {
                WrappedEvent::Button1Press
            } else if pin == btn2 {
                WrappedEvent::Button2Press
            } else if pin == btn3 {
                WrappedEvent::Button3Press
            } else if pin == js_left {
                WrappedEvent::JoystickLeft
            } else if pin == js_up {
                WrappedEvent::JoystickUp
            } else if pin == js_right {
                WrappedEvent::JoystickRight
            } else if pin == js_down {
                WrappedEvent::JoystickDown
            } else if pin == js_press {
                WrappedEvent::JoystickPressed
            } else {
                error!("oopsie, unknown button pressed");
                continue;
            };
            event_tx.send(event).context("failed to send event")?;
        }
    });

    let (display_tx, mut display_rx) = mpsc::channel(1000);
    let display_handle: JoinHandle<anyhow::Result<()>> = spawn(async move {
        let text_style = MonoTextStyleBuilder::new()
            .font(&ascii::FONT_5X8)
            .text_color(BinaryColor::On)
            .build();
        loop {
            match display_rx.recv().await.context("display channel closed")? {
                DisplayCommand::Clear => display.clear(),
                DisplayCommand::Text(text) => {
                    display.clear();
                    Text::with_baseline(&text, Point::zero(), text_style, Baseline::Top)
                        .draw(&mut display)
                        .context("failed to write text")?;
                    display.flush().expect("failed to flush");
                }
            }
        }
    });

    provider_run(
        WaveshareoledProvider {
            display: display_tx,
            events: event_tx,
            actor_subscribers: Default::default(),
        },
        hd,
        Some("Waveshareoled Provider".to_string()),
    )
    .await
    .map_err(|e| anyhow!(e.to_string()).context("failed to run provider"))?;

    try_join!(
        async {
            event_handle
                .await
                .context("failed to await event handle")?
                .context("event handle thread failed")
        },
        async {
            display_handle
                .await
                .context("failed to await display handle")?
                .context("display handle thread failed")
        }
    )?;

    eprintln!("Waveshareoled provider exiting");
    Ok(())
}

/// Implementation for cosmonic:waveshareoled
#[derive(Clone, Provider)]
#[services(Waveshareoled)]
struct WaveshareoledProvider {
    display: mpsc::Sender<DisplayCommand>,
    events: broadcast::Sender<WrappedEvent>,
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
        let mut events = self.events.subscribe();
        let owned_ld = ld.to_owned();
        actors.insert(
            ld.actor_id.clone(),
            spawn(async move {
                let default_context = Context::default();
                let sender = WaveshareSubscriberSender::for_actor(&owned_ld);
                loop {
                    match events.recv().await {
                        Ok(evt) => {
                            debug!("Received event: {evt:?}");

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

        self.display
            .send(DisplayCommand::Text(input.message.clone()))
            .await
            .map_err(|e| RpcError::Other(e.to_string()))?;
        Ok(())
    }

    async fn clear(&self, _ctx: &wasmbus_rpc::provider::prelude::Context) -> RpcResult<()> {
        self.display
            .send(DisplayCommand::Clear)
            .await
            .map_err(|e| RpcError::Other(e.to_string()))?;
        Ok(())
    }
}
