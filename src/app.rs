use std::{fs::File, io::BufReader, path::PathBuf};

use structopt::StructOpt;

use eframe::{
    egui::{self},
    epi,
};

use rodio::{Decoder, OutputStream, OutputStreamHandle, Source};

#[cfg(feature = "persistence")]
use serde::{Deserialize, Serialize};

#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
struct Opt {
    /// Files to process
    #[structopt(name = "files", parse(from_os_str))]
    files: Vec<PathBuf>,
}

#[cfg_attr(feature = "persistence", derive(Deserialize, Serialize))]
pub struct ApplicationState {

    #[serde(skip)]
    pub active_sound: Option<Decoder<BufReader<File>>>,
    #[serde(skip)]
    pub streams: Option<(OutputStream, OutputStreamHandle)>,
}

impl Default for ApplicationState {
    fn default() -> Self {
        Self {
            active_sound: None,
            streams: None,
        }
    }
}

impl epi::App for ApplicationState {
    fn name(&self) -> &str {
        env!("CARGO_PKG_NAME")
    }

    #[cfg(feature = "persistence")]
    fn setup(
        &mut self,
        _ctx: &egui::CtxRef,
        _frame: &mut epi::Frame<'_>,
        storage: Option<&dyn epi::Storage>,
    ) {
        let args = Opt::from_args();
        let input_files = args.files;
        let file = BufReader::new(File::open(&input_files.first().unwrap()).unwrap());
        // Decode that sound file into a source
        let source = Decoder::new(file).unwrap();

        if let Some(storage) = storage {
            let storage: ApplicationState =
                epi::get_value(storage, epi::APP_KEY).unwrap_or_default();
            *self = storage;
            self.active_sound = Some(source);
            self.streams = OutputStream::try_default().ok();
        }

        dbg!(&self.streams.is_some());
    }

    #[cfg(feature = "persistence")]
    fn save(&mut self, storage: &mut dyn epi::Storage) {
        epi::set_value(storage, epi::APP_KEY, self);
    }

    fn update(&mut self, ctx: &egui::CtxRef, frame: &mut epi::Frame<'_>) {
        let ApplicationState {
            active_sound,
            streams,
        } = self;

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                egui::menu::menu(ui, "File", |ui| {
                    if ui.button("💣 Quit").clicked() {
                        frame.quit();
                    }
                });
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Central panel");
            ui.separator();

            if let Some(sound) = &active_sound {
                // dbg!("znd");
                ui.label(format!("Rate {}", sound.sample_rate()));
                ui.label(format!("Channels {}", sound.channels()));
            } else {
                ui.label("No sound");
            }

            if let Some((_str, handle)) = streams {
            } else {
                ui.label("No stream");
            }

            if ui.button("Play").clicked() {
                if let Some(sound) = active_sound.take() {
                    if let Some((_str, handle)) = streams {
                        handle.play_raw(sound.convert_samples()).unwrap();
                    } else {
                        ui.label("No stream");
                    }
                }
            }
        });

        // set to true to test floating window
        if false {
            egui::Window::new("Window").show(ctx, |ui| {
                ui.label("Windows can be moved by dragging them.");
                ui.label("They are automatically sized based on contents.");
                ui.label("You can turn on resizing and scrolling if you like.");
                ui.label("You would normally chose either panels OR windows.");
            });
        }
    }
}
