use arboard::Clipboard;
use crate::error::{ClprsError, Result};

pub struct ClipboardManager {
    clipboard: Clipboard,
}

impl ClipboardManager {
    pub fn new() -> Result<Self> {
        let clipboard = Clipboard::new()?;
        Ok(Self { clipboard })
    }

    pub fn get_text(&mut self) -> Result<String> {
        match self.clipboard.get_text() {
            Ok(text) => {
                if text.trim().is_empty() {
                    Err(ClprsError::EmptyClipboard)
                } else {
                    Ok(text)
                }
            }
            Err(e) => Err(ClprsError::ClipboardError(e)),
        }
    }

    pub fn set_text(&mut self, text: &str) -> Result<()> {
        self.clipboard.set_text(text)?;
        
        // Keep clipboard alive longer on Linux to ensure clipboard managers see the contents
        #[cfg(target_os = "linux")]
        std::thread::sleep(std::time::Duration::from_millis(50));
        
        Ok(())
    }

    pub fn process_clipboard_with_original<F>(&mut self, processor: F) -> Result<(String, String)>
    where
        F: FnOnce(&str) -> Result<String>,
    {
        let original_text = self.get_text()?;
        let processed_text = processor(&original_text)?;
        
        // Only update clipboard if text actually changed
        if processed_text != original_text {
            self.set_text(&processed_text)?;
        }
        
        Ok((original_text, processed_text))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clipboard_creation() {
        // This test might fail in headless environments
        match ClipboardManager::new() {
            Ok(_) => {},
            Err(ClprsError::ClipboardError(_)) => {
                // Expected in CI/headless environments
                println!("Clipboard test skipped in headless environment");
            },
            Err(e) => panic!("Unexpected error: {}", e),
        }
    }
}