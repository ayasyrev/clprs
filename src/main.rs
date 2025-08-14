mod error;
mod config;
mod layout;
mod clipboard;

use clap::Command;
use error::Result;
use config::Config;
use layout::LayoutConverter;
use clipboard::ClipboardManager;

fn main() {
    let _matches = Command::new("clprs")
        .about("Clipboard layout correction tool")
        .long_about("Clprs (Clipper) reads text from clipboard, detects keyboard layout issues, and corrects them automatically. Primarily designed for Russian ↔ English layout conversion.")
        .version("0.1.0")
        .author("Clprs Development Team")
        .get_matches();

    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let config = Config::load()?;
    let converter = LayoutConverter::new(config);
    let mut clipboard = ClipboardManager::new()?;

    match clipboard.process_clipboard_with_original(|text| {
        converter.auto_convert(text)
    }) {
        Ok((original, converted)) => {
            // Format output showing first 10 characters of original and result
            let original_preview = truncate_string(&original, 10);
            let converted_preview = truncate_string(&converted, 10);
            
            if original != converted {
                println!("done: \"{}\" --> \"{}\"", original_preview, converted_preview);
            } else {
                println!("done: \"{}\" --> \"{}\"", original_preview, converted_preview);
            }
        }
        Err(crate::error::ClprsError::EmptyClipboard) => {
            println!("Empty clipboard");
        }
        Err(e) => return Err(e),
    }

    Ok(())
}

fn truncate_string(s: &str, max_chars: usize) -> String {
    if s.chars().count() <= max_chars {
        s.to_string()
    } else {
        s.chars().take(max_chars).collect()
    }
}
