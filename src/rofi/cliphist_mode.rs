use crate::{
    cache::{CacheEntry, SimpleCache},
    clipboard::Clipboard,
    cliphist::{ClipHist, ClipHistEntry},
    rofi::{self, RofiEntry},
};

use super::{KbCustom, Rofi, RofiOptions, RofiResult};

enum Mode {
    Text,
    Image,
}

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
    pub fn new(
        rofi: rofi::Rofi,
        cache: SimpleCache,
        cliphist: ClipHist,
        clipboard: Clipboard,
    ) -> Self {
        let (txt, img): (Vec<_>, Vec<_>) = cliphist
            .list()
            .into_iter()
            .partition(|e| matches!(e, ClipHistEntry::Text { .. }));

        Self {
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
                        KbCustom::new(1, "Alt+i", "Switch to images"),
                        KbCustom::new(3, "Alt+d", "Delete entry"),
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
                        KbCustom::new(2, "Alt+t", "Switch to texts"),
                        KbCustom::new(3, "Alt+d", "Delete entry"),
                    ],
                    Self::theme(Mode::Image),
                ),
            },
            mode: Mode::Text,
        }
    }

    pub fn run(&mut self) {
        loop {
            self.sync_cache();

            let current = match self.mode {
                Mode::Text => &mut self.txt,
                Mode::Image => &mut self.img,
            };

            let entries = current
                .entries
                .iter()
                .map(|e| e as &dyn rofi::RofiEntry)
                .collect::<Vec<_>>();

            match self.rofi.run(&entries, &current.options, &self.cache) {
                RofiResult::Selection { id } => {
                    current.options.selected_row = id;

                    let entry = current.entries.get(id).expect("Invalid id");
                    let cliphist_id = RofiEntry::id(entry);
                    self.clipboard.copy(self.cliphist.value_of(cliphist_id));
                    break;
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
                            self.cliphist.remove(RofiEntry::id(&entry));
                        }
                        _ => panic!("Unexpected key: {}", key),
                    }
                }
                RofiResult::Cancel => {
                    // just inform the user and exit
                    println!("Cancelled");
                    break;
                }
                RofiResult::Signal { key } => {
                    // just inform the user and exit
                    println!("Signaled: {key}");
                    break;
                }
                RofiResult::Empty => {
                    // just inform the user and exit
                    println!("Empty response");
                    break;
                }
            }
        }
    }

    fn sync_cache(&self) -> usize {
        let entries = self.cliphist.list();

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
                let value = self.cliphist.value_of(id.into());
                self.cache.add(*entry, value);
            }
        }

        let exclusions = entries
            .iter()
            .map(|e| (*e as &dyn CacheEntry))
            .map(|e| e.id())
            .collect::<Vec<_>>();

        self.cache.prune(exclusions).expect("Error syncing cache")
    }

    fn theme(mode: Mode) -> Vec<String> {
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
