use std::env;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use anyhow::anyhow;
use tokio::process::Command;
use tracing::debug;

const LIB_STR: &str = include_str!("display.py");

pub fn setup() -> anyhow::Result<PathBuf> {
    let dir = env::temp_dir();
    let path = dir.as_path().join("display.py");

    let mut file = File::create(&path)?;
    writeln!(file, "{}", LIB_STR)?;

    Ok(path)
}

pub async fn draw_message(lib_path: &str, message: &str) -> anyhow::Result<()> {
    let output = Command::new("python3")
        .arg(lib_path)
        .arg(message)
        .output()
        .await
        .map_err(|e| anyhow!(e))?;

    debug!(
        "Status: {}\nStdout: {:?}\nStderr: {:?}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    Ok(())
}
