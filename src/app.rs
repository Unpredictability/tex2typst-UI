use egui::{FontData, FontFamily};
use egui_notify::Toasts;
use regex::{Captures, Regex};
use std::sync::Arc;
use std::time::Duration;
use tex2typst_rs::command_registry::{parse_custom_macros, CommandRegistry};
use tex2typst_rs::converter::convert_tree;
use tex2typst_rs::tex_parser::LatexParser;
use tex2typst_rs::tex_tokenizer::tokenize;
use tex2typst_rs::typst_writer::TypstWriter;
use tex2typst_rs::{
    converter, replace_all, tex2typst, tex_tokenizer, text_and_tex2typst, typst_writer,
};
use typstyle_core;
use web_sys::js_sys::Atomics::store;

struct Worker {
    tex_parser: LatexParser,
    command_registry: CommandRegistry,
    typst_writer: TypstWriter,
    regex: Regex,
}

impl Worker {
    fn work(&mut self, tex: &str) -> Result<String, String> {
        let mut new = String::with_capacity(tex.len());
        let mut last_match = 0;
        let captures: Vec<_> = self.regex.captures_iter(tex).collect();
        for caps in captures {
            let m = caps.get(0).unwrap();
            new.push_str(&tex[last_match..m.start()]);
            let t = if let Some(inline_math) = caps.get(1) {
                let typst_math = self.convert_math(inline_math.as_str().trim())?;
                format!("${}$", typst_math)
            } else if let Some(display_math) = caps.get(2) {
                let typst_math = self.convert_math(display_math.as_str().trim())?;
                format!("$\n{}\n$", typst_math)
            } else {
                caps[0].to_string()
            };
            new.push_str(t.as_str());
            last_match = m.end();
        }
        new.push_str(&tex[last_match..]);
        Ok(new)
    }

    fn convert_math(&mut self, tex: &str) -> Result<String, String> {
        let tokens = tokenize(tex)?;
        let expanded_tokens = self.command_registry.expand_macros(&tokens)?;

        let tex_tree = self.tex_parser.parse(expanded_tokens)?;
        let typst_tree = convert_tree(&tex_tree)?;

        let mut writer = TypstWriter::new();
        writer.serialize(&typst_tree)?;
        let typst = writer.finalize()?;
        Ok(typst)
    }

    fn register_macros(&mut self, macros: &str) -> Result<usize, String> {
        self.command_registry.custom_macros.clear();
        self.command_registry.custom_macro_names.clear();
        self.command_registry
            .register_custom_macros(parse_custom_macros(macros)?);
        Ok(self.command_registry.custom_macros.len())
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct App {
    tex_code: String,
    typst_code: String,
    custom_macros: String,
    #[serde(skip)]
    worker: Worker,
    #[serde(skip)]
    toasts: Toasts,
}

impl Default for App {
    fn default() -> Self {
        let default_tex_code = r"Here comes some text
\[
    x = \frac{-b \pm \sqrt{b^2 - 4ac}}{2a}
\]

The following use some custom macros (see below)
\(\R\)

\(\Arg \Log \Int \ball \disk\)";
        let default_custom_macros = r"\newcommand{\N}{\mathbb{N}}
\newcommand{\Z}{\mathbb{Z}}
\newcommand{\Q}{\mathbb{Q}}
\newcommand{\R}{\mathbb{R}}
\newcommand{\CC}{\mathbb{C}}
\newcommand{\HH}{\mathbb{H}}
\newcommand{\T}{\mathbb{T}}
\newcommand{\Arg}{\operatorname{Arg}}
\newcommand{\Log}{\operatorname{Log}}
\newcommand{\ball}{\mathbb{B}}
\newcommand{\disk}{\mathbb{D}}
\newcommand{\Int}{\operatorname{Int}}
";

        let mut command_registry = CommandRegistry::new();
        command_registry
            .register_custom_macros(parse_custom_macros(default_custom_macros).unwrap());

        let worker = Worker {
            tex_parser: LatexParser::new(false, false),
            command_registry,
            typst_writer: TypstWriter::new(),
            regex: Regex::new(r"\\\((.+?)\\\)|(?s)\\\[(.+?)\\\]").unwrap(),
        };

        Self {
            tex_code: default_tex_code.to_owned(),
            typst_code: String::new(),
            custom_macros: default_custom_macros.to_owned(),
            worker,
            toasts: Toasts::default(),
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
            let mut app: Self = eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
            app.worker.command_registry
                .register_custom_macros(parse_custom_macros(&app.custom_macros).unwrap());
        }


        Self::default()
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        match self.worker.work(&self.tex_code) {
            Ok(typst_code) => {
                self.typst_code = typstyle_core::format_with_width(&typst_code, 80);
            }
            Err(e) => {
                self.typst_code = format!("Error: {}", e);
            }
        }

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                egui::widgets::global_theme_preference_buttons(ui);
                ui.add(egui::github_link_file!(
                    "https://github.com/Unpredictability/tex2typst-UI/blob/main/",
                    "> Source code."
                ));
                ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                    powered_by(ui);
                    egui::warn_if_debug_build(ui);
                });
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.heading("TeX to Typst Convert Demo");

                ui.label("This is a wrapper of tex2typst-rs compiled to WASM, which means it runs natively in your browser.");

                ui.separator();

                ui.columns(2, |columns| {
                    // Left: LaTeX Input
                    columns[0].push_id("latex_input", |ui| {
                        ui.vertical_centered(|ui| {
                            ui.heading("LaTeX Input");
                            ui.add_space(10.0);
                            egui::ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
                                ui.add(
                                    egui::TextEdit::multiline(&mut self.tex_code)
                                        .code_editor()
                                        .desired_width(ui.available_width())
                                        .desired_rows(15)
                                );
                            });
                        });
                    });
                    // Right: Typst Output
                    columns[1].push_id("typst_output", |ui| {
                        ui.vertical_centered(|ui| {
                            ui.heading("Typst Output");
                            ui.add_space(10.0);
                            egui::ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
                                ui.add(
                                    egui::TextEdit::multiline(&mut self.typst_code)
                                        .code_editor()
                                        .desired_width(ui.available_width())
                                        .desired_rows(15)
                                );
                            });
                        });
                    });
                });

                ui.add_space(10.0);

                ui.columns(2, |columns| {
                    columns[0].push_id("custom_macros", |ui| {
                        ui.collapsing("Custom Macros", |ui| {
                            ui.add(
                                egui::TextEdit::multiline(&mut self.custom_macros)
                                    .code_editor()
                                    .desired_width(ui.available_width())
                                    .desired_rows(5)
                            );
                            if ui.button("register macros").clicked() {
                                match self.worker.register_macros(&self.custom_macros) {
                                    Ok(num) => {
                                        self.toasts.info(format!("Custom macros ({num} in total) registered")).duration(Some(Duration::from_secs(3)));
                                    }
                                    Err(e) => {
                                        self.toasts.warning(e).duration(Some(Duration::from_secs(5)));
                                    }
                                }
                            }
                        });
                    });
                });
            });
        });
        self.toasts.show(ctx);
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
