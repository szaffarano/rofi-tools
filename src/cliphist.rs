use std::{
    io::Write,
    process::{Command, Stdio},
};

use regex::Regex;

use crate::cache::CacheEntry;

pub enum ClipHistEntry {
    Text { id: String, title: String },
    Image { id: String, content_type: String },
}

pub struct ClipHist {
    bin: String,
    line_parser: Regex,
    binary_parser: Regex,
}

pub fn new(bin: impl Into<String>) -> ClipHist {
    ClipHist {
        bin: bin.into(),
        line_parser: Regex::new(r"^(?P<idx>[0-9]+)\t(?P<value>.*)$").unwrap(),
        binary_parser: Regex::new(r"^(\[\[\s)?binary.*(?P<ext>jpg|jpeg|png|bmp)").unwrap(),
    }
}

impl ClipHist {
    pub fn list(&self) -> Vec<ClipHistEntry> {
        let history = Command::new(&self.bin)
            .arg("list")
            .output()
            .expect("Error executing cliphist");

        if !history.status.success() {
            panic!(
                "Error executing cliphist: {}",
                String::from_utf8_lossy(&history.stderr)
            );
        }

        String::from_utf8_lossy(&history.stdout)
            .split('\n')
            .filter(|line| !line.is_empty())
            .map(|line| self.parse_entry(line))
            .collect::<Vec<_>>()
            .into_iter()
            .collect::<Vec<ClipHistEntry>>()
    }

    pub fn remove(&self, id: String) {
        let mut child = Command::new(&self.bin)
            .arg("delete")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("Error executing cliphist");

        child
            .stdin
            .as_mut()
            .expect("Failed to open stdin")
            .write_all(format!("{id}\t").as_bytes())
            .unwrap();

        let status = child.wait().expect("Error executing cliphist");

        if !status.success() {
            panic!("Error executing cliphist");
        }
    }

    pub fn value_of(&self, id: String) -> Vec<u8> {
        let img = Command::new(&self.bin)
            .arg("decode")
            .arg(format!("{id}\t\n"))
            .output()
            .expect("Error executing cliphist");

        if !img.status.success() {
            panic!(
                "Error executing cliphist: {}",
                String::from_utf8_lossy(&img.stderr)
            );
        }

        img.stdout
    }

    fn parse_entry(&self, line: &str) -> ClipHistEntry {
        let parsed = self
            .line_parser
            .captures(line)
            .unwrap_or_else(|| panic!("Invalid cliphist entry: '{line}'"));

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

        entry
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
