use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use crate::error::{ClprsError, Result};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub default_from_layout: String,
    pub default_to_layout: String,
    pub layout_mappings: HashMap<String, LayoutMapping>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LayoutMapping {
    pub name: String,
    pub char_map: HashMap<char, char>,
}

impl Default for Config {
    fn default() -> Self {
        let mut layout_mappings = HashMap::new();
        
        // Default Russian -> English mapping
        let mut ru_en_map = HashMap::new();
        let ru_chars = "йцукенгшщзхъфывапролджэячсмитьбюё";
        let en_chars = "qwertyuiop[]asdfghjkl;'zxcvbnm,.`";
        
        for (ru, en) in ru_chars.chars().zip(en_chars.chars()) {
            ru_en_map.insert(ru, en);
            ru_en_map.insert(ru.to_uppercase().next().unwrap(), en.to_uppercase().next().unwrap());
        }
        
        layout_mappings.insert("ru_to_en".to_string(), LayoutMapping {
            name: "Russian to English".to_string(),
            char_map: ru_en_map.clone(),
        });
        
        // Reverse mapping for English -> Russian
        let en_ru_map: HashMap<char, char> = ru_en_map.iter().map(|(k, v)| (*v, *k)).collect();
        layout_mappings.insert("en_to_ru".to_string(), LayoutMapping {
            name: "English to Russian".to_string(),
            char_map: en_ru_map,
        });

        Config {
            default_from_layout: "auto".to_string(),
            default_to_layout: "auto".to_string(),
            layout_mappings,
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        match Self::config_path() {
            Some(path) => {
                if path.exists() {
                    let content = fs::read_to_string(&path)?;
                    let config: Config = toml::from_str(&content)?;
                    Ok(config)
                } else {
                    let config = Config::default();
                    config.save()?;
                    Ok(config)
                }
            }
            None => Ok(Config::default()),
        }
    }

    pub fn save(&self) -> Result<()> {
        if let Some(path) = Self::config_path() {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            let content = toml::to_string_pretty(self)
                .map_err(|e| ClprsError::ConfigError(e.to_string()))?;
            fs::write(&path, content)?;
        }
        Ok(())
    }

    fn config_path() -> Option<PathBuf> {
        dirs::config_dir().map(|dir| dir.join("clprs").join("config.toml"))
    }
}