mod app;
pub mod sound;
pub mod theme;
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
}

fn main() {
    env_logger::builder().filter_level(LevelFilter::Info).init();
    info!("Starting up");
    let app = app::ApplicationState::default();
    let opts = epi::NativeOptions::default();
    eframe::run_native(Box::new(app), opts);
}
