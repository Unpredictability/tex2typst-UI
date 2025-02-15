use regex::Regex;
use tex2typst_rs::command_registry::{parse_custom_macros, CommandRegistry};
use tex2typst_rs::converter::convert_tree;
use tex2typst_rs::tex_parser::LatexParser;
use tex2typst_rs::tex_tokenizer::tokenize;
use tex2typst_rs::typst_writer::TypstWriter;
use wasm_bindgen::prelude::*;
use web_sys::{console, window, Document, Event, HtmlTextAreaElement};

#[wasm_bindgen]
struct Worker {
    tex_parser: LatexParser,
    command_registry: CommandRegistry,
    typst_writer: TypstWriter,
    regex: Regex,
}

#[wasm_bindgen]
impl Worker {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        let tex_parser = LatexParser::new(false, false);
        let command_registry = CommandRegistry::new();
        let typst_writer = TypstWriter::new();
        let regex = Regex::new(r"\\\((.+?)\\\)|(?s)\\\[(.+?)\\\]").unwrap();
        Worker {
            tex_parser,
            command_registry,
            typst_writer,
            regex,
        }
    }

    #[wasm_bindgen]
    pub fn work(&mut self, tex: &str) -> Result<String, String> {
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

fn main() {
    console_error_panic_hook::set_once();

    let document = window()
        .and_then(|win| win.document())
        .expect("Could not access the document");
    let body = document.body().expect("Could not access document.body");
    let text_node = document.create_text_node("Hello, world from Vanilla Rust!");
    body.append_child(text_node.as_ref())
        .expect("Failed to append text");
}

#[wasm_bindgen]
pub fn log_message() {
    console::log_1(&"Hello from Rust!".into());
}

const DEFAULT_TEX_CODE: &str = r"Here comes some text
\[
    x = \frac{-b \pm \sqrt{b^2 - 4ac}}{2a}
\]

The following use some custom macros (see below)
\(\R\)

\(\Arg \Log \Int \ball \disk\)";
const DEFAULT_CUSTOM_MACROS: &str = r"\newcommand{\N}{\mathbb{N}}
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
