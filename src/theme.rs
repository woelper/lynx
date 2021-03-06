use eframe::egui::{self, Color32, FontDefinitions, FontFamily, Response, TextStyle, Ui, FontData, WidgetText};
use log::info;
#[cfg(feature = "persistence")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "persistence", derive(Deserialize, Serialize))]
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Theme {
    EguiLight,
    EguiDark,
    Red,
    Grey,
}

impl Default for Theme {
    fn default() -> Self {
        Theme::Red
    }
}

impl Theme {
    pub fn apply(&self, ctx: &egui::CtxRef) {
        info!("Setting {:?}", self);
        match self {
            Theme::EguiDark => {
                ctx.set_visuals(egui::Visuals::dark());
            }
            Theme::EguiLight => {
                ctx.set_visuals(egui::Visuals::light());
            }
            Theme::Red => {
                ctx.set_visuals(egui::Visuals::light());
                let main_col = Color32::from_rgb(255, 144, 144);
                let mut style: egui::Style = (*ctx.style()).clone();
                style.visuals.widgets.inactive.bg_fill = main_col;
                style.visuals.widgets.active.bg_fill = main_col;
                style.visuals.widgets.open.bg_fill = main_col;
                style.visuals.selection.bg_fill = main_col;
                // style.visuals.widgets.noninteractive.bg_fill = main_col;
                ctx.set_style(style);
            }
            Theme::Grey => {
                ctx.set_visuals(egui::Visuals::light());
            }
        }

        let mut fonts = FontDefinitions::default();

        // Install my own font (maybe supporting non-latin characters):
        fonts.font_data.insert(
            "my_font".to_owned(),
            FontData::from_static(include_bytes!("IBMPlexSans-Regular.ttf"))
            // std::borrow::Cow::Borrowed(),
        ); // .ttf and .otf supported

        // Put my font first (highest priority):
        fonts
            .fonts_for_family
            .get_mut(&FontFamily::Proportional)
            .unwrap()
            .insert(0, "my_font".to_owned());

        fonts
            .family_and_size
            .insert(TextStyle::Body, (FontFamily::Proportional, 18.0));
        fonts
            .family_and_size
            .insert(TextStyle::Button, (FontFamily::Proportional, 18.0));

        ctx.set_fonts(fonts);
    }
}

// pub struct GradientButton

pub fn grad_button(text: impl Into<WidgetText>, ui: &mut Ui) -> Response {
    // ui.image(texture_id, size)
    // ui.p
    ui.button(text)
}
