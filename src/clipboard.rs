use std::io::Write;

use anyhow::{bail, Context};
use log::trace;

pub struct Clipboard {
    bin: String,
}

pub fn new(bin: impl Into<String>) -> Clipboard {
    Clipboard { bin: bin.into() }
}

impl Clipboard {
    pub fn copy(&self, content: Vec<u8>) -> anyhow::Result<()> {
        trace!("Copying to clipboard");

        let mut child = std::process::Command::new(&self.bin)
            .stdin(std::process::Stdio::piped())
            .spawn()
            .context("Error executing clipboard")?;

        child
            .stdin
            .as_mut()
            .context("Failed to open stdin")?
            .write_all(&content)
            .context("Failed to write to stdin")?;

        let status = child.wait().context("Error executing clipboard")?;
        if !status.success() {
            bail!("Error executing clipboard");
        }
        Ok(())
    }

    pub fn paste(&self) -> anyhow::Result<()> {
        trace!("Pasting from clipboard");

        let mut child = std::process::Command::new("wtype")
            .args(&["-M", "ctrl", "-k", "v", "-m", "ctrl"])
            .spawn()
            .context("Error executing wtype")?;

        let status = child.wait().context("Error executing wtype")?;
        if !status.success() {
            bail!("Error executing wtype");
        }
        Ok(())
    }
}
