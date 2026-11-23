use std::path::PathBuf;

mod backend;
mod config;

use anyhow::bail;
use backend::ClipcatBackend;
use clap::Parser;
use log::{debug, info, Level};
use roto_core::{
    cache,
    rofi::{self, mode::Mode, mode::ModeConfig},
};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Show verbose output
    #[command(flatten)]
    verbose: clap_verbosity_flag::Verbosity,

    /// Path to rofi executable
    #[clap(short, long, default_value = "rofi")]
    rofi_path: Option<String>,

    /// Clipcat gRPC endpoint
    #[clap(short, long)]
    endpoint: Option<String>,

    /// Clipcat access token
    #[clap(short, long)]
    access_token: Option<String>,

    /// Sets a custom config file
    #[arg(short = 'f', long, value_name = "FILE")]
    config: Option<PathBuf>,

    /// Sets a custom theme
    #[arg(short = 't', long, value_name = "THEME")]
    theme: Option<PathBuf>,
}

fn main() -> anyhow::Result<()> {
    let mut args = Args::parse();
    let theme = args.theme.take();

    simple_logger::init_with_level(args.verbose.log_level().unwrap_or(Level::Error))?;

    let mut cfg = if let Some(config_path) = &args.config {
        info!("Using custom config file: {config_path:?}");
        config::load(config_path).expect("Error loading config file")
    } else {
        match config::load_default() {
            Ok(c) => c,
            Err(e) => {
                if let Some(io_err) = e.downcast_ref::<std::io::Error>() {
                    if io_err.kind() == std::io::ErrorKind::NotFound {
                        debug!("Config file not found, using default config");
                        config::Config::default()
                    } else {
                        bail!("Error reading config: {:?}", e);
                    }
                } else {
                    bail!("Unexpected error reading config: {:?}", e);
                }
            }
        }
    };

    merge_args_into_config(&mut cfg, args);

    let endpoint: http::Uri = cfg
        .clipcat
        .endpoint
        .parse()
        .map_err(|e| anyhow::anyhow!("Invalid clipcat endpoint: {}", e))?;

    let backend = ClipcatBackend::new(endpoint, cfg.clipcat.access_token)?;
    let cache = cache::SimpleCache::new("rofi-clipcat/thumbs").expect("Error creating cache");
    let rofi = rofi::new(cfg.rofi.path, theme);

    debug!("Starting Mode");

    Mode::new(
        rofi,
        cache,
        backend,
        ModeConfig {
            text_mode: cfg.text_mode_config,
            image_mode: cfg.image_mode_config,
            delete_mode: cfg.delete_mode_config,
            delete_previous_mode: cfg.delete_previous_config,
            delete_next_mode: cfg.delete_next_config,
            toggle_mode: cfg.toggle_mode_config,
        },
    )?
    .run()
}

fn merge_args_into_config(cfg: &mut config::Config, args: Args) {
    if let Some(rofi_path) = args.rofi_path {
        cfg.rofi.path = rofi_path;
    }
    if let Some(endpoint) = args.endpoint {
        cfg.clipcat.endpoint = endpoint;
    }
    if let Some(access_token) = args.access_token {
        cfg.clipcat.access_token = Some(access_token);
    }
}
