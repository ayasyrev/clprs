use crate::config::Config;
use crate::error::Result;

#[derive(Debug, Clone, PartialEq)]
pub enum Layout {
    Russian,
    English,
    Mixed,
    Unknown,
}

pub struct LayoutConverter {
    config: Config,
}

impl LayoutConverter {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    pub fn convert_text(&self, text: &str, from_layout: Layout, to_layout: Layout) -> Result<String> {
        let mapping_key = match (from_layout, to_layout) {
            (Layout::Russian, Layout::English) => "ru_to_en",
            (Layout::English, Layout::Russian) => "en_to_ru",
            _ => return Ok(text.to_string()), // No conversion needed
        };

        if let Some(layout_mapping) = self.config.layout_mappings.get(mapping_key) {
            let converted = text.chars()
                .map(|c| {
                    layout_mapping.char_map.get(&c.to_string())
                        .map(|s| s.chars().next().unwrap_or(c))
                        .unwrap_or(c)
                })
                .collect();
            Ok(converted)
        } else {
            Ok(text.to_string())
        }
    }

    pub fn auto_convert(&self, text: &str) -> Result<String> {
        let detected_layout = self.detect_layout(text);
        
        match detected_layout {
            Layout::Russian => self.convert_text(text, Layout::Russian, Layout::English),
            Layout::English => {
                // Check if this might be Russian typed on English layout
                let ru_converted = self.convert_text(text, Layout::English, Layout::Russian)?;
                if self.is_more_likely_russian(&ru_converted) {
                    Ok(ru_converted)
                } else {
                    // No conversion needed - return original text
                    Ok(text.to_string())
                }
            }
            _ => {
                // For unknown/mixed layouts, still try English->Russian conversion
                // as it might be Latin chars representing Russian words
                if self.is_likely_latin_representing_russian(text) {
                    let ru_converted = self.convert_text(text, Layout::English, Layout::Russian)?;
                    if self.is_more_likely_russian(&ru_converted) {
                        Ok(ru_converted)
                    } else {
                        Ok(text.to_string())
                    }
                } else {
                    Ok(text.to_string())
                }
            }
        }
    }

    pub fn detect_layout(&self, text: &str) -> Layout {
        let mut ru_chars = 0;
        let mut en_chars = 0;
        let mut total_chars = 0;

        for c in text.chars() {
            if c.is_alphabetic() {
                total_chars += 1;
                if self.is_cyrillic(c) {
                    ru_chars += 1;
                } else if c.is_ascii_alphabetic() {
                    en_chars += 1;
                }
            }
        }

        if total_chars == 0 {
            return Layout::Unknown;
        }

        let ru_ratio = ru_chars as f32 / total_chars as f32;
        let en_ratio = en_chars as f32 / total_chars as f32;

        if ru_ratio > 0.7 {
            Layout::Russian
        } else if en_ratio > 0.7 {
            Layout::English
        } else if ru_ratio > 0.0 && en_ratio > 0.0 {
            Layout::Mixed
        } else {
            Layout::Unknown
        }
    }

    fn is_cyrillic(&self, c: char) -> bool {
        matches!(c as u32, 0x0400..=0x04FF)
    }

    fn is_more_likely_russian(&self, text: &str) -> bool {
        // Simple heuristic: check for common Russian letter patterns
        let russian_patterns = ["ый", "ий", "ой", "ей", "ах", "ов", "ин", "ен", "ан"];
        let mut pattern_count = 0;
        
        for pattern in &russian_patterns {
            if text.contains(pattern) {
                pattern_count += 1;
            }
        }
        
        // Also check for common Russian words
        let common_words = ["что", "это", "как", "все", "для", "или", "его", "она", "они", "привет", "пока", "спасибо"];
        for word in &common_words {
            if text.contains(word) {
                pattern_count += 2; // Weight common words more heavily
            }
        }
        
        pattern_count > 0
    }

    fn is_likely_latin_representing_russian(&self, text: &str) -> bool {
        // Check if the text contains only ASCII characters that could represent Russian
        let mut ascii_count = 0;
        let mut total_alpha = 0;
        
        for c in text.chars() {
            if c.is_alphabetic() {
                total_alpha += 1;
                if c.is_ascii_alphabetic() {
                    ascii_count += 1;
                }
            }
        }
        
        // If most alphabetic characters are ASCII, it might be Latin representing Russian
        if total_alpha > 0 && (ascii_count as f32 / total_alpha as f32) >= 0.8 {
            // Additional heuristic: check for patterns that commonly occur 
            // when typing Russian words on English layout
            let latin_russian_patterns = ["gh", "tn", "yf", "jk", "hj", "yr"];
            for pattern in &latin_russian_patterns {
                if text.contains(pattern) {
                    return true;
                }
            }
            
            // Also try converting and see if it produces recognizable patterns
            if let Ok(converted) = self.convert_text(text, Layout::English, Layout::Russian) {
                if self.contains_cyrillic(&converted) {
                    return true;
                }
            }
        }
        
        false
    }

    fn contains_cyrillic(&self, text: &str) -> bool {
        text.chars().any(|c| self.is_cyrillic(c))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layout_detection() {
        let config = Config::default();
        let converter = LayoutConverter::new(config);

        assert_eq!(converter.detect_layout("hello world"), Layout::English);
        assert_eq!(converter.detect_layout("привет мир"), Layout::Russian);
        assert_eq!(converter.detect_layout("hello мир"), Layout::Mixed);
        assert_eq!(converter.detect_layout("123!@#"), Layout::Unknown);
    }

    #[test]
    fn test_russian_to_english_conversion() {
        let config = Config::default();
        let converter = LayoutConverter::new(config);
        
        let result = converter.convert_text("руддщ", Layout::Russian, Layout::English).unwrap();
        assert_eq!(result, "hello");
    }

    #[test]
    fn test_english_to_russian_conversion() {
        let config = Config::default();
        let converter = LayoutConverter::new(config);
        
        let result = converter.convert_text("hello", Layout::English, Layout::Russian).unwrap();
        assert_eq!(result, "руддщ");
    }

    #[test] 
    fn test_auto_convert_latin_to_russian() {
        let config = Config::default();
        let converter = LayoutConverter::new(config);
        
        // Test common Russian words typed in Latin layout
        let result = converter.auto_convert("ghbdtn").unwrap();
        assert_eq!(result, "привет");
        
        let result = converter.auto_convert("gjrf").unwrap();
        assert_eq!(result, "пока");
        
        let result = converter.auto_convert("ghbdtn vbh").unwrap();
        assert_eq!(result, "привет мир");
    }

    #[test]
    fn test_auto_convert_russian_to_latin() {
        let config = Config::default();
        let converter = LayoutConverter::new(config);
        
        // Test Russian text conversion to Latin
        let result = converter.auto_convert("привет").unwrap();
        assert_eq!(result, "ghbdtn");
        
        let result = converter.auto_convert("привет мир").unwrap();
        assert_eq!(result, "ghbdtn vbh");
    }

    #[test]
    fn test_no_conversion_for_normal_english() {
        let config = Config::default();
        let converter = LayoutConverter::new(config);
        
        // Normal English words should not be converted
        let result = converter.auto_convert("hello world").unwrap();
        assert_eq!(result, "hello world");
        
        let result = converter.auto_convert("test").unwrap();
        assert_eq!(result, "test");
    }
}