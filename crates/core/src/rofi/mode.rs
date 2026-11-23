use anyhow::{bail, Context};
use log::{debug, trace};

use crate::{
    backend::{ClipEntry, ClipboardBackend},
    cache::{CacheEntry, SimpleCache},
    config,
    rofi::{self, RofiEntry},
};

use super::{KbCustom, Rofi, RofiOptions, RofiResult};

/// Current display mode
#[derive(Debug)]
enum DisplayMode {
    Text,
    Image,
}

/// Configuration for the Mode
pub struct ModeConfig {
    pub text_mode: config::ModeConfig,
    pub image_mode: config::ModeConfig,
    pub delete_mode: config::ModeConfig,
    pub delete_previous_mode: config::ModeConfig,
    pub delete_next_mode: config::ModeConfig,
    pub toggle_mode: config::ModeConfig,
}

/// A rofi "mode" to display clipboard entries from a generic backend
/// It keeps an internal state and spawns rofi to display the entries
pub struct Mode<B: ClipboardBackend> {
    rofi: Rofi,
    cache: SimpleCache,
    backend: B,
    txt: RofiState,
    img: RofiState,
    mode: DisplayMode,
}

struct RofiState {
    entries: Vec<ClipEntry>,
    options: RofiOptions,
}

impl<B: ClipboardBackend> Mode<B> {
    /// Create a new instance of Mode
    pub fn new(
        rofi: rofi::Rofi,
        cache: SimpleCache,
        backend: B,
        config: ModeConfig,
    ) -> anyhow::Result<Self> {
        trace!("Creating Mode");

        let (txt, img): (Vec<_>, Vec<_>) = backend
            .list()
            .context("Error listing clipboard entries")?
            .into_iter()
            .partition(|e| matches!(e, ClipEntry::Text { .. }));

        let delete_shortcut = &config.delete_mode.shortcut;
        let delete_description = &config.delete_mode.description;
        let instance = Self {
            txt: RofiState {
                entries: txt,
                options: RofiOptions::new(
                    Self::title(DisplayMode::Text),
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
                        KbCustom::new(
                            6,
                            &config.toggle_mode.shortcut,
                            config.toggle_mode.description.clone(),
                        ),
                    ],
                    Self::theme(DisplayMode::Text),
                    rofi.theme.clone(),
                ),
            },
            img: RofiState {
                entries: img,
                options: RofiOptions::new(
                    Self::title(DisplayMode::Image),
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
                        KbCustom::new(
                            6,
                            &config.toggle_mode.shortcut,
                            config.toggle_mode.description.clone(),
                        ),
                    ],
                    Self::theme(DisplayMode::Image),
                    rofi.theme.clone(),
                ),
            },
            rofi,
            cache,
            backend,
            mode: DisplayMode::Text,
        };

        Ok(instance)
    }

    /// This is the "main loop" of the mode
    pub fn run(&mut self) -> anyhow::Result<()> {
        debug!("Running Mode");

        loop {
            self.sync_cache()?;

            let current = match self.mode {
                DisplayMode::Text => &mut self.txt,
                DisplayMode::Image => &mut self.img,
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
                    let entry_id = entry.id();
                    self.backend
                        .activate(entry_id)
                        .context("Error activating entry")?;
                    return Ok(());
                }
                RofiResult::Keyboard { key, id } => {
                    current.options.selected_row = id;

                    match key {
                        10 => {
                            self.mode = DisplayMode::Image;
                        }
                        11 => {
                            self.mode = DisplayMode::Text;
                        }
                        12 => {
                            let entry = current.entries.remove(id);
                            self.backend.remove(entry.id())?;
                        }
                        13 => {
                            let entries_to_delete = current.entries.drain(..id).collect::<Vec<_>>();
                            for entry in entries_to_delete {
                                self.backend.remove(entry.id())?;
                            }
                        }
                        14 => {
                            let entries_to_delete =
                                current.entries.drain(id + 1..).collect::<Vec<_>>();
                            for entry in entries_to_delete {
                                self.backend.remove(entry.id())?;
                            }
                        }
                        15 => {
                            self.mode = match self.mode {
                                DisplayMode::Text => DisplayMode::Image,
                                DisplayMode::Image => DisplayMode::Text,
                            };
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

        let entries = self
            .backend
            .list()
            .context("Error listing clipboard entries")?;

        let entries = entries.iter().filter(|e| e.is_image()).collect::<Vec<_>>();

        let entries = entries.as_slice();

        for entry in entries {
            if !self.cache.exists(&CacheEntry::id(*entry)) {
                let value = self
                    .backend
                    .raw_bytes(entry.id())
                    .context("Error getting entry bytes")?;
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

    fn theme(mode: DisplayMode) -> Vec<String> {
        trace!("Switching theme to {mode:?}");

        match mode {
        DisplayMode::Text => vec![
            "element { children: [element-text]; orientation: vertical; }".into(),
            "listview { layout: vertical; }".into(),
        ],
        DisplayMode::Image => vec![
            "element { children: [element-icon]; orientation: vertical;}".into(),
            "element-icon { size: 228px; padding: 0px; }".into(),
            "listview { layout: vertical; lines: 3; columns: 3; fixed-height: true; fixed-columns: true; }".into(),
        ],
    }
    }

    fn title(mode: DisplayMode) -> String {
        match mode {
            DisplayMode::Text => "Texts".into(),
            DisplayMode::Image => "Images".into(),
        }
    }
}

/// Build the stdin bytes for rofi from entries and cache
///
/// shortcut: titles are single-line per backend contract; raw newlines would break rofi index framing
pub fn render_entries(entries: &[&dyn RofiEntry], cache: &SimpleCache) -> anyhow::Result<Vec<u8>> {
    let mut buffer = Vec::new();
    for entry in entries {
        buffer.extend_from_slice(entry.label().as_bytes());
        if let Some(icon) = entry.icon() {
            let cached = cache.path(&icon);
            if cached.exists() {
                buffer.extend_from_slice(
                    format!("\0icon\x1f{}", cached.to_str().context("resolving icon")?).as_bytes(),
                );
            } else {
                buffer.extend_from_slice(format!("\0icon\x1f{icon}").as_bytes());
            }
        }
        buffer.push(b'\n');
    }
    Ok(buffer)
}

#[cfg(test)]
mod tests {
    use crate::backend::{ClipEntry, ClipboardBackend};
    use crate::cache::{CacheEntry, SimpleCache};
    use crate::rofi::RofiEntry;

    use super::render_entries;

    #[expect(dead_code, reason = "test fixture")]
    struct FakeBackend {
        entries: Vec<ClipEntry>,
    }

    impl ClipboardBackend for FakeBackend {
        fn list(&self) -> anyhow::Result<Vec<ClipEntry>> {
            Ok(self.entries.clone())
        }
        fn raw_bytes(&self, _id: &str) -> anyhow::Result<Vec<u8>> {
            Ok(vec![])
        }
        fn remove(&self, _id: &str) -> anyhow::Result<()> {
            Ok(())
        }
        fn activate(&self, _id: &str) -> anyhow::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn feeds_icon_frame_for_images() {
        #[expect(clippy::useless_vec, reason = "need owned vec for cache operations")]
        let entries = vec![
            ClipEntry::Text {
                id: "1".to_string(),
                title: "Sample text entry".to_string(),
            },
            ClipEntry::Image {
                id: "2".to_string(),
                content_type: "png".to_string(),
            },
        ];
        let cache = SimpleCache::new("test-cache").unwrap();

        // Pre-populate cache with a dummy image file for the image entry
        let cache_key = CacheEntry::id(&entries[1]);
        cache.add(&entries[1], vec![0x89, 0x50, 0x4E, 0x47]); // minimal PNG header

        let entry_refs: Vec<&dyn RofiEntry> = entries.iter().map(|e| e as &dyn RofiEntry).collect();
        let output = render_entries(&entry_refs, &cache).unwrap();
        let output_str = String::from_utf8_lossy(&output);

        // Split by newlines to verify exact entry count
        let lines: Vec<&str> = output_str.trim_end().split('\n').collect();
        assert_eq!(lines.len(), 2, "Should have exactly 2 rofi entries");

        // First entry: text with no icon frame
        assert!(lines[0].contains("Sample text entry"));
        assert!(
            !lines[0].contains("\0icon\x1f"),
            "Text entry should not have icon frame"
        );

        // Second entry: image with icon frame pointing to cached file
        assert!(lines[1].contains("\0icon\x1f"));
        let cache_path = cache.path(&cache_key);
        assert!(cache_path.exists(), "Cache file should exist");
        let cache_path_str = cache_path.to_str().unwrap();
        assert!(
            lines[1].contains(cache_path_str),
            "Image entry should reference cached file path"
        );
    }
}
