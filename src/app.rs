use egui::{FontData, FontFamily};
use std::sync::Arc;
use tex2typst_rs::text_and_tex2typst;

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct App {
    tex_code: String,
    typst_code: String,
}

impl Default for App {
    fn default() -> Self {
        Self {
            tex_code: r"Here comes some text
\[
    x = \frac{-b \pm \sqrt{b^2 - 4ac}}{2a}
\]"
            .to_owned(),
            typst_code: String::new(),
        }
    }
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // add font
        let mut fonts = egui::FontDefinitions::default();
        fonts.font_data.insert(
            "SC".to_owned(),
            Arc::new(FontData::from_static(include_bytes!(
                "../assets/NotoSansSC-Regular.ttf"
            ))),
        );
        fonts
            .families
            .get_mut(&FontFamily::Proportional)
            .unwrap()
            .insert(1, "SC".to_owned());
        fonts
            .families
            .get_mut(&FontFamily::Monospace)
            .unwrap()
            .insert(1, "SC".to_owned());
        cc.egui_ctx.set_fonts(fonts);

        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        match text_and_tex2typst(&self.tex_code) {
            Ok(typst_code) => {
                self.typst_code = typstyle_core::format_with_width(&typst_code, 80);
            }
            Err(e) => {
                self.typst_code = format!("Error: {}", e);
            }
        }

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::widgets::global_theme_preference_buttons(ui);
        });

        egui::CentralPanel::default().show(ctx, |ui| {

            ui.heading("TeX to Typst Convert Demo");

            ui.label("This is a wrapper of tex2typst-rs compiled to WASM, which means it runs natively in your browser.");

            ui.separator();

            let cols = ui.columns(2, |columns| {
                // 左侧
                columns[0].push_id("left", |ui| {
                    ui.vertical_centered(|ui| {
                        ui.heading("LaTeX Input");
                        ui.add_space(20.0);
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            let mut theme =
                                egui_extras::syntax_highlighting::CodeTheme::from_memory(ui.ctx(), ui.style());
                            ui.collapsing("Theme", |ui| {
                                ui.group(|ui| {
                                    theme.ui(ui);
                                    theme.clone().store_in_memory(ui.ctx());
                                });
                            });

                            let mut layouter = |ui: &egui::Ui, string: &str, wrap_width: f32| {
                                let mut layout_job = egui_extras::syntax_highlighting::highlight(
                                    ui.ctx(),
                                    ui.style(),
                                    &theme,
                                    string,
                                    "tex".into(),
                                );
                                layout_job.wrap.max_width = wrap_width;
                                ui.fonts(|f| f.layout_job(layout_job))
                            };
                            ui.add(
                                egui::TextEdit::multiline(&mut self.tex_code)
                                    .code_editor()
                                    .desired_width(ui.available_width())
                                    .desired_rows(25)
                                    .layouter(&mut layouter)
                            );
                        });
                    })
                });

                // 右侧
                columns[1].push_id("right", |ui| {
                    ui.vertical_centered(|ui| {
                        ui.heading("Typst Output");
                        ui.add_space(20.0);
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            let mut theme =
                                egui_extras::syntax_highlighting::CodeTheme::from_memory(ui.ctx(), ui.style());
                            ui.collapsing("Theme", |ui| {
                                ui.group(|ui| {
                                    theme.ui(ui);
                                    theme.clone().store_in_memory(ui.ctx());
                                });
                            });

                            let mut layouter = |ui: &egui::Ui, string: &str, wrap_width: f32| {
                                let mut layout_job = egui_extras::syntax_highlighting::highlight(
                                    ui.ctx(),
                                    ui.style(),
                                    &theme,
                                    string,
                                    "tex".into(),
                                );
                                layout_job.wrap.max_width = wrap_width;
                                ui.fonts(|f| f.layout_job(layout_job))
                            };
                            ui.add(
                                egui::TextEdit::multiline(&mut self.typst_code)
                                    .code_editor()
                                    .desired_width(ui.available_width())
                                    .desired_rows(25)
                                    .layouter(&mut layouter)
                            );
                        });
                    })
                });
            });
        });

        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.add(egui::github_link_file!(
                    "https://github.com/Unpredictability/tex2typst-UI/blob/main/",
                    "Source code."
                ));

                ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                    powered_by(ui);
                    egui::warn_if_debug_build(ui);
                });
            });
        });
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }
}

fn powered_by(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.label("Powered by ");
        ui.hyperlink_to(
            "tex2typst-rs. ",
            "https://github.com/Unpredictability/tex2typst-rs",
        );
        ui.label("Formatter ");
        ui.hyperlink_to("typstyle.", "https://github.com/Enter-tainer/typstyle");
    });
}
