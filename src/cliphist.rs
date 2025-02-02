use std::{
    io::Write,
    process::{Command, Stdio},
};

use anyhow::{bail, Context};
use log::{debug, trace};
use regex::Regex;

use crate::cache::CacheEntry;

/// A given entry in the clipboard history.
pub enum ClipHistEntry {
    Text { id: String, title: String },
    Image { id: String, content_type: String },
}

/// Api for interacting with the `cliphist` clipboard manager.
pub struct ClipHist {
    bin: String,
    line_parser: Regex,
    binary_parser: Regex,
}

/// Create a new instance of the `ClipHist` api.
pub fn new(bin: impl Into<String>) -> ClipHist {
    trace!("Creating new ClipHist instance");
    ClipHist {
        bin: bin.into(),
        line_parser: Regex::new(r"^(?P<idx>[0-9]+)\t(?P<value>.*)$").unwrap(),
        binary_parser: Regex::new(r"^(\[\[\s)?binary.*(?P<ext>jpg|jpeg|png|bmp)").unwrap(),
    }
}

impl ClipHist {
    /// List all entries in the clipboard history.
    pub fn list(&self) -> anyhow::Result<Vec<ClipHistEntry>> {
        trace!("Listing clipboard history");
        let history = Command::new(&self.bin)
            .arg("list")
            .output()
            .context("Error executing cliphist")?;

        if !history.status.success() {
            bail!(
                "Error executing cliphist: {}",
                String::from_utf8_lossy(&history.stderr)
            );
        }

        let history: anyhow::Result<Vec<ClipHistEntry>> = String::from_utf8_lossy(&history.stdout)
            .split('\n')
            .filter(|line| !line.is_empty())
            .map(|line| self.parse_entry(line))
            .collect::<Vec<_>>()
            .into_iter()
            .collect();

        let history = history.context("Error parsing cliphist output")?;

        debug!("Found {} entries in clipboard history", history.len());

        Ok(history)
    }

    /// Remove an entry from the clipboard history.
    pub fn remove(&self, id: String) -> anyhow::Result<()> {
        debug!("About to remove entry with id: {}", id);
        let mut child = Command::new(&self.bin)
            .arg("delete")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .context("Error executing cliphist")?;

        child
            .stdin
            .as_mut()
            .context("Failed to open stdin")?
            .write_all(format!("{id}\t").as_bytes())
            .context("Failed to write to stdin")?;

        let status = child.wait().context("Error executing cliphist")?;

        if !status.success() {
            bail!("Error executing cliphist");
        }

        debug!("Successfully removed entry with id: {}", id);

        Ok(())
    }

    /// Get the value of a given entry in the clipboard history.
    pub fn value_of(&self, id: String) -> anyhow::Result<Vec<u8>> {
        trace!("Getting value of entry with id: {}", id);

        let value = Command::new(&self.bin)
            .arg("decode")
            .arg(format!("{id}\t\n"))
            .output()
            .context("Error executing cliphist")?;

        if !value.status.success() {
            bail!(
                "Error executing cliphist: {}",
                String::from_utf8_lossy(&value.stderr)
            );
        }

        debug!(
            "Got value of entry with id: {} ({}) bytes",
            id,
            value.stdout.len()
        );

        Ok(value.stdout)
    }

    fn parse_entry(&self, line: &str) -> anyhow::Result<ClipHistEntry> {
        let parsed = self
            .line_parser
            .captures(line)
            .context(format!("Invalid cliphist entry: '{line}'"))?;

        let id = String::from(&parsed["idx"]);
        let value = &parsed["value"];

        let entry: ClipHistEntry = if let Some(binary) = self.binary_parser.captures(value) {
            ClipHistEntry::Image {
                id,
                content_type: binary["ext"].into(),
            }
        } else {
            ClipHistEntry::Text {
                id,
                title: value.into(),
            }
        };

        Ok(entry)
    }
}

impl CacheEntry for ClipHistEntry {
    fn id(&self) -> String {
        match self {
            ClipHistEntry::Text { id, .. } => id.to_string(),
            ClipHistEntry::Image { id, content_type } => format!("{}.{}", id, content_type),
        }
    }
}
