mod app;
pub mod sound;
pub mod theme;
pub mod ui_components;
#[cfg(target_os = "macos")]
mod mac;
use eframe::epi;
use log::{info, LevelFilter};
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
struct Opt {
    /// Files to process
    #[structopt(name = "files", parse(from_os_str))]
    files: Vec<PathBuf>,
    /// Bypass mac "open with" behaviour
    #[structopt(short = "c")]
    chainload: bool
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
    let opts = epi::NativeOptions::default();
    eframe::run_native(Box::new(app), opts);
}
