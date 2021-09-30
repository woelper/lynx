mod app;
use eframe::epi;
use std::path::PathBuf;
use structopt::StructOpt;

use rodio::{source::Source, Decoder, OutputStream};
use std::fs::File;
use std::io::BufReader;

#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
struct Opt {
    /// Files to process
    #[structopt(name = "files", parse(from_os_str))]
    files: Vec<PathBuf>,
}

fn main() {
    // let args = Opt::from_args();
    // let input_files = args.files;

    // Get a output stream handle to the default physical sound device
    //let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    // Load a sound from a file, using a path relative to Cargo.toml
    // let file = BufReader::new(File::open(&input_files.first().unwrap()).unwrap());
    // // Decode that sound file into a source
    // let source = Decoder::new(file).unwrap();

    // dbg!(&source.channels());

    let app = app::ApplicationState::default();
    // app.active_sound = Some(source);

    // if app.active_sound.is_some() {
    //     dbg!("some sound");
    // }

    let opts = epi::NativeOptions::default();
    eframe::run_native(Box::new(app), opts);
}
