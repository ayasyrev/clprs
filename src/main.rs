mod error;
mod config;
mod layout;
mod clipboard;

use error::Result;
use config::Config;
use layout::LayoutConverter;
use clipboard::ClipboardManager;

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let config = Config::load()?;
    let converter = LayoutConverter::new(config);
    let mut clipboard = ClipboardManager::new()?;

    let _result = clipboard.process_clipboard(|text| {
        converter.auto_convert(text)
    })?;

    // Silent success as per PRD requirements
    // Only output on error or if explicitly requested
    Ok(())
}
