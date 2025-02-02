use std::{
    io::{Read, Write},
    os::unix::process::ExitStatusExt,
    process::{Command, Stdio},
};

use anyhow::Context;
use log::trace;

use crate::{cache, cliphist::ClipHistEntry};

pub mod cliphist_mode;

/// Entry to be displayed in rofi
///
/// Id is a unique identifier for the entry when selected.
/// Label is the text to be displayed.
/// Icon is an optional icon to be displayed, either a path to a filesystem image o a supported
/// pango icon name.
pub trait RofiEntry {
    fn id(&self) -> String;
    fn label(&self) -> String;
    fn icon(&self) -> Option<String>;
}

/// Possible result of a rofi execution
pub enum RofiResult {
    Cancel,
    Empty,
    Keyboard { key: i32, id: usize },
    Selection { id: usize },
    Signal { key: i32 }, // just to capture OS signals, not sure whether it's useful
}

/// API to interact with rofi using command execution.
pub struct Rofi {
    pub bin: String,
}

/// Options to configure rofi when spawning it.
pub struct RofiOptions {
    format: Option<String>, // not expose format to foce using "i"
    pub case_insensitive: bool,
    pub custom_kbs: Vec<KbCustom>,
    pub dmenu: bool,
    pub mesg: Option<String>,
    pub no_custom: bool,
    pub prompt: Option<String>,
    pub selected_row: usize,
    pub theme_str: Vec<String>,
}

/// Custom keyboard shortcuts for rofi.
pub struct KbCustom {
    key: i32,
    shortcut: String,
    description: String,
}

impl Default for RofiOptions {
    fn default() -> Self {
        trace!("Creating default RofiOptions");
        RofiOptions {
            case_insensitive: true,
            custom_kbs: vec![],
            dmenu: true,
            format: Some("i".into()), // force return index instead of value
            mesg: None,
            no_custom: true,
            prompt: None,
            selected_row: 0,
            theme_str: vec![],
        }
    }
}

impl RofiOptions {
    pub fn new<I, K, S>(
        prompt: impl Into<String>,
        mesg: impl Into<String>,
        custom_kbs: K,
        theme_str: I,
    ) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
        K: IntoIterator<Item = KbCustom>,
    {
        trace!("Creating RofiOptions");
        RofiOptions {
            mesg: Some(mesg.into()),
            prompt: Some(prompt.into()),
            custom_kbs: custom_kbs.into_iter().collect::<Vec<_>>(),
            theme_str: theme_str.into_iter().map(|s| s.into()).collect::<Vec<_>>(),
            ..Default::default()
        }
    }
}

impl KbCustom {
    pub fn new(key: i32, shortcut: impl Into<String>, description: impl Into<String>) -> Self {
        trace!("Creating KbCustom");
        KbCustom {
            key,
            shortcut: shortcut.into(),
            description: description.into(),
        }
    }
}

impl From<&RofiOptions> for Vec<String> {
    fn from(val: &RofiOptions) -> Self {
        let mut options = Vec::new();
        options.push("-selected-row".into());
        options.push(val.selected_row.to_string());
        if val.dmenu {
            options.push("-dmenu".into());
        }
        if val.case_insensitive {
            options.push("-i".into());
        }
        if val.no_custom {
            options.push("-no-custom".into());
        }
        if let Some(format) = &val.format {
            options.push("-format".into());
            options.push(format.into());
        }
        for KbCustom { key, shortcut, .. } in &val.custom_kbs {
            options.push(format!("-kb-custom-{}", key));
            options.push(shortcut.into());
        }
        if !&val.custom_kbs.is_empty() {
            let mut mesg = String::from("<span size='small' alpha='70%'>");
            for KbCustom {
                shortcut,
                description,
                ..
            } in &val.custom_kbs
            {
                mesg.push_str(format!("<b>{shortcut}</b>: {description} | ").as_str());
            }
            mesg.push_str("</span>");
            options.push("-mesg".into());
            options.push(mesg);
        }
        if let Some(prompt) = &val.prompt {
            options.push("-p".into());
            options.push(prompt.into());
        }
        for theme_str in &val.theme_str {
            options.push("-theme-str".into());
            options.push(theme_str.into());
        }
        options
    }
}

impl Rofi {
    pub fn run(
        &self,
        entries: &[&dyn RofiEntry],
        options: &RofiOptions,
        cache: &cache::SimpleCache,
    ) -> anyhow::Result<RofiResult> {
        let options = if entries.is_empty() {
            let base_msg = "No clipboard entries to show".into();
            let error_msg = options
                .prompt
                .as_ref()
                .map(|p| format!("<b>{p}</b>: {base_msg}"))
                .unwrap_or_else(|| base_msg);
            vec!["-e".into(), error_msg, "-markup".into()]
        } else {
            options.into()
        };

        let mut process = Command::new(&self.bin)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .args(options)
            .spawn()
            .context("Error executing rofi")?;

        if let Some(mut writer) = process.stdin.take() {
            for entry in entries {
                let mut str: Vec<u8> = Vec::new();
                str.extend_from_slice(entry.label().to_string().as_bytes());
                if let Some(icon) = &entry.icon() {
                    let cached = cache.path(icon);
                    if cached.exists() {
                        str.extend_from_slice(
                            format!("\0icon\x1f{}", cached.to_str().context("resolving icon")?)
                                .as_bytes(),
                        );
                    } else {
                        str.extend_from_slice(format!("\0icon\x1f{}", icon).as_bytes());
                    }
                }
                str.push(b'\n');
                writer
                    .write_all(&str)
                    .context("writing entries to rofi through stdin")?;
            }
        }

        let status = process.wait().context("waiting rofi's execution")?;

        let mut buffer = String::new();
        if let Some(mut reader) = process.stdout.take() {
            reader
                .read_to_string(&mut buffer)
                .context("reading rofi's output")?;
        }
        if buffer.ends_with('\n') {
            buffer.pop();
        }

        let result = if status.success() {
            if buffer.is_empty() {
                RofiResult::Empty
            } else {
                RofiResult::Selection {
                    id: buffer.parse::<usize>().context("parsing usize")?,
                }
            }
        } else if let Some(code) = status.code() {
            if buffer.is_empty() {
                RofiResult::Cancel
            } else {
                RofiResult::Keyboard {
                    key: code,
                    id: buffer.parse::<usize>().context("parsing usize")?,
                }
            }
        } else if let Some(code) = status.signal() {
            RofiResult::Signal { key: code }
        } else if let Some(code) = status.stopped_signal() {
            RofiResult::Signal { key: code }
        } else {
            panic!("Rofi exited unexpectedly");
        };

        Ok(result)
    }
}

impl RofiEntry for ClipHistEntry {
    fn id(&self) -> String {
        match self {
            ClipHistEntry::Text { id, .. } => id.into(),
            ClipHistEntry::Image { id, .. } => id.into(),
        }
    }
    fn icon(&self) -> Option<String> {
        match self {
            ClipHistEntry::Text { .. } => None,
            ClipHistEntry::Image { id, content_type } => Some(format!("{}.{}", id, content_type)),
        }
    }
    fn label(&self) -> String {
        match self {
            ClipHistEntry::Text { title, .. } => title.into(),
            ClipHistEntry::Image { id, content_type } => {
                format!("{}.{}", id, content_type)
            }
        }
    }
}

pub fn new(bin: impl Into<String>) -> Rofi {
    Rofi { bin: bin.into() }
}
