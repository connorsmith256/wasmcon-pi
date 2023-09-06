use std::fs::File;
use std::io::Write;
use std::{env, process::Stdio};

use anyhow::anyhow;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, Command};
use tokio::sync::broadcast::{channel, Sender};
use tokio::task::JoinHandle;
use tracing::{debug, error};
use waveshareoled_interface::WrappedEvent;

const LIB_STR: &str = include_str!("display.py");
const FONT_DATA: &[u8] = include_bytes!("Font.ttf");

pub struct Wrapper {
    stdin: ChildStdin,
    _child: Child,
    // TODO: channel here
    _handle: JoinHandle<()>,
}

impl Wrapper {
    pub async fn new() -> anyhow::Result<(Self, Sender<WrappedEvent>)> {
        let dir = env::temp_dir();
        let py_path = dir.as_path().join("display.py");

        let mut file = File::create(&py_path)?;
        writeln!(file, "{}", LIB_STR)?;

        let path = dir.as_path().join("Font.ttf");
        let mut file = File::create(path)?;
        file.write_all(FONT_DATA)?;

        let mut child = Command::new("python3")
            .arg(py_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .env("PYTHONUNBUFFERED", "TRUE")
            .kill_on_drop(true)
            .spawn()
            .map_err(|e| anyhow!(e))?;

        let output = child.stdout.take().ok_or_else(|| anyhow!("No stdout"))?;
        let (tx, _rx) = channel(1000);
        let cloned_tx = tx.clone();
        let handle = tokio::spawn(async move {
            let mut lines = BufReader::new(output).lines();
            loop {
                match lines.next_line().await {
                    Ok(Some(line)) => {
                        let evt = match WrappedEvent::try_from(line.as_ref()) {
                            Ok(e) => e,
                            Err(e) => {
                                error!(error = %e, event_name = %line, "Error when parsing event");
                                continue;
                            }
                        };
                        if let Err(e) = tx.send(evt) {
                            error!(error = %e, "No receiver for event, exiting");
                            return;
                        }
                    }
                    Ok(None) => {
                        error!("Python exited");
                        return;
                    }
                    Err(e) => {
                        error!(error = %e, "Error when reading from python stdout");
                    }
                }
            }
        });

        Ok((
            Wrapper {
                stdin: child.stdin.take().ok_or_else(|| anyhow!("No stdin"))?,
                _child: child,
                _handle: handle,
            },
            cloned_tx,
        ))
    }

    pub async fn draw_message(&mut self, message: &str) -> anyhow::Result<()> {
        debug!("Sending message to python");
        self.stdin.write_all(message.as_bytes()).await?;
        self.stdin.write_all(b"\n").await?;

        Ok(())
    }
}
