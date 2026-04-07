use pulldown_cmark::{Parser, Event};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use regex::Regex;
use zspell::Dictionary;
use clap::{Parser as ClapParser, Subcommand, CommandFactory};
use clap_complete::{generate, Shell};
use anyhow::{Context, Result, anyhow};

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
        /// The language to use (e.g., en-US, de-DE)
        #[arg(short, long, default_value = "en-US")]
        lang: String,
    },
    /// Show version information
    Version,
    /// Generate shell completions
    Complete {
        /// The shell to generate completions for
        shell: Shell,
    },
    /// Manage dictionaries
    Dicts {
        #[command(subcommand)]
        action: DictAction,
    },
}

#[derive(Subcommand)]
enum DictAction {
    /// List installed dictionaries
    List,
    /// List dictionaries available for download
    ListRemote,
    /// Install a new dictionary (e.g., en-US, de-DE)
    Install {
        /// The language code to install
        lang: String,
    },
    /// Show the path to the dictionaries directory
    Path,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Scan { path, lang } => {
            run_scan(path, lang)?;
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
        Commands::Dicts { action } => match action {
            DictAction::List => list_dicts()?,
            DictAction::ListRemote => list_remote_dicts()?,
            DictAction::Install { lang } => install_dict(lang)?,
            DictAction::Path => println!("{}", get_dict_dir()?.display()),
        },
    }

    Ok(())
}

fn list_remote_dicts() -> Result<()> {
    let url = "https://api.github.com/repos/wooorm/dictionaries/contents/dictionaries";
    let client = reqwest::blocking::Client::builder()
        .user_agent("rsspell")
        .build()?;
    
    println!("Fetching available dictionaries from wooorm/dictionaries...");
    
    let resp = client.get(url).send()?.error_for_status()?;
    let contents: Vec<serde_json::Value> = resp.json()?;
    
    let mut langs = Vec::new();
    for item in contents {
        if item["type"] == "dir" {
            if let Some(name) = item["name"].as_str() {
                langs.push(name.to_string());
            }
        }
    }
    
    langs.sort();
    println!("Available languages:");
    for lang in langs {
        println!("  - {}", lang);
    }
    println!("\nInstall any of these with: rsspell dicts install <lang>");
    
    Ok(())
}

fn get_dict_dir() -> Result<PathBuf> {
    let mut path = dirs::data_local_dir()
        .context("Could not find local data directory")?;
    path.push("rsspell");
    path.push("dicts");
    if !path.exists() {
        fs::create_dir_all(&path).context("Failed to create dictionary directory")?;
    }
    Ok(path)
}

fn list_dicts() -> Result<()> {
    let dict_dir = get_dict_dir()?;
    println!("Dictionaries stored in: {}", dict_dir.display());
    println!("Installed languages:");
    
    let mut langs = Vec::new();
    for entry in fs::read_dir(dict_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("aff") {
            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                langs.push(stem.to_string());
            }
        }
    }
    
    if langs.is_empty() {
        println!("  (none)");
    } else {
        langs.sort();
        for lang in langs {
            println!("  - {}", lang);
        }
    }
    println!("  - en-US (embedded fallback)");
    Ok(())
}

fn install_dict(lang: &str) -> Result<()> {
    let dict_dir = get_dict_dir()?;
    // Normalize lang for the source (wooorm uses dashes like en-US)
    let lang_normalized = lang.replace('_', "-");
    
    let aff_url = format!("https://raw.githubusercontent.com/wooorm/dictionaries/main/dictionaries/{}/index.aff", lang_normalized);
    let dic_url = format!("https://raw.githubusercontent.com/wooorm/dictionaries/main/dictionaries/{}/index.dic", lang_normalized);

    println!("Downloading dictionary for {}...", lang_normalized);
    
    let aff_content = reqwest::blocking::get(&aff_url)
        .context("Failed to download .aff file")?
        .error_for_status()
        .context("Server returned error for .aff file. Check if language exists at https://github.com/wooorm/dictionaries")?
        .text()?;
        
    let dic_content = reqwest::blocking::get(&dic_url)
        .context("Failed to download .dic file")?
        .error_for_status()
        .context("Server returned error for .dic file")?
        .text()?;

    fs::write(dict_dir.join(format!("{}.aff", lang_normalized)), aff_content)?;
    fs::write(dict_dir.join(format!("{}.dic", lang_normalized)), dic_content)?;

    println!("Successfully installed {} dictionary.", lang_normalized);
    Ok(())
}

fn load_dictionary(lang: &str) -> Result<Dictionary> {
    let dict_dir = get_dict_dir()?;
    let lang_normalized = lang.replace('_', "-");
    let aff_path = dict_dir.join(format!("{}.aff", lang_normalized));
    let dic_path = dict_dir.join(format!("{}.dic", lang_normalized));

    if aff_path.exists() && dic_path.exists() {
        let aff_content = fs::read_to_string(aff_path)?;
        let dic_content = fs::read_to_string(dic_path)?;
        return zspell::builder()
            .config_str(&aff_content)
            .dict_str(&dic_content)
            .build()
            .map_err(|e| anyhow!("Failed to build dictionary: {}", e));
    }

    if lang_normalized == "en-US" || lang_normalized == "en" {
        const AFF_CONTENT: &str = include_str!("../dictionaries/en_US.aff");
        const DIC_CONTENT: &str = include_str!("../dictionaries/en_US.dic");
        return zspell::builder()
            .config_str(AFF_CONTENT)
            .dict_str(DIC_CONTENT)
            .build()
            .map_err(|e| anyhow!("Failed to build embedded dictionary: {}", e));
    }

    Err(anyhow!("Dictionary for '{}' not found. Install it with: rsspell dicts install {}", lang_normalized, lang_normalized))
}

fn run_scan(root_path: &str, lang: &str) -> Result<()> {
    let dict = load_dictionary(lang)?;
    let re = Regex::new(r"[a-zA-Z]+").unwrap();

    println!("Scanning for typos using zspell (lang: {}) in: {}\n", lang, root_path);

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
    Ok(())
}

fn check_markdown(path: &Path, dict: &Dictionary, re: &Regex) {
    let content = fs::read_to_string(path).expect("Could not read file");
    let parser = Parser::new(&content);

    println!("Checking Markdown: {}", path.display());
    for event in parser {
        if let Event::Text(text) = event {
            let _ = find_typos(&text, dict, re);
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
                let _ = find_typos(&text, dict, re);
            }
            _ => {}
        }
    }
    println!();
}

fn find_typos(text: &str, dict: &Dictionary, re: &Regex) -> Vec<String> {
    let mut typos = Vec::new();
    for mat in re.find_iter(text) {
        let word = mat.as_str();
        if !dict.check_word(word) {
            println!("  -> Typo found: \"{}\"", word);
            typos.push(word.to_string());
        }
    }
    typos
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_typos() {
        let aff_content = include_str!("../dictionaries/en_US.aff");
        let dic_content = include_str!("../dictionaries/en_US.dic");
        let dict = zspell::builder()
            .config_str(aff_content)
            .dict_str(dic_content)
            .build()
            .unwrap();
        let re = Regex::new(r"[a-zA-Z]+").unwrap();
        
        let typos = find_typos("This is a test with a typo: markdonw", &dict, &re);
        assert_eq!(typos, vec!["markdonw"]);
    }
}
