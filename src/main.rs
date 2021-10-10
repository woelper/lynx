mod app;
use app::MetaSound;
use eframe::epi;
use kira::{instance::InstanceSettings, manager::{AudioManager, AudioManagerSettings}, sound::SoundSettings};
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


    // let args = Opt::from_args();
    // if let Some(first_arg) = args.files.first() {
    //     let mut audio_manager = AudioManager::new(AudioManagerSettings::default()).unwrap();
    //     let mut sound_handle = audio_manager.load_sound(first_arg, SoundSettings::default()).unwrap();
    //         sound_handle.play(InstanceSettings::default()).unwrap();
    //         std::thread::sleep(std::time::Duration::from_secs(10));
    // }


    eframe::run_native(Box::new(app), opts);
}
