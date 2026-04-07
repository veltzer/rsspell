use pulldown_cmark::{Parser, Event};
use std::fs;
use std::path::Path;
use walkdir::WalkDir;
use regex::Regex;
use zspell::Dictionary;
use clap::{Parser as ClapParser, Subcommand, CommandFactory};
use clap_complete::{generate, Shell};

#[derive(ClapParser)]
#[command(name = "rsspell")]
#[command(about = "A fast and efficient spell checker", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Scan files for typos
    Scan {
        /// The directory or file to scan
        #[arg(default_value = ".")]
        path: String,
    },
    /// Show version information
    Version,
    /// Generate shell completions
    Complete {
        /// The shell to generate completions for
        shell: Shell,
    },
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Scan { path } => {
            run_scan(path);
        }
        Commands::Version => {
            println!("rsspell {} by {}", env!("CARGO_PKG_VERSION"), env!("CARGO_PKG_AUTHORS"));
            println!("GIT_DESCRIBE: {}", env!("GIT_DESCRIBE"));
            println!("GIT_SHA: {}", env!("GIT_SHA"));
            println!("GIT_BRANCH: {}", env!("GIT_BRANCH"));
            println!("GIT_DIRTY: {}", env!("GIT_DIRTY"));
            println!("RUSTC_SEMVER: {}", env!("RUSTC_SEMVER"));
            println!("RUST_EDITION: {}", env!("RUST_EDITION"));
            println!("BUILD_TIMESTAMP: {}", env!("BUILD_TIMESTAMP"));
        }
        Commands::Complete { shell } => {
            let mut cmd = Cli::command();
            let name = cmd.get_name().to_string();
            generate(*shell, &mut cmd, name, &mut std::io::stdout());
        }
    }
}

fn run_scan(root_path: &str) {
    // 1. Load the Hunspell files into strings
    const AFF_CONTENT: &str = include_str!("../en_US.aff");
    const DIC_CONTENT: &str = include_str!("../en_US.dic");

    // 2. Build the zspell dictionary
    let dict: Dictionary = zspell::builder()
        .config_str(AFF_CONTENT)
        .dict_str(DIC_CONTENT)
        .build()
        .expect("Failed to build dictionary");

    let re = Regex::new(r"[a-zA-Z]+").unwrap();

    println!("Scanning for typos using zspell in: {}\n", root_path);

    for entry in WalkDir::new(root_path).into_iter().filter_map(|e| e.ok()) {
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
