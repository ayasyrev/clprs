use arboard::Clipboard;
use crate::error::{ClprsError, Result};
use std::fs;
use std::path::PathBuf;

pub struct ClipboardManager {
    clipboard: Clipboard,
    previous_content: Option<String>,
    backup_file: PathBuf,
}

impl ClipboardManager {
    pub fn new() -> Result<Self> {
        let clipboard = Clipboard::new()?;
        
        // Create backup file path in system temp directory
        let mut backup_file = std::env::temp_dir();
        backup_file.push("clprs_backup.txt");
        
        // Load previous content if backup file exists
        let previous_content = if backup_file.exists() {
            match fs::read_to_string(&backup_file) {
                Ok(content) if !content.trim().is_empty() => Some(content),
                _ => None,
            }
        } else {
            None
        };
        
        Ok(Self { 
            clipboard,
            previous_content,
            backup_file,
        })
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


    pub fn process_clipboard_with_original<F>(&mut self, processor: F) -> Result<(String, String)>
    where
        F: FnOnce(&str) -> Result<String>,
    {
        let original_text = self.get_text()?;
        let processed_text = processor(&original_text)?;
        
        // Only update clipboard if text actually changed
        if processed_text != original_text {
            // Backup the original text before replacing
            self.save_backup(&original_text)?;
            self.clipboard.set_text(&processed_text)?;
            
            // Keep clipboard alive longer on Linux
            #[cfg(target_os = "linux")]
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
        
        Ok((original_text, processed_text))
    }

    pub fn restore_previous(&mut self) -> Result<()> {
        match &self.previous_content {
            Some(content) => {
                // Don't backup when restoring to avoid infinite undo chain
                self.clipboard.set_text(content)?;
                
                // Keep clipboard alive longer on Linux
                #[cfg(target_os = "linux")]
                std::thread::sleep(std::time::Duration::from_millis(50));
                
                // Clear the previous content and remove backup file
                self.previous_content = None;
                let _ = fs::remove_file(&self.backup_file);
                
                Ok(())
            }
            None => Err(ClprsError::NoPreviousContent),
        }
    }

    pub fn has_previous(&self) -> bool {
        self.previous_content.is_some()
    }

    pub fn get_previous_content(&self) -> Option<String> {
        self.previous_content.clone()
    }

    pub fn set_text_with_backup(&mut self, original: &str, new_text: &str) -> Result<()> {
        // Backup the original text before replacing
        self.save_backup(original)?;
        
        self.clipboard.set_text(new_text)?;
        
        // Keep clipboard alive longer on Linux
        #[cfg(target_os = "linux")]
        std::thread::sleep(std::time::Duration::from_millis(50));
        
        Ok(())
    }

    fn save_backup(&mut self, content: &str) -> Result<()> {
        if !content.trim().is_empty() {
            self.previous_content = Some(content.to_string());
            // Also save to file for persistence between runs
            fs::write(&self.backup_file, content).map_err(|e| ClprsError::IoError(e))?;
        }
        Ok(())
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