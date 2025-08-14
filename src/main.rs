mod error;
mod config;
mod layout;
mod clipboard;

use clap::{Arg, ArgAction, Command};
use std::io::{self, Write};
use error::Result;
use config::Config;
use layout::LayoutConverter;
use clipboard::ClipboardManager;

fn main() {
    let matches = Command::new("clprs")
        .about("Clipboard layout correction tool")
        .long_about("Clprs (Clipper) reads text from clipboard, detects keyboard layout issues, and corrects them automatically. Primarily designed for Russian ↔ English layout conversion.")
        .version("0.1.0")
        .author("Clprs Development Team")
        .arg(
            Arg::new("dry-run")
                .short('d')
                .long("dry-run")
                .action(ArgAction::SetTrue)
                .help("Show clipboard content and proposed conversion without modifying clipboard")
        )
        .arg(
            Arg::new("interactive")
                .short('i')
                .long("interactive")
                .action(ArgAction::SetTrue)
                .help("Ask for confirmation before applying conversion")
        )
        .arg(
            Arg::new("undo")
                .short('u')
                .long("undo")
                .action(ArgAction::SetTrue)
                .help("Restore previous clipboard content")
        )
        .get_matches();

    let dry_run = matches.get_flag("dry-run");
    let interactive = matches.get_flag("interactive");
    let undo = matches.get_flag("undo");

    if let Err(e) = run(dry_run, interactive, undo) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn run(dry_run: bool, interactive: bool, undo: bool) -> Result<()> {
    let config = Config::load()?;
    let converter = LayoutConverter::new(config);
    let mut clipboard = ClipboardManager::new()?;

    if undo {
        run_undo_mode(&mut clipboard)
    } else if dry_run {
        run_dry_mode(&converter, &mut clipboard)
    } else if interactive {
        run_interactive_mode(&converter, &mut clipboard)
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

fn run_interactive_mode(converter: &LayoutConverter, clipboard: &mut ClipboardManager) -> Result<()> {
    match clipboard.get_text() {
        Ok(original) => {
            let converted = converter.auto_convert(&original)?;
            
            if original == converted {
                println!("No conversion needed - text is already in correct layout");
                println!("Current clipboard content:");
                println!("{}", original);
                return Ok(());
            }
            
            // Show the proposed change
            println!("Current clipboard content:");
            println!("{}", original);
            println!("\nProposed conversion:");
            println!("{}", converted);
            
            // Ask for confirmation
            if confirm_conversion()? {
                clipboard.set_text_with_backup(&original, &converted)?;
                let original_preview = truncate_string(&original, 10);
                let converted_preview = truncate_string(&converted, 10);
                println!("done: \"{}\" --> \"{}\"", original_preview, converted_preview);
            } else {
                println!("Conversion cancelled - clipboard unchanged");
            }
        }
        Err(crate::error::ClprsError::EmptyClipboard) => {
            println!("Empty clipboard");
        }
        Err(e) => return Err(e),
    }

    Ok(())
}

fn confirm_conversion() -> Result<bool> {
    loop {
        print!("\nApply this conversion? (y/n): ");
        io::stdout().flush().unwrap();
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        
        match input.trim().to_lowercase().as_str() {
            "y" | "yes" => return Ok(true),
            "n" | "no" => return Ok(false),
            _ => {
                println!("Please enter 'y' for yes or 'n' for no.");
                continue;
            }
        }
    }
}

fn run_undo_mode(clipboard: &mut ClipboardManager) -> Result<()> {
    if clipboard.has_previous() {
        let restored_content = clipboard.get_previous_content();
        clipboard.restore_previous()?;
        
        if let Some(content) = restored_content {
            let preview = truncate_string(&content, 10);
            println!("Previous clipboard content restored: \"{}\"", preview);
        } else {
            println!("Previous clipboard content restored");
        }
    } else {
        println!("No previous clipboard content to restore");
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
