use crate::{
    cache::{self, CacheEntry},
    cliphist::{ClipHist, ClipHistEntry},
    rofi,
};

use super::KbCustom;

pub struct RofiMode<'a> {
    pub rofi: rofi::Rofi,
    pub entries: Vec<&'a dyn rofi::RofiEntry>,
    pub options: rofi::RofiOptions,
    pub cache: &'a cache::SimpleCache,
    pub cliphist: &'a ClipHist,
}

pub enum Mode {
    Text,
    Image,
}

impl RofiMode<'_> {
    pub fn run(&self) -> rofi::RofiResult {
        self.rofi.run(&self.entries, &self.options, self.cache)
    }

    pub fn sync_cache(&self) -> usize {
        let entries = self.cliphist.list();

        let entries = entries
            .iter()
            .filter(|e| matches!(e, ClipHistEntry::Image { .. }))
            .collect::<Vec<_>>();

        let entries = entries.as_slice();

        for entry in entries {
            if !self.cache.exists(&entry.id()) {
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
}

pub fn mode_help(img_enabled: bool, txt_enabled: bool) -> String {
    let mut mesg = String::from("<span size='small' alpha='70%'>");

    if img_enabled {
        mesg.push_str("<b>alt+i</b>: switch to images | ");
    }

    if txt_enabled {
        mesg.push_str("<b>alt+t</b>: switch to texts | ");
    }

    mesg.push_str("<b>alt+d</b>: delete entry");
    mesg.push_str("</span>");

    mesg
}

pub fn mode_keybindings(img_enabled: bool, txt_enabled: bool) -> Vec<KbCustom> {
    let mut kbs = Vec::new();
    if img_enabled {
        kbs.push(KbCustom::new(1, "Alt+i"));
    }
    if txt_enabled {
        kbs.push(KbCustom::new(2, "Alt+t"));
    }
    kbs.push(KbCustom::new(3, "Alt+d"));
    kbs
}

pub fn mode_theme(mode: Mode) -> Vec<String> {
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

pub fn mode_title(mode: Mode) -> String {
    match mode {
        Mode::Text => "Texts".into(),
        Mode::Image => "Images".into(),
    }
}
