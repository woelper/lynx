use eframe::egui::{self, DroppedFile, ScrollArea, Vec2};
use std::collections::{HashMap, HashSet};

use structopt::StructOpt;

use crate::theme::Theme;
use crate::ui_components::*;
use kira::instance::{InstanceSettings, InstanceState, StopInstanceSettings};
use kira::manager::AudioManagerSettings;
use kira::{
    instance::{PauseInstanceSettings, ResumeInstanceSettings},
    manager::AudioManager,
};
use log::{debug, info};

use super::sound::*;
use eframe::epi;

#[cfg(feature = "persistence")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "persistence", derive(Deserialize, Serialize))]
pub struct ApplicationState {
    #[serde(skip)]
    pub audiomanager: Option<AudioManager>,
    pub active_sound: Option<MetaSound>,
    volume: f64,
    queue: SoundQueue,
    play_count: HashMap<MetaSound, usize>,
    favourites: HashSet<MetaSound>,
    bookmarks: HashSet<MetaSound>,
    theme: Theme,
    powersave: bool,
}

impl Default for ApplicationState {
    fn default() -> Self {
        Self {
            audiomanager: None,
            active_sound: None,
            volume: 1.0,
            queue: vec![],
            play_count: HashMap::default(),
            favourites: HashSet::default(),
            bookmarks: HashSet::default(),
            theme: Theme::default(),
            powersave: true,
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

        
        self.theme.apply(ctx);

        // Parse arguments to auto-play sound
        let args = super::Opt::from_args();

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
            theme,
            powersave,
        } = self;
        if egui::CentralPanel::default()
            .show(ctx, |ui| {
                // ctx.style_ui(ui);
                ctx.request_repaint();

                // info!("{:?}", ctx.input().raw);
                if !ctx.input().raw.dropped_files.is_empty() {
                    info!("{:?}", ctx.input().raw.dropped_files);
                    handle_dropped(&ctx.input().raw.dropped_files, queue);
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
                            if ui.add(egui::Button::new("⏮")).clicked() {
                                if let Some(i) = queue.to_index(&current_metasound.clone()) {
                                    let ri = (i as isize - 1).max(0) as usize;
                                    play_as_active(active_sound, &queue[ri], manager, play_count);
                                }
                            }
                        }

                        // info about current song
                        if let Some(current_metasound) = active_sound {
                            if let Some(instancehandle) = current_metasound.instancehandle.as_mut()
                            {
                                if let Some(soundhandle) = current_metasound.soundhandle.as_mut() {
                                    // done playing?
                                    debug!(
                                        "{}",
                                        instancehandle.position() - soundhandle.duration()
                                    );
                                    if instancehandle.position() - soundhandle.duration() > -0.05 {
                                        info!("Sound has finished playing, next one!");
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
                            if let Some(instancehandle) = current_metasound.instancehandle.as_mut()
                            {
                                if let Some(soundhandle) = current_metasound.soundhandle.as_mut() {
                                    match instancehandle.state() {
                                        InstanceState::Playing => {
                                            if ui.button("⏸").clicked() {
                                                let _ =
                                                    soundhandle.pause(PauseInstanceSettings::new());
                                            }
                                            if ui.button("⏹").clicked() {
                                                let _ =
                                                    soundhandle.stop(StopInstanceSettings::new());
                                            }
                                        }
                                        InstanceState::Paused(_) => {
                                            if ui.button("▶").clicked() {
                                                let _ = soundhandle
                                                    .resume(ResumeInstanceSettings::new());
                                            }
                                            if ui.button("⏹").clicked() {
                                                let _ =
                                                    soundhandle.stop(StopInstanceSettings::new());
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
                                    *current_metasound =
                                        current_metasound.load_soundhandle(manager);
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

                    ScrollArea::auto_sized().show(ui, |ui| {
                        playlist_ui(queue, active_sound, play_count, manager, ui);
                        playcount_ui(active_sound, play_count, manager, ui);
                        favourite_ui(active_sound, favourites, play_count, manager, ui);
                        bookmark_ui(active_sound, bookmarks, manager, ui);
                        settings_ui(theme, powersave, ui);
                    });
                } else {
                    ui.label("No Audio manager");
                }
            })
            .response
            .hovered()
            || !*powersave
        {
            // only repaint on hover
            ctx.request_repaint();
        }
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

/// Recurse dropped folders
fn handle_dropped(dropped_files: &Vec<DroppedFile>, queue: &mut SoundQueue) {
    for p in dropped_files.iter().filter_map(|d| d.path.as_ref()) {
        if p.is_dir() {
            for f in walkdir::WalkDir::new(p)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().is_some())
            {
                let s = MetaSound::default().with_path(f.path()).try_meta();
                if s.is_supported() {
                    queue.push(s);
                }
            }
        } else {
            let s = MetaSound::default().with_path(p).try_meta();
            if s.is_supported() {
                queue.push(s);
            }
        }
    }
}
