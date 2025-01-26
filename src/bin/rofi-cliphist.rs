use rofi_cliphist::{
    cache, clipboard, cliphist,
    rofi::{self, cliphist_mode::ClipHistMode},
};

fn main() {
    let cliphist = cliphist::new("cliphist");
    let cache = cache::SimpleCache::new("rofi-cliphist/thumbs-new").expect("Error creating cache");
    let clipboard = clipboard::new("wl-copy");
    let rofi = rofi::new("rofi");

    ClipHistMode::new(rofi, cache, cliphist, clipboard).run();
}
