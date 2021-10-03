use anyhow::{anyhow, Result};
use std::{
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
    time::Duration,
};

use eframe::{
    egui::{self},
    epi,
};
use std::ffi::OsStr;
use structopt::StructOpt;

use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink, Source};

#[cfg(feature = "persistence")]
use serde::{Deserialize, Serialize};

type SoundQueue = Vec<Sound>;

#[cfg_attr(feature = "persistence", derive(Deserialize, Serialize))]
#[derive(Debug, Default, Clone)]
pub struct Sound {
    pub path: PathBuf,
    pub sample_rate: u32,
    pub channels: u16,
    pub duration: Option<Duration>,
    pub looped: bool,
}

impl Sound {
    pub fn with_path<P: AsRef<Path>>(&self, path: P) -> Self {
        Self {
            path: path.as_ref().into(),
            ..self.clone()
        }
    }

    pub fn generate_source(&mut self) -> Result<Decoder<BufReader<File>>> {
        let open_file = File::open(&self.path)?;
        let r = BufReader::new(open_file);
        // Decode that sound file into a source
        let source = Decoder::new(r)?;
        self.sample_rate = source.sample_rate();
        self.channels = source.channels();
        Ok(source)
    }
}

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
    #[serde(skip)]
    pub sink: Option<Sink>,
    volume: f32,
    queue: SoundQueue,
}

impl Default for ApplicationState {
    fn default() -> Self {
        Self {
            active_sound: None,
            streams: None,
            sink: None,
            volume: 1.0,
            queue: vec![],
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
        if let Some(storage) = storage {
            let storage: ApplicationState =
                epi::get_value(storage, epi::APP_KEY).unwrap_or_default();
            *self = storage;
            self.streams = OutputStream::try_default().ok();
            self.sink = Sink::try_new(&self.streams.as_ref().unwrap().1).ok();
        }

        // Parse arguments to auto-play sound
        let args = Opt::from_args();

        if let Some(first_arg) = args.files.first() {
            let mut sound = Sound::default().with_path(first_arg);
            if let Ok(source) = sound.generate_source() {
                self.queue.push(sound);
                if let Some(sink) = &self.sink {
                    sink.append(source);
                }
            }
        }
    }

    #[cfg(feature = "persistence")]
    fn save(&mut self, storage: &mut dyn epi::Storage) {
        epi::set_value(storage, epi::APP_KEY, self);
    }

    fn update(&mut self, ctx: &egui::CtxRef, frame: &mut epi::Frame<'_>) {
        let ApplicationState {
            active_sound,
            streams,
            sink,
            volume,
            queue,
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
            if !ctx.input().raw.dropped_files.is_empty() {
                for file in ctx
                    .input()
                    .raw
                    .dropped_files
                    .iter()
                    .filter_map(|d| d.path.as_ref())
                {
                    let s = Sound::default().with_path(file);
                    queue.push(s);
                }
            }

            if let Some(sound) = &active_sound {
                // dbg!("znd");
                ui.label(format!("Rate {}", sound.sample_rate()));
                ui.label(format!("Channels {}", sound.channels()));
            } else {
                ui.label("No sound");
            }

            if let Some(sink) = sink {
                ui.horizontal(|ui| {
                    if ui.button("Play").clicked() {
                        sink.play();
                    }

                    if ui.button("Pause").clicked() {
                        sink.pause();
                    }

                    if ui.button("Stop").clicked() {
                        sink.stop();
                    }
                });

                if ui
                    .add(egui::Slider::new(volume, 0.0..=3.0).text("volume"))
                    .changed()
                {
                    sink.set_volume(*volume);
                }

                for sound in queue {
                    if ui
                        .button(format!(
                            "{}",
                            sound
                                .path
                                .file_name()
                                .unwrap_or(OsStr::new("no path"))
                                .to_string_lossy()
                        ))
                        .clicked()
                    {
                        if let Ok(source) = sound.generate_source() {
                            dbg!("loaded", &sound);
                            // due to rodio's design, the sink is lost after stopping
                            *sink = Sink::try_new(&streams.as_ref().unwrap().1).unwrap();

                            sink.append(source);
                            sink.play();
                        }
                    }
                }
            }
        });
    }
}
