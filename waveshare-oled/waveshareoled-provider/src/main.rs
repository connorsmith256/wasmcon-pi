//! Implementation for cosmonic:waveshareoled
//!

use std::convert::Infallible;
use std::io::prelude::*;
use std::sync::Arc;

use anyhow::{anyhow, bail, Context};
use rand::{rngs::StdRng, Rng, SeedableRng};
use rppal::gpio::{Gpio, OutputPin};
use rppal::spi::{Bus, Mode, SlaveSelect, Spi};
use tokio::sync::{Mutex, RwLock};
use tracing::{debug, instrument};

use wasmbus_rpc::{core::LinkDefinition, provider::prelude::*};
use waveshareoled_interface::{DrawMessageInput, Waveshareoled, WaveshareoledReceiver};

const RST_PIN: u8 = 25;
const DC_PIN: u8 = 24;
const CS_PIN: u8 = 8;
const BL_PIN: u8 = 18;

// const HIGH: u8 = 1;
// const LOW: u8 = 0;

const LCD_WIDTH: u32 = 128;
const LCD_HEIGHT: u32 = 64;
const LCD_BUFFER_SIZE: usize = (LCD_WIDTH * LCD_HEIGHT) as usize / 8;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let hd = load_host_data()?;

    let gpio = Gpio::new()
        .map_err(anyhow::Error::from)
        .context("failed to get GPIO")?;

    // init GPIO lines
    let mut rst = gpio
        .get(RST_PIN)
        .context("failed to get RST pin")?
        .into_output();
    let mut dc = gpio
        .get(DC_PIN)
        .context("failed to get DC pin")?
        .into_output();
    let mut cs = gpio
        .get(CS_PIN)
        .context("failed to get CS pin")?
        .into_output();
    let mut bl = gpio
        .get(BL_PIN)
        .context("failed to get BL pin")?
        .into_output();

    // init SPI
    let mut spi = Spi::new(Bus::Spi0, SlaveSelect::Ss0, 10_000_000, Mode::Mode0)
        .map_err(anyhow::Error::from)
        .context("failed to get SPI")?;

    // init GPIO values
    cs.set_low();
    bl.set_high();
    dc.set_low();

    // reset display
    reset(&mut rst).context("failed to reset display")?;

    // init display
    command(&mut dc, &mut spi, 0xAE)?; //--turn off oled panel
    command(&mut dc, &mut spi, 0x02)?; //---set low column address
    command(&mut dc, &mut spi, 0x10)?; //---set high column address
    command(&mut dc, &mut spi, 0x40)?; //--set start line address  Set Mapping RAM Display Start Line (0x00~0x3F)
    command(&mut dc, &mut spi, 0x81)?; //--set contrast control register
    command(&mut dc, &mut spi, 0xA0)?; //--Set SEG/Column Mapping
    command(&mut dc, &mut spi, 0xC0)?; //Set COM/Row Scan Direction
    command(&mut dc, &mut spi, 0xA6)?; //--set normal display
    command(&mut dc, &mut spi, 0xA8)?; //--set multiplex ratio(1 to 64)
    command(&mut dc, &mut spi, 0x3F)?; //--1/64 duty
    command(&mut dc, &mut spi, 0xD3)?; //-set display offset    Shift Mapping RAM Counter (0x00~0x3F)
    command(&mut dc, &mut spi, 0x00)?; //-not offset
    command(&mut dc, &mut spi, 0xd5)?; //--set display clock divide ratio/oscillator frequency
    command(&mut dc, &mut spi, 0x80)?; //--set divide ratio, Set Clock as 100 Frames/Sec
    command(&mut dc, &mut spi, 0xD9)?; //--set pre-charge period
    command(&mut dc, &mut spi, 0xF1)?; //Set Pre-Charge as 15 Clocks & Discharge as 1 Clock
    command(&mut dc, &mut spi, 0xDA)?; //--set com pins hardware configuration
    command(&mut dc, &mut spi, 0x12)?; //
    command(&mut dc, &mut spi, 0xDB)?; //--set vcomh
    command(&mut dc, &mut spi, 0x40)?; //Set VCOM Deselect Level
    command(&mut dc, &mut spi, 0x20)?; //-Set Page Addressing Mode (0x00/0x01/0x02)
    command(&mut dc, &mut spi, 0x02)?; //
    command(&mut dc, &mut spi, 0xA4)?; // Disable Entire Display On (0xa4/0xa5)
    command(&mut dc, &mut spi, 0xA6)?; // Disable Inverse Display On (0xa6/a7)
    std::thread::sleep(std::time::Duration::from_millis(100));
    command(&mut dc, &mut spi, 0xAF)?; //--turn on oled panel

    provider_start(
        WaveshareoledProvider {
            gpio: Arc::new(Mutex::new(GPIO { rst, dc })),
            // spi: Arc::new(RwLock::new(spi)),
            rng: Arc::new(Mutex::new(StdRng::from_entropy())),
        },
        hd,
        Some("Waveshareoled Provider".to_string()),
    )?;

    eprintln!("Waveshareoled provider exiting");
    Ok(())
}

fn reset(rst: &mut OutputPin) -> anyhow::Result<()> {
    rst.set_high();
    std::thread::sleep(std::time::Duration::from_millis(100));
    rst.set_low();
    std::thread::sleep(std::time::Duration::from_millis(100));
    rst.set_high();
    std::thread::sleep(std::time::Duration::from_millis(100));
    Ok(())
}

fn command(dc: &mut OutputPin, spi: &mut Spi, byte: u8) -> anyhow::Result<()> {
    dc.set_low();

    let written = spi
        .write(&[byte])
        .map_err(anyhow::Error::from)
        .context("failed to write SPI byte")?;

    if written != 1 {
        anyhow::bail!("failed to write SPI byte");
    } else {
        Ok(())
    }
}

fn draw_buffer(
    dc: &mut OutputPin,
    spi: &mut Spi,
    buf: &[u8; LCD_BUFFER_SIZE],
) -> anyhow::Result<()> {
    for page in 0..8 {
        command(dc, spi, 0xB0 + page as u8)?; //set page address
        command(dc, spi, 0x02)?; //set low column address
        command(dc, spi, 0x10)?; //set high column address

        dc.set_high();

        for col in 0..LCD_WIDTH {
            let byte = 0xFF;
            // let byte = !buf[(col + LCD_WIDTH * page) as usize];

            let _ = spi
                .write(&[byte])
                .map_err(anyhow::Error::from)
                .context("failed to write SPI byte")?;
            // command(dc, spi, byte).context("failed to write buffer byte")?;
        }
    }
    Ok(())
}

struct GPIO {
    rst: OutputPin,
    dc: OutputPin,
}

/// Implementation for cosmonic:waveshareoled
#[derive(Clone, Provider)]
#[services(Waveshareoled)]
struct WaveshareoledProvider {
    gpio: Arc<Mutex<GPIO>>,
    // spi: Arc<RwLock<Spi>>,
    rng: Arc<Mutex<StdRng>>,
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

        let mut buf = [0u8; LCD_BUFFER_SIZE];

        let mut rng = self.rng.lock().await;
        for i in 0..LCD_BUFFER_SIZE {
            buf[i] = rng.gen();
        }

        // let mut spi = self.spi.write().await;
        let mut spi = Spi::new(Bus::Spi0, SlaveSelect::Ss0, 10_000_000, Mode::Mode0)
            .map_err(anyhow::Error::from)
            .context("failed to get SPI")
            .map_err(|e| RpcError::Other(format!("{e}")))?;

        let mut gpio = self.gpio.lock().await;
        let mut dc = &mut gpio.dc;
        draw_buffer(&mut dc, &mut spi, &buf).map_err(|e| RpcError::Other(format!("{e}")))
    }
}

// // TODO: is this needed?
// impl Drop for WaveshareoledProvider {
//     fn drop(&mut self) {
//         self.gpio.lock().await.rst.set_low();
//         self.gpio.lock().await.dc.set_low();
//     }
// }
