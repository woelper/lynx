mod app;
use app::MetaSound;
use eframe::epi;
use kira::{
    instance::InstanceSettings,
    manager::{AudioManager, AudioManagerSettings},
    sound::SoundSettings,
};
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
    let app = app::ApplicationState::default();
    let opts = epi::NativeOptions::default();
    eframe::run_native(Box::new(app), opts);
}
