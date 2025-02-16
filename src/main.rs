use regex::Regex;
use tex2typst_rs::command_registry::{parse_custom_macros, CommandRegistry};
use tex2typst_rs::converter::convert_tree;
use tex2typst_rs::tex_parser::LatexParser;
use tex2typst_rs::tex_tokenizer::tokenize;
use tex2typst_rs::typst_writer::TypstWriter;
use typstyle_core;
use wasm_bindgen::prelude::*;

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
        Ok(typstyle_core::format_with_width(&new, 80))
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

    #[wasm_bindgen]
    pub fn register_macros(&mut self, macros: &str) -> Result<usize, String> {
        self.command_registry.custom_macros.clear();
        self.command_registry.custom_macro_names.clear();
        self.command_registry
            .register_custom_macros(parse_custom_macros(macros)?);
        Ok(self.command_registry.custom_macros.len())
    }
}

fn main() {}
