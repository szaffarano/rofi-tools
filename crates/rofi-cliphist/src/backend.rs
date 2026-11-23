use std::{
    io::Write,
    process::{Command, Stdio},
};

use anyhow::{bail, Context};
use log::{debug, trace};
use regex::Regex;
use roto_core::backend::{ClipEntry, ClipboardBackend};

/// Cliphist-specific implementation of ClipboardBackend
pub struct CliphistBackend {
    cliphist_bin: String,
    wl_copy_bin: String,
    line_parser: Regex,
    binary_parser: Regex,
}

impl CliphistBackend {
    pub fn new(cliphist_bin: impl Into<String>, wl_copy_bin: impl Into<String>) -> Self {
        trace!("Creating CliphistBackend");
        Self {
            cliphist_bin: cliphist_bin.into(),
            wl_copy_bin: wl_copy_bin.into(),
            line_parser: Regex::new(r"^(?P<idx>[0-9]+)\t(?P<value>.*)$")
                .expect("invalid line parser regex"),
            binary_parser: Regex::new(r"^(\[\[\s)?binary.*(?P<ext>jpg|jpeg|png|bmp)")
                .expect("invalid binary parser regex"),
        }
    }

    fn parse_line(&self, line: &str) -> anyhow::Result<ClipEntry> {
        let parsed = self
            .line_parser
            .captures(line)
            .context(format!("Invalid cliphist entry: '{line}'"))?;

        let id = String::from(&parsed["idx"]);
        let value = &parsed["value"];

        let entry: ClipEntry = if let Some(binary) = self.binary_parser.captures(value) {
            ClipEntry::Image {
                id,
                content_type: binary["ext"].into(),
            }
        } else {
            ClipEntry::Text {
                id,
                title: value.into(),
            }
        };

        Ok(entry)
    }

    fn parse_list(&self, output: &str) -> anyhow::Result<Vec<ClipEntry>> {
        output
            .split('\n')
            .filter(|line| !line.is_empty())
            .map(|line| self.parse_line(line))
            .collect()
    }
}

impl ClipboardBackend for CliphistBackend {
    fn list(&self) -> anyhow::Result<Vec<ClipEntry>> {
        trace!("Listing clipboard history");
        let history = Command::new(&self.cliphist_bin)
            .arg("list")
            .output()
            .context("Error executing cliphist")?;

        if !history.status.success() {
            bail!(
                "Error executing cliphist: {}",
                String::from_utf8_lossy(&history.stderr)
            );
        }

        let entries = self
            .parse_list(&String::from_utf8_lossy(&history.stdout))
            .context("Error parsing cliphist output")?;

        debug!("Found {} entries in clipboard history", entries.len());

        Ok(entries)
    }

    fn raw_bytes(&self, id: &str) -> anyhow::Result<Vec<u8>> {
        trace!("Getting raw bytes for entry with id: {id}");

        let value = Command::new(&self.cliphist_bin)
            .arg("decode")
            .arg(format!("{id}\t\n"))
            .output()
            .context("Error executing cliphist decode")?;

        if !value.status.success() {
            bail!(
                "Error executing cliphist decode: {}",
                String::from_utf8_lossy(&value.stderr)
            );
        }

        debug!(
            "Got raw bytes for entry with id: {} ({} bytes)",
            id,
            value.stdout.len()
        );

        Ok(value.stdout)
    }

    fn remove(&self, id: &str) -> anyhow::Result<()> {
        debug!("About to remove entry with id: {id}");
        let mut child = Command::new(&self.cliphist_bin)
            .arg("delete")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .context("Error executing cliphist delete")?;

        child
            .stdin
            .as_mut()
            .context("Failed to open stdin")?
            .write_all(format!("{id}\t").as_bytes())
            .context("Failed to write to stdin")?;

        let status = child.wait().context("Error executing cliphist delete")?;

        if !status.success() {
            bail!("Error executing cliphist delete");
        }

        debug!("Successfully removed entry with id: {id}");

        Ok(())
    }

    fn activate(&self, id: &str) -> anyhow::Result<()> {
        trace!("Activating entry with id: {id}");

        // Get the raw bytes from cliphist
        let content = self.raw_bytes(id)?;

        // Copy to clipboard using wl-copy
        let mut child = Command::new(&self.wl_copy_bin)
            .stdin(Stdio::piped())
            .spawn()
            .context("Error executing wl-copy")?;

        child
            .stdin
            .as_mut()
            .context("Failed to open stdin")?
            .write_all(&content)
            .context("Failed to write to stdin")?;

        let status = child.wait().context("Error executing wl-copy")?;
        if !status.success() {
            bail!("Error executing wl-copy");
        }

        debug!("Successfully activated entry with id: {id}");

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_list_and_classifies() {
        let backend = CliphistBackend::new("cliphist", "wl-copy");

        let text_line = "123\tHello World";
        let result = backend.parse_line(text_line).unwrap();
        match result {
            ClipEntry::Text { id, title } => {
                assert_eq!(id, "123");
                assert_eq!(title, "Hello World");
            }
            _ => panic!("Expected Text entry"),
        }

        let binary_line = "456\t[[ binary data 1234 png ]]";
        let result = backend.parse_line(binary_line).unwrap();
        match result {
            ClipEntry::Image { id, content_type } => {
                assert_eq!(id, "456");
                assert_eq!(content_type, "png");
            }
            _ => panic!("Expected Image entry"),
        }
    }

    #[test]
    fn empty_list_is_ok() {
        let backend = CliphistBackend::new("cliphist", "wl-copy");
        let empty_input = "";
        let result = backend.parse_list(empty_input).unwrap();
        assert_eq!(result.len(), 0);
    }
}
