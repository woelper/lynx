use std::collections::{HashMap, HashSet};

use eframe::egui::{
    Color32, ComboBox, CtxRef, CursorIcon, Label, LayerId, Order, Response, SelectableLabel, Sense,
    Stroke, Ui, Vec2,
};
use kira::manager::{AudioManager, backend::Backend};
use kira_cpal::CpalBackend;

use crate::{
    sound::{MetaSound, SoundQueue},
    theme::{grad_button, Theme},
};

pub fn playlist_ui(
    queue: &mut SoundQueue,
    active_sound: &mut Option<MetaSound>,
    play_count: &mut HashMap<MetaSound, usize>,
    manager: &mut AudioManager<CpalBackend>,
    ui: &mut Ui,
) {
    ui.collapsing("♫ Playlist", |ui| {
        ui.vertical_centered_justified(|ui| {
            let mut drag_index: Option<usize> = None;
            let mut drop_index: Option<usize> = None;
            // if ui.button("clr").clicked() {
            //     queue.clear();
            // }
            for (i, sound) in queue.clone().iter().enumerate() {
                ui.horizontal(|ui| {
                    let pl_item = ui
                        .selectable_label(Some(sound) == active_sound.as_ref(), &sound.name)
                        .interact(Sense::click_and_drag());

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
                                        [0.0, 0.0],
                                        Label::new(&sound.name).background_color(
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
                    } else {
                        if ui
                            .add(Label::new("🗙").small().weak().sense(Sense::click()))
                            .clicked()
                        {
                            queue.remove(i);
                        }
                    }

                    if pl_item.hovered() {
                        drop_index = Some(i);
                    }
                });
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

pub fn playcount_ui(
    // queue_index: &mut usize,
    active_sound: &mut Option<MetaSound>,
    counter: &mut HashMap<MetaSound, usize>,
    manager: &mut AudioManager<CpalBackend>,
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

        for sound in &sorted {
            ui.horizontal(|ui| {
                ui.label(format!("{:02}", sound.1));
                if grad_button("▶", ui).clicked() {
                    play_as_active(active_sound, &sound.0, manager, counter);
                }
                ui.label(&sound.0.name);
            });
        }
    });
}

pub fn favourite_ui(
    // queue_index: &mut usize,
    active_sound: &mut Option<MetaSound>,
    favourites: &mut HashSet<MetaSound>,
    counter: &mut HashMap<MetaSound, usize>,
    manager: &mut AudioManager<CpalBackend>,
    ui: &mut Ui,
) {
    ui.collapsing("♡ Favourites", |ui| {
        for favsound in favourites.iter() {
            ui.horizontal(|ui| {
                if grad_button("▶", ui).clicked() {
                    play_as_active(active_sound, favsound, manager, counter);
                }
                ui.label(&favsound.name);
            });
        }
    });
}

pub fn bookmark_ui(
    // queue_index: &mut usize,
    active_sound: &mut Option<MetaSound>,
    bookmarks: &mut HashSet<MetaSound>,
    manager: &mut AudioManager<CpalBackend>,
    ui: &mut Ui,
) {
    ui.collapsing("🔖 Bookmarks", |ui| {
        for s in bookmarks.iter() {
            ui.label(&s.name);
            ui.horizontal(|ui| {
                for b in &s.bookmarks {
                    if grad_button(format!("{:.1}", b), ui).clicked() {
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

/// The scrollbar / scrub bar
pub fn scrubber(ui: &mut Ui, scale: f32) -> Response {
    let mut dim = ui.available_rect_before_wrap();
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

pub fn settings_ui(theme: &mut Theme, powersave: &mut bool, ui: &mut Ui) {
    ui.collapsing("⛭ Settings", |ui| {
        ui.checkbox(powersave, "Powersave mode");

        ComboBox::from_label("Theme")
            .selected_text(format!("{:?}", theme))
            .show_ui(ui, |ui| {
                for t in [Theme::EguiDark, Theme::EguiLight, Theme::Red] {
                    if ui
                        .selectable_value(theme, t.clone(), format!("{:?}", t))
                        .clicked()
                    {
                        theme.apply(ui.ctx());
                    }
                }
            });
    });
}

pub fn play_as_active(
    active_sound: &mut Option<MetaSound>,
    sound: &MetaSound,
    manager: &mut AudioManager<CpalBackend>,
    counter: &mut HashMap<MetaSound, usize>,
) {
    let _ = active_sound.as_mut().map(|s| s.stop());
    *active_sound = Some(sound.clone());
    let _ = active_sound.as_mut().map(|s| s.play_load_mut(manager));
    *counter.entry(sound.clone()).or_insert(0) += 1;
}
