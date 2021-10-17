use eframe::egui::{self, Color32, FontDefinitions, FontFamily, TextStyle};
#[cfg(feature = "persistence")]
use serde::{Deserialize, Serialize};


#[cfg_attr(feature = "persistence", derive(Deserialize, Serialize))]

pub enum Theme {
    EguiLight,
    EguiDark,
    Red,
    Grey
}

impl Default for Theme {
    fn default() -> Self {
        Theme::Red
    }
}

impl Theme {
    pub fn apply(&self, ctx: &egui::CtxRef) {
        let main_col = Color32::from_rgb(255, 144, 144);
        ctx.set_visuals(egui::Visuals::light());
        let mut style: egui::Style = (*ctx.style()).clone();
        style.visuals.widgets.inactive.bg_fill = main_col;
        style.visuals.widgets.active.bg_fill = main_col;
        style.visuals.widgets.open.bg_fill = main_col;
        style.visuals.selection.bg_fill = main_col;
        // style.visuals.widgets.noninteractive.bg_fill = main_col;
        ctx.set_style(style);

        let mut fonts = FontDefinitions::default();

        // Install my own font (maybe supporting non-latin characters):
        fonts.font_data.insert(
            "my_font".to_owned(),
            std::borrow::Cow::Borrowed(include_bytes!("IBMPlexSans-Regular.ttf")),
        ); // .ttf and .otf supported

        // Put my font first (highest priority):
        fonts
            .fonts_for_family
            .get_mut(&FontFamily::Proportional)
            .unwrap()
            .insert(0, "my_font".to_owned());

        fonts.family_and_size.insert(
            TextStyle::Body,
            (FontFamily::Proportional, 18.0)
        );
        fonts.family_and_size.insert(
            TextStyle::Button,
            (FontFamily::Proportional, 18.0)
        );

        ctx.set_fonts(fonts);
    }
}