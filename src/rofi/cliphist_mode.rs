use anyhow::{bail, Context};
use log::{debug, trace};

use crate::{
    cache::{CacheEntry, SimpleCache},
    clipboard::Clipboard,
    cliphist::{ClipHist, ClipHistEntry},
    config,
    rofi::{self, RofiEntry},
};

use super::{KbCustom, Rofi, RofiOptions, RofiResult};

/// Current mode to display
#[derive(Debug)]
enum Mode {
    Text,
    Image,
}

/// Configuration for the ClipHistMode
pub struct ClipHistModeConfig {
    pub text_mode: config::ModeConfig,
    pub image_mode: config::ModeConfig,
    pub delete_mode: config::ModeConfig,
    pub delete_previous_mode: config::ModeConfig,
    pub delete_next_mode: config::ModeConfig,
}

/// A rofi "mode" to display the clipboard history
/// It keeps an internal state and spawns rofi to display the entries
pub struct ClipHistMode {
    rofi: Rofi,
    cache: SimpleCache,
    cliphist: ClipHist,
    clipboard: Clipboard,
    txt: RofiState,
    img: RofiState,
    mode: Mode,
}

struct RofiState {
    entries: Vec<ClipHistEntry>,
    options: RofiOptions,
}

impl ClipHistMode {
    /// Create a new instance of ClipHistMode
    pub fn new(
        rofi: rofi::Rofi,
        cache: SimpleCache,
        cliphist: ClipHist,
        clipboard: Clipboard,
        config: ClipHistModeConfig,
    ) -> anyhow::Result<Self> {
        trace!("Creating ClipHistMode");

        let (txt, img): (Vec<_>, Vec<_>) = cliphist
            .list()
            .context("Error listing cliphist")?
            .into_iter()
            .partition(|e| matches!(e, ClipHistEntry::Text { .. }));

        let delete_shortcut = &config.delete_mode.shortcut;
        let delete_description = &config.delete_mode.description;
        let instance = Self {
            rofi,
            cache,
            cliphist,
            clipboard,
            txt: RofiState {
                entries: txt,
                options: RofiOptions::new(
                    Self::title(Mode::Text),
                    "",
                    [
                        KbCustom::new(1, config.image_mode.shortcut, config.image_mode.description),
                        KbCustom::new(3, delete_shortcut, delete_description),
                        KbCustom::new(
                            4,
                            &config.delete_previous_mode.shortcut,
                            config.delete_previous_mode.description.clone(),
                        ),
                        KbCustom::new(
                            5,
                            &config.delete_next_mode.shortcut,
                            config.delete_next_mode.description.clone(),
                        ),
                    ],
                    Self::theme(Mode::Text),
                ),
            },
            img: RofiState {
                entries: img,
                options: RofiOptions::new(
                    Self::title(Mode::Image),
                    "",
                    [
                        KbCustom::new(2, config.text_mode.shortcut, config.text_mode.description),
                        KbCustom::new(3, delete_shortcut, delete_description),
                        KbCustom::new(
                            4,
                            &config.delete_previous_mode.shortcut,
                            config.delete_previous_mode.description.clone(),
                        ),
                        KbCustom::new(
                            5,
                            &config.delete_next_mode.shortcut,
                            config.delete_next_mode.description.clone(),
                        ),
                    ],
                    Self::theme(Mode::Image),
                ),
            },
            mode: Mode::Text,
        };

        Ok(instance)
    }

    /// This is the "main loop" of the mode
    pub fn run(&mut self) -> anyhow::Result<()> {
        debug!("Running ClipHistMode");

        loop {
            self.sync_cache()?;

            let current = match self.mode {
                Mode::Text => &mut self.txt,
                Mode::Image => &mut self.img,
            };

            let entries = current
                .entries
                .iter()
                .map(|e| e as &dyn rofi::RofiEntry)
                .collect::<Vec<_>>();

            match self
                .rofi
                .run(&entries, &current.options, &self.cache)
                .context("running rofi")?
            {
                RofiResult::Selection { id } => {
                    current.options.selected_row = id;

                    let entry = current.entries.get(id).expect("Invalid id");
                    let cliphist_id = RofiEntry::id(entry);
                    self.clipboard.copy(
                        self.cliphist
                            .value_of(cliphist_id)
                            .context("Error getting cliphist entry")?,
                    )?;
                    return Ok(());
                }
                RofiResult::Keyboard { key, id } => {
                    current.options.selected_row = id;

                    match key {
                        10 => {
                            self.mode = Mode::Image;
                        }
                        11 => {
                            self.mode = Mode::Text;
                        }
                        12 => {
                            let entry = current.entries.remove(id);
                            self.cliphist.remove(RofiEntry::id(&entry))?;
                        }
                        13 => {
                            let entries_to_delete = current.entries.drain(..id).collect::<Vec<_>>();
                            for entry in entries_to_delete {
                                self.cliphist.remove(RofiEntry::id(&entry))?;
                            }
                        }
                        14 => {
                            let entries_to_delete =
                                current.entries.drain(id + 1..).collect::<Vec<_>>();
                            for entry in entries_to_delete {
                                self.cliphist.remove(RofiEntry::id(&entry))?;
                            }
                        }
                        _ => bail!("Unexpected key: {}", key),
                    }
                }
                RofiResult::Cancel => {
                    // just inform the user and exit
                    trace!("Cancelled");
                    return Ok(());
                }
                RofiResult::Signal { key } => {
                    // just inform the user and exit
                    trace!("Signaled: {key}");
                    return Ok(());
                }
                RofiResult::Empty => {
                    // just inform the user and exit
                    trace!("Empty response");
                    return Ok(());
                }
            }
        }
    }

    fn sync_cache(&self) -> anyhow::Result<usize> {
        trace!("Syncing cache");

        let entries = self.cliphist.list().context("Error listing cliphist")?;

        let entries = entries
            .iter()
            .filter(|e| matches!(e, ClipHistEntry::Image { .. }))
            .collect::<Vec<_>>();

        let entries = entries.as_slice();

        for entry in entries {
            if !self.cache.exists(&CacheEntry::id(*entry)) {
                let id = match entry {
                    ClipHistEntry::Text { id, .. } => id,
                    ClipHistEntry::Image { id, .. } => id,
                };
                let value = self
                    .cliphist
                    .value_of(id.into())
                    .context("Error getting cliphist entry")?;
                self.cache.add(*entry, value);
            }
        }

        let exclusions = entries
            .iter()
            .map(|e| *e as &dyn CacheEntry)
            .map(|e| e.id())
            .collect::<Vec<_>>();

        self.cache.prune(exclusions).context("Error syncing cache")
    }

    fn theme(mode: Mode) -> Vec<String> {
        trace!("Switching theme to {mode:?}");

        match mode {
        Mode::Text => vec![
            "element { children: [element-text]; orientation: vertical; }".into(),
            "listview { layout: vertical; }".into(),
        ],
        Mode::Image => vec![
            "element { children: [element-icon]; orientation: vertical;}".into(),
            "element-icon { size: 228px; padding: 0px; }".into(),
            "listview { layout: vertical; lines: 3; columns: 3; fixed-height: true; fixed-columns: true; }".into(),
        ],
    }
    }

    fn title(mode: Mode) -> String {
        match mode {
            Mode::Text => "Texts".into(),
            Mode::Image => "Images".into(),
        }
    }
}
