use pulldown_cmark::{Parser, Event};
use std::fs;
use std::path::Path;
use walkdir::WalkDir;
use regex::Regex;
use zspell::Dictionary;

fn main() {
    // 1. Load the Hunspell files into strings
    let aff_content = fs::read_to_string("en_US.aff").expect("Failed to load en_US.aff");
    let dic_content = fs::read_to_string("en_US.dic").expect("Failed to load en_US.dic");

    // 2. Build the zspell dictionary
    let dict: Dictionary = zspell::builder()
        .config_str(&aff_content)
        .dict_str(&dic_content)
        .build()
        .expect("Failed to build dictionary");

    let re = Regex::new(r"[a-zA-Z]+").unwrap();

    println!("Scanning for typos using zspell...\n");

    for entry in WalkDir::new(".").into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file() {
            match path.extension().and_then(|s| s.to_str()) {
                Some("md") => check_markdown(path, &dict, &re),
                Some("svg") => check_svg(path, &dict, &re),
                _ => {}
            }
        }
    }
}

fn check_markdown(path: &Path, dict: &Dictionary, re: &Regex) {
    let content = fs::read_to_string(path).expect("Could not read file");
    let parser = Parser::new(&content);

    println!("Checking Markdown: {}", path.display());
    for event in parser {
        if let Event::Text(text) = event {
            find_typos(&text, dict, re);
        }
    }
    println!();
}

fn check_svg(path: &Path, dict: &Dictionary, re: &Regex) {
    let content = fs::read_to_string(path).expect("Could not read file");
    println!("Checking SVG: {}", path.display());

    let mut parser = svg::Parser::new(&content);
    while let Some(event) = parser.next() {
        match event {
            svg::parser::Event::Text(text) => {
                find_typos(&text, dict, re);
            }
            _ => {}
        }
    }
    println!();
}

fn find_typos(text: &str, dict: &Dictionary, re: &Regex) {
    for mat in re.find_iter(text) {
        let word = mat.as_str();
        if !dict.check_word(word) {
            println!("  -> Typo found: \"{}\"", word);
        }
    }
}
