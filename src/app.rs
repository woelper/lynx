use anyhow::{anyhow, Result};
use std::{path::{Path, PathBuf}, sync::Arc, time::Duration};

use kira::{instance::{PauseInstanceSettings, ResumeInstanceSettings, handle::InstanceHandle}, manager::AudioManager, sound::{handle::SoundHandle, SoundSettings}};
use kira::manager::AudioManagerSettings;
use kira::instance::InstanceSettings;


use eframe::{egui::{self, Label}, epi};
use std::ffi::OsStr;
use structopt::StructOpt;

#[cfg(feature = "persistence")]
use serde::{Deserialize, Serialize};

type SoundQueue = Vec<MetaSound>;

#[cfg_attr(feature = "persistence", derive(Deserialize, Serialize))]
#[derive(Debug, Clone, Default)]
pub struct MetaSound {
    pub path: PathBuf,
    pub sample_rate: u32,
    pub channels: u16,
    pub duration: Option<Duration>,
    pub looped: bool,
    #[serde(skip)]
    pub handle: Option<SoundHandle>,
}

impl MetaSound {
    pub fn with_path<P: AsRef<Path>>(&self, path: P) -> Self {
        Self {
            path: path.as_ref().into(),
            ..self.clone()
        }
    }

    pub fn generate_source(&mut self, manager: &mut AudioManager) -> Result<()> {
        let sound_handle = manager.load_sound(&self.path, SoundSettings::default())?;
        self.handle = Some(sound_handle);
        Ok(())
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
    pub manager: Option<AudioManager>,
    #[serde(skip)]
    active_sound: Option<SoundHandle>,
    #[serde(skip)]
    active_instance: Option<Arc<InstanceHandle>>,
    volume: f32,
    queue: SoundQueue,
    queue_index: usize,
}

impl Default for ApplicationState {
    fn default() -> Self {
        Self {
            manager: None,
            active_sound: None,
            active_instance: None,
            volume: 1.0,
            queue: vec![],
            queue_index: 0,
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
        }

        // Parse arguments to auto-play sound
        let args = Opt::from_args();

        // Create an AudioManager
        self.manager = AudioManager::new(AudioManagerSettings::default()).ok();

        // If the application was called with files as an argument, play the first
        if let Some(first_arg) = args.files.first() {
            let mut sound = MetaSound::default().with_path(first_arg);
            if let Some(manager) = &mut self.manager {
                self.active_sound = manager.load_sound(first_arg, SoundSettings::default()).ok();
                if let Some(active_sound) = &mut self.active_sound {

                    let inst = active_sound.play(InstanceSettings::default()).ok();
                    self.active_instance = Some(Arc::new(inst.unwrap()));
                }
                // let x  = sound_handle.play(InstanceSettings::default()).ok();

                // if let Ok(_) = sound.generate_source(manager) {
                //     self.active_sound =
                //         sound.handle.unwrap().play(InstanceSettings::default()).ok();
                //         dbg!(&self.active_sound);
                // }
            }
        }
    }

    #[cfg(feature = "persistence")]
    fn save(&mut self, storage: &mut dyn epi::Storage) {
        epi::set_value(storage, epi::APP_KEY, self);
    }

    fn update(&mut self, ctx: &egui::CtxRef, _frame: &mut epi::Frame<'_>) {
        let ApplicationState {
            manager,
            active_sound,
            active_instance,
            volume,
            queue,
            queue_index,
        } = self;

        
        // Repaint every frame to update progress bar etc
        ctx.request_repaint();

        egui::CentralPanel::default().show(ctx, |ui| {


            // Handle droppec files
            if !ctx.input().raw.dropped_files.is_empty() {
                for file in ctx
                    .input()
                    .raw
                    .dropped_files
                    .iter()
                    .filter_map(|d| d.path.as_ref())
                {
                    let s = MetaSound::default().with_path(file);
                    queue.push(s);
                }
            }

            if let Some(manager) = manager {
                if let Some(current_song) = queue.get(*queue_index) {
                    ui.label(format!(
                        "{} kHz, {} channels {:?}",
                        current_song.sample_rate,
                        current_song.channels,
                        current_song.duration.unwrap_or_default()
                    ));

                    if ui
                    .add(egui::Slider::new(volume, 0.0..=3.0).text("volume"))
                    .changed()
                    {
                        let _ = manager.main_track().set_volume(*volume as f64);
                    }
                }

                if let Some(active_sound) = active_sound {
                    // ui.label(format!("Pos {}", active_sound.));

                    ui.horizontal(|ui| {
                        if ui.button("Play").clicked() {
                            let _ = active_sound.resume(ResumeInstanceSettings::new());
                            // active_sound.play(InstanceSettings::new()).unwrap();
                        }

                        if ui.button("Pause").clicked() {
                            let _ = active_sound.pause(PauseInstanceSettings::new());
                        }

                        if ui.button("Stop").clicked() {

                        }
                    });

                    // TODO:move this out and put everything into MetaSound
                    if let Some(active_instance) = active_instance {
                        let cur_pos = active_instance.position();
                        let len = active_sound.duration();

                        let progress = (cur_pos/len) as f32;
                        ui.add(egui::ProgressBar::new(progress).text(format!("-{:.1}s", len - cur_pos)));
                        // ui.label(format!("pos {:.1} {:.1}% {:.1}", cur_pos, (cur_pos/len)*100., len));
                    }
                }



                ui.vertical_centered_justified(|ui| {
                    for (i, sound) in queue.iter_mut().enumerate() {
                        if ui
                            .selectable_label(*queue_index == i, nice_name(&sound.path))
                            .double_clicked()
                        {
                            *queue_index = i;
                            if let Ok(source) = sound.generate_source(manager) {
                                dbg!("loaded", &sound);
                                // due to rodio's design, the sink is lost after stopping
                            }
                        }
                    }
                });
            } else {
                ui.label("Could not create an AudioManager.");
            }
        });
    }
}

fn nice_name(p: &Path) -> String {
    format!(
        "{}",
        p.file_name()
            .unwrap_or(OsStr::new("no path"))
            .to_string_lossy()
            .replace("_", " ")
            .replace("-", " ")
    )
}
