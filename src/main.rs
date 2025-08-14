mod error;
mod config;
mod layout;
mod clipboard;

use clap::{Arg, ArgAction, Command};
use error::Result;
use config::Config;
use layout::LayoutConverter;
use clipboard::ClipboardManager;

fn main() {
    let matches = Command::new("clprs")
        .about("Clipboard layout correction tool")
        .long_about("Clprs (Clipper) reads text from clipboard, detects keyboard layout issues, and corrects them automatically. Primarily designed for Russian ↔ English layout conversion.")
        .version("0.1.1")
        .author("Clprs Development Team")
        .arg(
            Arg::new("dry-run")
                .short('d')
                .long("dry-run")
                .action(ArgAction::SetTrue)
                .help("Show clipboard content and proposed conversion without modifying clipboard")
        )
        .get_matches();

    let dry_run = matches.get_flag("dry-run");

    if let Err(e) = run(dry_run) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn run(dry_run: bool) -> Result<()> {
    let config = Config::load()?;
    let converter = LayoutConverter::new(config);
    let mut clipboard = ClipboardManager::new()?;

    if dry_run {
        run_dry_mode(&converter, &mut clipboard)
    } else {
        run_normal_mode(&converter, &mut clipboard)
    }
}

fn run_normal_mode(converter: &LayoutConverter, clipboard: &mut ClipboardManager) -> Result<()> {
    match clipboard.process_clipboard_with_original(|text| {
        converter.auto_convert(text)
    }) {
        Ok((original, converted)) => {
            // Format output showing first 10 characters of original and result
            let original_preview = truncate_string(&original, 10);
            let converted_preview = truncate_string(&converted, 10);
            
            println!("done: \"{}\" --> \"{}\"", original_preview, converted_preview);
        }
        Err(crate::error::ClprsError::EmptyClipboard) => {
            println!("Empty clipboard");
        }
        Err(e) => return Err(e),
    }

    Ok(())
}

fn run_dry_mode(converter: &LayoutConverter, clipboard: &mut ClipboardManager) -> Result<()> {
    match clipboard.get_text() {
        Ok(original) => {
            let converted = converter.auto_convert(&original)?;
            
            println!("Clipboard content:");
            println!("{}", original);
            println!("\nProposed conversion:");
            println!("{}", converted);
            
            if original == converted {
                println!("\nNo conversion needed - text is already in correct layout");
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
