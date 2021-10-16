use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use eframe::egui::{CursorIcon, Id, LayerId, Order};
use kira::instance::{InstanceSettings, InstanceState, StopInstanceSettings};
use kira::manager::AudioManagerSettings;
use kira::{
    instance::{handle::InstanceHandle, PauseInstanceSettings, ResumeInstanceSettings},
    manager::AudioManager,
};

use super::sound::*;
use eframe::{
    egui::{self, Color32, Response, Sense, Stroke, Ui},
    epi,
};
use structopt::StructOpt;

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
    pub audiomanager: Option<AudioManager>,
    pub active_sound: Option<MetaSound>,
    volume: f64,
    queue: SoundQueue,
    queue_index: usize,
    play_count: HashMap<PathBuf, usize>,
    favourites: HashSet<PathBuf>,
    bookmarks: HashSet<MetaSound>,
}

impl Default for ApplicationState {
    fn default() -> Self {
        Self {
            audiomanager: None,
            active_sound: None,
            volume: 1.0,
            queue: vec![],
            queue_index: 0,
            play_count: HashMap::default(),
            favourites: HashSet::default(),
            bookmarks: HashSet::default(),
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
        ctx: &egui::CtxRef,
        _frame: &mut epi::Frame<'_>,
        storage: Option<&dyn epi::Storage>,
    ) {
        if let Some(storage) = storage {
            let storage: ApplicationState =
                epi::get_value(storage, epi::APP_KEY).unwrap_or_default();
            *self = storage;
        }

        let main_col = Color32::from_rgb(255, 144, 144);
        ctx.set_visuals(egui::Visuals::light());
        let mut style: egui::Style = (*ctx.style()).clone();
        style.visuals.widgets.inactive.bg_fill = main_col;
        style.visuals.widgets.active.bg_fill = main_col;
        style.visuals.widgets.open.bg_fill = main_col;
        style.visuals.selection.bg_fill = main_col;
        // style.visuals.widgets.noninteractive.bg_fill = main_col;
        ctx.set_style(style);

        // Parse arguments to auto-play sound
        let args = Opt::from_args();

        // Create an AudioManager
        self.audiomanager = AudioManager::new(AudioManagerSettings::default()).ok();

        // If the application was called with files as an argument, play the first
        if let Some(first_arg) = args.files.first() {
            if let Some(manager) = &mut self.audiomanager {
                // restore previous volume
                let _ = manager.main_track().set_volume(self.volume);
                // load the sound from disk
                let sound = MetaSound::default()
                    .with_path(first_arg)
                    .try_meta()
                    .load_soundhandle(manager);

                self.active_sound = Some(sound.clone());
                self.active_sound.as_mut().map(|s| s.play());

                dbg!(&self.active_sound);
                // push sound into queue
                // if !self.queue.contains(&sound) {
                // }
                self.queue.push(sound);
                self.queue_index = self.queue.len() - 1;
            }
        }
    }

    #[cfg(feature = "persistence")]
    fn save(&mut self, storage: &mut dyn epi::Storage) {
        epi::set_value(storage, epi::APP_KEY, self);
    }

    fn update(&mut self, ctx: &egui::CtxRef, _frame: &mut epi::Frame<'_>) {
        let ApplicationState {
            audiomanager: manager,
            active_sound,
            volume,
            queue,
            queue_index,
            bookmarks,
            favourites,
            play_count,
        } = self;
        
        // Repaint every frame to update progress bar etc
        ctx.request_repaint();
        egui::CentralPanel::default().show(ctx, |ui| {
            // ctx.style_ui(ui);
            // Handle dropped files. TODO: On dir drop, add recursively
            if !ctx.input().raw.dropped_files.is_empty() {
                for file in ctx
                    .input()
                    .raw
                    .dropped_files
                    .iter()
                    .filter_map(|d| d.path.as_ref())
                {
                    let s = MetaSound::default().with_path(file).try_meta();
                    queue.push(s);
                }
            }

            // info about current song
            if let Some(current_metasound) = active_sound {
                // ui.label(format!(
                //     "{} {} kHz, {} channels len {:?}, soundhandle: {:?} instance {:?}",
                //     current_metasound.name,
                //     current_metasound.sample_rate,
                //     current_metasound.channels,
                //     current_metasound.duration,
                //     current_metasound.soundhandle,
                //     current_metasound.instancehandle,
                // ));

                ui.label(format!("{}", current_metasound.name,));

                if let Some(manager) = manager {
                    if ui
                        .add(
                            egui::Slider::new(volume, 0.0..=3.0)
                                .text("🔈")
                                .show_value(false),
                        )
                        .changed()
                    {
                        let _ = manager.main_track().set_volume(*volume as f64);
                    }

                    if let Some(instancehandle) = current_metasound.instancehandle.as_mut() {
                        if let Some(soundhandle) = current_metasound.soundhandle.as_mut() {
                            let cur_pos = instancehandle.position();
                            let len = soundhandle.duration();
                            let progress = (cur_pos / len) as f32;

                            if let Some(pos) = scrubber(ui, progress).interact_pointer_pos() {
                                let w = ui.available_size().x;
                                let p = pos.x;
                                let fac = (p / w) as f64;
                                let _ = instancehandle.seek_to(fac * len);
                            }

                            ui.horizontal(|ui| {
                                if ui
                                    .add(egui::Button::new("⏮").enabled(*queue_index != 0))
                                    .clicked()
                                {
                                    *queue_index -= 1;
                                }

                                match instancehandle.state() {
                                    InstanceState::Playing => {
                                        if ui.button("⏸").clicked() {
                                            let _ = soundhandle.pause(PauseInstanceSettings::new());
                                        }
                                        if ui.button("⏹").clicked() {
                                            let _ = soundhandle.stop(StopInstanceSettings::new());
                                        }
                                    }
                                    InstanceState::Paused(_) => {
                                        if ui.button("▶").clicked() {
                                            let _ =
                                                soundhandle.resume(ResumeInstanceSettings::new());
                                            // active_sound.play(InstanceSettings::new()).unwrap();
                                        }
                                        if ui.button("⏹").clicked() {
                                            let _ = soundhandle.stop(StopInstanceSettings::new());
                                        }
                                    }
                                    InstanceState::Stopped => {
                                        if ui.button("▶").clicked() {
                                            let _ = soundhandle.play(InstanceSettings::new());
                                        }
                                    }
                                    _ => {}
                                }
                                if ui
                                    .add(egui::Button::new("⏭").enabled(*queue_index < queue.len()))
                                    .clicked()
                                {
                                    *queue_index += 1;
                                }
                            });
                        } else {
                            ui.label("No active sound handle");
                        }
                    } else {
                        // There is no active instance handle, offer to play
                        ui.label("No active sound instance");
                        if ui.button("▶").clicked() {
                            play_from_queue(active_sound, queue, queue_index, manager);
                        }
                    }

                    //playlist
                    playlist_ui(queue, queue_index, active_sound, manager, ui)

                    // playlist_ui()
                } else {
                    ui.label("Could not create an AudioManager.");
                }
            } else {
                ui.label("No active sound");
            }
        });
    }

    fn warm_up_enabled(&self) -> bool {
        false
    }

    fn on_exit(&mut self) {}

    fn auto_save_interval(&self) -> std::time::Duration {
        std::time::Duration::from_secs(30)
    }

    fn max_size_points(&self) -> egui::Vec2 {
        // Some browsers get slow with huge WebGL canvases, so we limit the size:
        egui::Vec2::new(1024.0, 2048.0)
    }

    fn clear_color(&self) -> egui::Rgba {
        // NOTE: a bright gray makes the shadows of the windows look weird.
        // We use a bit of transparency so that if the user switches on the
        // `transparent()` option they get immediate results.
        egui::Color32::from_rgba_unmultiplied(12, 12, 12, 180).into()
    }
}

/// The scrollbar / scrub bar
pub fn scrubber(ui: &mut Ui, scale: f32) -> Response {
    let mut dim = ui.available_rect_before_wrap_finite();
    dim.set_height(ui.spacing().interact_size.y);
    let x = ui.allocate_rect(dim, Sense::click());
    let radius = ui.style().visuals.widgets.active.corner_radius;
    ui.painter().rect(
        dim,
        radius,
        ui.style().visuals.extreme_bg_color,
        Stroke::default(),
    );
    dim.set_width(dim.width() * scale);
    ui.painter().rect(
        dim,
        radius,
        ui.style().visuals.widgets.active.bg_fill,
        Stroke::default(),
    );
    x
}

fn playlist_ui(
    queue: &mut SoundQueue,
    queue_index: &mut usize,
    active_sound: &mut Option<MetaSound>,
    manager: &mut AudioManager,
    ui: &mut Ui,
) {
    ui.vertical_centered_justified(|ui| {
        let mut drag_index: Option<usize> = None;
        let mut drop_index: Option<usize> = None;
        // if ui.button("clr").clicked() {
        //     queue.clear();
        // }
        for (i, sound) in queue.clone().iter().enumerate() {
            let pl_item = ui
                .selectable_label(*queue_index == i, &sound.name)
                .interact(egui::Sense::click_and_drag());

            if pl_item.drag_released() {
                drag_index = Some(i);
            }

            if pl_item.dragged() {
                ui.output().cursor_icon = CursorIcon::Grabbing;

                // Paint the body to a new layer:
                let layer_id = LayerId::new(Order::Tooltip, pl_item.id);
                let response = ui
                    .with_layer_id(layer_id, |ui| ui.label("we got a dragger"))
                    .response;

                if let Some(pointer_pos) = ui.input().pointer.interact_pos() {
                    let delta = pointer_pos - response.rect.center();
                    // dbg!(&delta.y);
                    ui.ctx().translate_layer(layer_id, delta);
                }
            }

            if pl_item.double_clicked() {
                // update index to current
                *queue_index = i;
                // stop current sound
                // let _ = active_sound.as_mut().map(|s| s.stop());
                // assign queue sound as active
                // *active_sound = queue.get(*queue_index).map(|s| s.load_soundhandle(manager));
                // let _ = active_sound.as_mut().map(|s| s.play());
                play_from_queue(active_sound, queue, queue_index, manager)
            }

            if pl_item.hovered() {
                drop_index = Some(i);
            }
        }

        if ui.input().pointer.any_released() {
            // swap
            dbg!(drag_index, drop_index);
            if let (Some(drag), Some(drop)) = (drag_index, drop_index) {
                let elem = queue.remove(drag);
                queue.insert(drop, elem);
                // queue.swap(drag, drop);
            }
        }
    });
}

pub fn play_from_queue(
    sound: &mut Option<MetaSound>,
    queue: &mut SoundQueue,
    queue_index: &mut usize,
    manager: &mut AudioManager,
) {
    let _ = sound.as_mut().map(|s| s.stop());
    *sound = queue.get(*queue_index).map(|s| s.load_soundhandle(manager));
    let _ = sound.as_mut().map(|s| s.play());
}
