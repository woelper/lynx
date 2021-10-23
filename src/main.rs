#![windows_subsystem = "windows"]
mod app;
#[cfg(target_os = "macos")]
mod mac;
pub mod sound;
pub mod theme;
pub mod ui_components;
use log::{info, LevelFilter};
use std::path::PathBuf;
use structopt::StructOpt;
extern crate static_vcruntime;

#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
struct Opt {
    /// Files to process
    #[structopt(name = "files", parse(from_os_str))]
    files: Vec<PathBuf>,
    /// Bypass mac "open with" behaviour
    #[structopt(short = "c")]
    chainload: bool,
}

fn main() {
    env_logger::builder().filter_level(LevelFilter::Info).init();
    info!("Starting up");

    let args = Opt::from_args();
    info!("Startup args {:?}", args);

    #[cfg(target_os = "macos")]
    if !args.chainload && args.files.is_empty() {
        info!("Chainload not specified, and no input file present. Invoking mac hack.");
        // MacOS needs an incredible dance performed just to open a file
        let _ = mac::launch();
    }

    let app = app::ApplicationState::default();
    let options = eframe::NativeOptions {
        drag_and_drop_support: true,
        ..Default::default()
    };
    eframe::run_native(Box::new(app), options);
}
