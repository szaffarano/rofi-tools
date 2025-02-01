use std::path::PathBuf;

use clap::Parser;
use roto::{
    cache, clipboard, cliphist, config,
    rofi::{self, cliphist_mode::ClipHistMode},
};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Show verbose output
    #[clap(short, long)]
    verbose: bool,

    /// Path to rofi executable
    #[clap(short, long, default_value = "rofi")]
    rofi_path: Option<String>,

    /// Path to cliphist executable
    #[clap(short, long, default_value = "cliphist")]
    cliphist_path: Option<String>,

    /// Path to wl-copy executable
    #[clap(short = 'w', long, default_value = "wl-copy")]
    clipboard_path: Option<String>,

    /// Sets a custom config file
    #[arg(short = 'f', long, value_name = "FILE")]
    config: Option<PathBuf>,
}

fn main() {
    let args = Args::parse();

    let mut cfg = if let Some(config_path) = &args.config {
        config::load(config_path).expect("Error loading config file")
    } else {
        match config::load_default() {
            Ok(c) => c,
            Err(e) => {
                if let Some(io_err) = e.downcast_ref::<std::io::Error>() {
                    if io_err.kind() == std::io::ErrorKind::NotFound {
                        println!("Config file not found, using default config");
                        config::Config::default()
                    } else {
                        panic!("Error reading config: {:?}", e);
                    }
                } else {
                    panic!("Unexpected error reading config: {:?}", e);
                }
            }
        }
    };

    merge_args_into_config(&mut cfg, args);

    let cliphist = cliphist::new(cfg.cliphist.path);
    let cache = cache::SimpleCache::new("rofi-cliphist/thumbs-new").expect("Error creating cache");
    let clipboard = clipboard::new(cfg.clipboard.path);
    let rofi = rofi::new(cfg.rofi.path);

    ClipHistMode::new(
        rofi,
        cache,
        cliphist,
        clipboard,
        cfg.text_mode_config,
        cfg.image_mode_config,
        cfg.delete_mode_config,
    )
    .run();
}

fn merge_args_into_config(cfg: &mut config::Config, args: Args) {
    cfg.rofi.path = args.rofi_path.unwrap_or(cfg.rofi.path.clone());
    cfg.clipboard.path = args.clipboard_path.unwrap_or(cfg.clipboard.path.clone());
    cfg.cliphist.path = args.cliphist_path.unwrap_or(cfg.cliphist.path.clone());
}
