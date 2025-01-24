use cliphist::ClipHistEntry;
use rofi::cliphist_mode::{Mode, RofiMode};
use rofi::RofiResult;
use rofi_cliphist::rofi::cliphist_mode;
use rofi_cliphist::{cache, clipboard, cliphist, rofi};

fn main() {
    let cliphist = cliphist::new("cliphist");
    let cache = cache::SimpleCache::new("rofi-cliphist/thumbs-new").expect("Error creating cache");
    let clipboard = clipboard::new("wl-copy");

    let (txt_entries, img_entries): (Vec<_>, Vec<_>) = cliphist
        .list()
        .into_iter()
        .partition(|e| matches!(e, ClipHistEntry::Text { .. }));

    let mut txt = RofiMode {
        rofi: rofi::new("rofi"),
        cache: &cache,
        entries: txt_entries
            .iter()
            .map(|e| e as &dyn rofi::RofiEntry)
            .collect::<Vec<_>>(),
        options: rofi::RofiOptions::new(
            cliphist_mode::mode_title(Mode::Text),
            cliphist_mode::mode_help(!img_entries.is_empty(), !txt_entries.is_empty()),
            cliphist_mode::mode_keybindings(!img_entries.is_empty(), !txt_entries.is_empty()),
            cliphist_mode::mode_theme(Mode::Text),
        ),
        cliphist: &cliphist,
    };

    let mut img = RofiMode {
        rofi: rofi::new("rofi"),
        cache: &cache,
        entries: img_entries
            .iter()
            .map(|e| e as &dyn rofi::RofiEntry)
            .collect::<Vec<_>>(),
        options: rofi::RofiOptions::new(
            cliphist_mode::mode_title(Mode::Image),
            cliphist_mode::mode_help(!img_entries.is_empty(), !txt_entries.is_empty()),
            cliphist_mode::mode_keybindings(!img_entries.is_empty(), !txt_entries.is_empty()),
            cliphist_mode::mode_theme(Mode::Image),
        ),
        cliphist: &cliphist,
    };

    let mut mode = Mode::Text;
    loop {
        let img_enabled = !img.entries.is_empty();
        let txt_enabled = !txt.entries.is_empty();

        if !img_enabled && !txt_enabled {
            println!("No clipboard entries to show");
            break;
        }

        // Only two modes, if one of them is disabled, switch to the other
        if !img_enabled {
            mode = Mode::Text;
        } else if !txt_enabled {
            mode = Mode::Image;
        }

        // update help and keybindings according to the current state
        for c in [&mut img, &mut txt].iter_mut() {
            c.options.mesg = Some(cliphist_mode::mode_help(img_enabled, txt_enabled));
            c.options.custom_kbs = cliphist_mode::mode_keybindings(img_enabled, txt_enabled);
        }

        // set the current mode
        let current = match mode {
            Mode::Text => &mut txt,
            Mode::Image => &mut img,
        };

        // ensure the cache is up to date
        current.sync_cache();

        // launch rofi
        match current.run() {
            RofiResult::Selection { id } => {
                current.options.selected_row = id;

                let id = current.entries.get(id).expect("Invalid id").id();
                clipboard.copy(cliphist.value_of(id));
                break;
            }
            RofiResult::Keyboard { key, id } => {
                current.options.selected_row = id;

                match key {
                    10 => {
                        mode = Mode::Image;
                    }
                    11 => {
                        mode = Mode::Text;
                    }
                    12 => {
                        let cliphist_id = current.entries.get(id).expect("Invalid id").id();
                        cliphist.remove(cliphist_id.to_string());
                        current.entries.remove(id);
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
                println!("Empty");
                break;
            }
        };
    }
}
