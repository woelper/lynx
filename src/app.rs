use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use crate::theme::Theme;
use eframe::egui::{CursorIcon, LayerId, Order};
use kira::instance::{InstanceSettings, InstanceState, StopInstanceSettings};
use kira::manager::AudioManagerSettings;
use kira::{
    instance::{PauseInstanceSettings, ResumeInstanceSettings},
    manager::AudioManager,
};
use log::{debug, info};

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
    // queue_index: usize,
    play_count: HashMap<MetaSound, usize>,
    favourites: HashSet<MetaSound>,
    bookmarks: HashSet<MetaSound>,
}

impl Default for ApplicationState {
    fn default() -> Self {
        Self {
            audiomanager: None,
            active_sound: None,
            volume: 1.0,
            queue: vec![],
            // queue_index: 0,
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

        let theme = Theme::Red;
        theme.apply(ctx);

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

                // check if playlist has this sound
                if !self.queue.contains(&sound) {
                    self.queue.push(sound);
                }
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
            bookmarks,
            favourites,
            play_count,
        } = self;

        // Repaint every frame to update progress bar etc
        ctx.request_repaint();
        egui::CentralPanel::default().show(ctx, |ui| {
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

            if let Some(manager) = manager {
                if let Some(current_metasound) = active_sound {
                    ui.label(&current_metasound.name);
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
                }

                if let Some(current_metasound) = active_sound {
                    if let Some(instancehandle) = current_metasound.instancehandle.as_mut() {
                        if let Some(soundhandle) = current_metasound.soundhandle.as_mut() {
                            let cur_pos = instancehandle.position();
                            let len = soundhandle.duration();
                            let progress = (cur_pos / len) as f32;

                            // current_metasound.soundhandle.unwrap().
                            let response = scrubber(ui, progress);
                            if ui.input().pointer.any_pressed() {
                                if let Some(pos) = response.interact_pointer_pos() {
                                    let w = ui.available_size().x;
                                    let p = pos.x;
                                    let fac = (p / w) as f64;
                                    let _ = instancehandle.seek_to(fac * len);
                                }
                            }
                        }
                    }
                } else {
                    ui.label("No sound active");
                }

                ui.horizontal(|ui| {
                    if let Some(current_metasound) = active_sound {
                        if ui
                            .add(
                                egui::Button::new("⏮"), // .enabled(*queue_index != 0)
                            )
                            .clicked()
                        {
                            if let Some(i) = queue.to_index(&current_metasound.clone()) {
                                let ri = (i as isize - 1).max(0) as usize;
                                play_as_active(active_sound, &queue[ri], manager, play_count);
                            }
                        }
                    }

                    // info about current song
                    if let Some(current_metasound) = active_sound {
                        if let Some(instancehandle) = current_metasound.instancehandle.as_mut() {
                            if let Some(soundhandle) = current_metasound.soundhandle.as_mut() {
                                // done playing?
                                debug!("{}", instancehandle.position() - soundhandle.duration());
                                if instancehandle.position() - soundhandle.duration() > -0.05 {
                                    if let Some(i) = queue.to_index(current_metasound) {
                                        let ri = (i + 1).min(queue.len() - 1);
                                        play_as_active(
                                            active_sound,
                                            &queue[ri],
                                            manager,
                                            play_count,
                                        );
                                    }
                                }
                            }
                        }
                    }

                    // info about current song
                    if let Some(current_metasound) = active_sound {
                        if let Some(instancehandle) = current_metasound.instancehandle.as_mut() {
                            if let Some(soundhandle) = current_metasound.soundhandle.as_mut() {
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
                            } else {
                                ui.label("No active sound handle");
                            }
                        } else {
                            // There is no active instance handle, offer to play
                            // ui.label("No active sound instance");
                            if ui.button("▶").clicked() {
                                *current_metasound = current_metasound.load_soundhandle(manager);
                                info!("{:?}", current_metasound.play());
                                *play_count.entry(current_metasound.clone()).or_insert(0) += 1;
                            }
                        }

                        if ui
                            .add(
                                egui::Button::new("⏭"), // .enabled(*queue_index < queue.len())
                            )
                            .clicked()
                        {
                            if let Some(i) = queue.to_index(current_metasound) {
                                let ri = (i + 1).min(queue.len() - 1);
                                play_as_active(active_sound, &queue[ri], manager, play_count);
                            }
                        }
                    } else {
                        ui.label("No sound active");
                    }

                    if let Some(s) = active_sound {
                        if ui.button("♡").clicked() {
                            favourites.insert(s.clone());
                        }
                    }

                    if let Some(s) = active_sound {
                        if ui.button("🔖").clicked() {
                            if let Some(instancehandle) = &s.instancehandle {
                                s.bookmarks.push(instancehandle.position());
                                let mut prev_bookmarks = bookmarks
                                    .get(s)
                                    .map(|b| b.bookmarks.clone())
                                    .unwrap_or_default();
                                prev_bookmarks.extend(s.bookmarks.clone());
                                prev_bookmarks.sort_by(|a, b| a.partial_cmp(b).unwrap());
                                prev_bookmarks.dedup();
                                debug!("{:?}", prev_bookmarks);
                                s.bookmarks = prev_bookmarks;
                                bookmarks.replace(s.clone());
                            }
                        }
                    }

                    // end horizontal layout
                });

                //playlist
                playlist_ui(queue, active_sound, play_count, manager, ui);
                playcount_ui(active_sound, play_count, manager, ui);
                favourite_ui(active_sound, favourites, play_count, manager, ui);
                bookmark_ui(active_sound, bookmarks, manager, ui);
            } else {
                ui.label("No Audio manager");
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

    fn clear_color(&self) -> egui::Rgba {
        // NOTE: a bright gray makes the shadows of the windows look weird.
        // We use a bit of transparency so that if the user switches on the
        // `transparent()` option they get immediate results.
        egui::Color32::from_rgba_unmultiplied(12, 12, 12, 180).into()
    }
}

pub fn play_as_active(
    active_sound: &mut Option<MetaSound>,
    sound: &MetaSound,
    manager: &mut AudioManager,
    counter: &mut HashMap<MetaSound, usize>,
) {
    let _ = active_sound.as_mut().map(|s| s.stop());
    let _ = active_sound.as_mut().map(|s| s.play_load_mut(manager));
    *counter.entry(sound.clone()).or_insert(0) += 1;
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
    active_sound: &mut Option<MetaSound>,
    play_count: &mut HashMap<MetaSound, usize>,
    manager: &mut AudioManager,
    ui: &mut Ui,
) {
    ui.collapsing("♫ Playlist", |ui| {
        ui.vertical_centered_justified(|ui| {
            let mut drag_index: Option<usize> = None;
            let mut drop_index: Option<usize> = None;
            if ui.button("clr").clicked() {
                queue.clear();
            }
            for (i, sound) in queue.clone().iter().enumerate() {
                // the laylist entry widget
                let pl_item = ui
                    .selectable_label(Some(sound) == active_sound.as_ref(), &sound.name)
                    .interact(egui::Sense::click_and_drag());

                if pl_item.drag_released() {
                    drag_index = Some(i);
                }

                if pl_item.double_clicked() {
                    play_as_active(active_sound, sound, manager, play_count);
                }

                if pl_item.dragged() {
                    // discard small drags, otherwise it looks awkward on double clicks
                    let mut dist = 0.0;
                    if let Some(pointer_pos) = ui.input().pointer.interact_pos() {
                        if let Some(orig) = ui.input().pointer.press_origin() {
                            dist = pointer_pos.distance(orig);
                        }
                    }

                    // Just a small distance to ensure user has moved a meaningful amount
                    if dist > 5.2 {
                        ui.output().cursor_icon = CursorIcon::Grabbing;

                        // Paint the body to a new layer:
                        let layer_id = LayerId::new(Order::Tooltip, pl_item.id);
                        let response = ui
                            .with_layer_id(layer_id, |ui| {
                                ui.add_sized(
                                    [40.0, 0.0],
                                    egui::Label::new(&sound.name).background_color(
                                        Color32::from_rgba_premultiplied(0, 0, 0, 50),
                                    ),
                                );
                                // ui.put(Rect::NOTHING, egui::Label::new("SDSDSDD"));
                                // // let r = ui.allocate_exact_size(Vec2::ZERO, Sense::click_and_drag());
                                // ui.add_sized(Vec2::ZERO, |ui| {
                                //     ui.label(&sound.name);
                                // });
                            })
                            .response;

                        if let Some(pointer_pos) = ui.input().pointer.interact_pos() {
                            let delta = pointer_pos - response.rect.center();
                            ui.ctx().translate_layer(layer_id, delta);
                        }
                    }
                }

                if pl_item.hovered() {
                    drop_index = Some(i);
                }
            }

            if ui.input().pointer.any_released() {
                // swap
                if let (Some(drag), Some(drop)) = (drag_index, drop_index) {
                    let elem = queue.remove(drag);
                    queue.insert(drop, elem);
                }
            }
        });
    });
}

fn playcount_ui(
    // queue_index: &mut usize,
    active_sound: &mut Option<MetaSound>,
    counter: &mut HashMap<MetaSound, usize>,
    manager: &mut AudioManager,
    ui: &mut Ui,
) {
    ui.collapsing("🔥 Most played", |ui| {
        // for s in counter
        let mut sorted = counter
            .iter()
            .map(|x| (x.0.clone(), *x.1))
            .collect::<Vec<_>>();
        sorted.sort_by_key(|a| a.1);
        sorted.reverse();

        for sound in sorted {
            ui.horizontal(|ui| {
                ui.label(format!("{}", sound.1));
                if ui.button("▶").clicked() {
                    play_as_active(active_sound, &sound.0, manager, counter);
                }
                ui.label(&sound.0.name);
            });
        }
    });
}

fn favourite_ui(
    // queue_index: &mut usize,
    active_sound: &mut Option<MetaSound>,
    favourites: &mut HashSet<MetaSound>,
    counter: &mut HashMap<MetaSound, usize>,
    manager: &mut AudioManager,
    ui: &mut Ui,
) {
    ui.collapsing("♡ Favourites", |ui| {
        for favsound in favourites.iter() {
            ui.horizontal(|ui| {
                if ui.button("▶").clicked() {
                    play_as_active(active_sound, favsound, manager, counter);
                }
                ui.label(&favsound.name);
            });
        }
    });
}

fn bookmark_ui(
    // queue_index: &mut usize,
    active_sound: &mut Option<MetaSound>,
    bookmarks: &mut HashSet<MetaSound>,
    manager: &mut AudioManager,
    ui: &mut Ui,
) {
    ui.collapsing("🔖 Bookmarks", |ui| {
        for s in bookmarks.iter() {
            ui.label(&s.name);
            ui.horizontal(|ui| {
                for b in &s.bookmarks {
                    if ui.button(format!("{:.1}", b)).clicked() {
                        if let Some(active) = active_sound {
                            //check if current sound is the one referenced in bookmark
                            if active == s {
                                if let Some(instancehandle) = active.instancehandle.as_mut() {
                                    let _ = instancehandle.seek_to(*b);
                                }
                            } else {
                                active.stop();
                                *active = s.clone();
                                let _ = active.play_load_mut(manager);
                                if let Some(instancehandle) = active.instancehandle.as_mut() {
                                    let _ = instancehandle.seek_to(*b);
                                }
                            }
                        }
                    }
                }
            });
        }
    });
}
