use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs;
use std::io::{Read, Write};
use serde::{Serialize, Deserialize};

/// Game settings categories
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SettingsCategory {
    Graphics,
    Audio,
    Controls,
    Gameplay,
    Accessibility,
    Advanced,
}

/// Setting value types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SettingValue {
    Bool(bool),
    Int(i32),
    Float(f32),
    String(String),
    StringList(Vec<String>),
    IntRange(i32, i32, i32), // value, min, max
    FloatRange(f32, f32, f32), // value, min, max
    Color(u8, u8, u8, u8), // r, g, b, a
    KeyBinding(String),
}

impl SettingValue {
    /// Get the value as a boolean
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            SettingValue::Bool(value) => Some(*value),
            _ => None,
        }
    }

    /// Get the value as an integer
    pub fn as_int(&self) -> Option<i32> {
        match self {
            SettingValue::Int(value) => Some(*value),
            SettingValue::IntRange(value, _, _) => Some(*value),
            _ => None,
        }
    }

    /// Get the value as a float
    pub fn as_float(&self) -> Option<f32> {
        match self {
            SettingValue::Float(value) => Some(*value),
            SettingValue::FloatRange(value, _, _) => Some(*value),
            _ => None,
        }
    }

    /// Get the value as a string
    pub fn as_string(&self) -> Option<&str> {
        match self {
            SettingValue::String(value) => Some(value),
            SettingValue::KeyBinding(value) => Some(value),
            _ => None,
        }
    }

    /// Update the value
    pub fn update_value(&mut self, new_value: SettingValue) -> bool {
        match (self, &new_value) {
            (SettingValue::Bool(value), SettingValue::Bool(new_value)) => {
                *value = *new_value;
                true
            },
            (SettingValue::Int(value), SettingValue::Int(new_value)) => {
                *value = *new_value;
                true
            },
            (SettingValue::Float(value), SettingValue::Float(new_value)) => {
                *value = *new_value;
                true
            },
            (SettingValue::String(value), SettingValue::String(new_value)) => {
                *value = new_value.clone();
                true
            },
            (SettingValue::IntRange(value, min, max), SettingValue::Int(new_value)) => {
                *value = (*new_value).clamp(*min, *max);
                true
            },
            (SettingValue::FloatRange(value, min, max), SettingValue::Float(new_value)) => {
                *value = (*new_value).clamp(*min, *max);
                true
            },
            (SettingValue::KeyBinding(value), SettingValue::KeyBinding(new_value)) => {
                *value = new_value.clone();
                true
            },
            _ => false, // Type mismatch
        }
    }
}

/// Setting definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Setting {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: SettingsCategory,
    pub value: SettingValue,
    pub default_value: SettingValue,
    pub requires_restart: bool,
}

impl Setting {
    pub fn new(
        id: String,
        name: String,
        description: String,
        category: SettingsCategory,
        value: SettingValue,
    ) -> Self {
        Setting {
            id,
            name,
            description,
            category,
            default_value: value.clone(),
            value,
            requires_restart: false,
        }
    }

    pub fn with_restart(mut self, requires_restart: bool) -> Self {
        self.requires_restart = requires_restart;
        self
    }

    /// Reset to default value
    pub fn reset_to_default(&mut self) {
        self.value = self.default_value.clone();
    }
}

/// Settings system errors
#[derive(Debug, thiserror::Error)]
pub enum SettingsError {
    #[error("Setting not found: {0}")]
    SettingNotFound(String),
    
    #[error("Invalid setting value: {0}")]
    InvalidValue(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
}

pub type SettingsResult<T> = Result<T, SettingsError>;

/// Game settings system
pub struct SettingsSystem {
    settings: HashMap<String, Setting>,
    settings_file: PathBuf,
}

impl SettingsSystem {
    /// Create a new settings system
    pub fn new<P: AsRef<Path>>(settings_file: P) -> Self {
        let mut system = SettingsSystem {
            settings: HashMap::new(),
            settings_file: settings_file.as_ref().to_path_buf(),
        };

        // Initialize with default settings
        system.initialize_default_settings();
        system
    }

    /// Initialize default settings
    fn initialize_default_settings(&mut self) {
        // Graphics settings
        self.add_setting(Setting::new(
            "fullscreen".to_string(),
            "Fullscreen".to_string(),
            "Run the game in fullscreen mode".to_string(),
            SettingsCategory::Graphics,
            SettingValue::Bool(false),
        ));

        self.add_setting(Setting::new(
            "vsync".to_string(),
            "VSync".to_string(),
            "Vertical synchronization".to_string(),
            SettingsCategory::Graphics,
            SettingValue::Bool(true),
        ));

        self.add_setting(Setting::new(
            "fps_limit".to_string(),
            "FPS Limit".to_string(),
            "Limit frames per second".to_string(),
            SettingsCategory::Graphics,
            SettingValue::IntRange(60, 30, 240),
        ));

        // Audio settings
        self.add_setting(Setting::new(
            "master_volume".to_string(),
            "Master Volume".to_string(),
            "Overall volume".to_string(),
            SettingsCategory::Audio,
            SettingValue::FloatRange(0.8, 0.0, 1.0),
        ));

        self.add_setting(Setting::new(
            "music_volume".to_string(),
            "Music Volume".to_string(),
            "Music volume".to_string(),
            SettingsCategory::Audio,
            SettingValue::FloatRange(0.7, 0.0, 1.0),
        ));

        // Control settings
        self.add_setting(Setting::new(
            "move_up".to_string(),
            "Move Up".to_string(),
            "Key binding for moving up".to_string(),
            SettingsCategory::Controls,
            SettingValue::KeyBinding("KeyW".to_string()),
        ));

        self.add_setting(Setting::new(
            "move_down".to_string(),
            "Move Down".to_string(),
            "Key binding for moving down".to_string(),
            SettingsCategory::Controls,
            SettingValue::KeyBinding("KeyS".to_string()),
        ));

        self.add_setting(Setting::new(
            "move_left".to_string(),
            "Move Left".to_string(),
            "Key binding for moving left".to_string(),
            SettingsCategory::Controls,
            SettingValue::KeyBinding("KeyA".to_string()),
        ));

        self.add_setting(Setting::new(
            "move_right".to_string(),
            "Move Right".to_string(),
            "Key binding for moving right".to_string(),
            SettingsCategory::Controls,
            SettingValue::KeyBinding("KeyD".to_string()),
        ));

        // Gameplay settings
        self.add_setting(Setting::new(
            "difficulty".to_string(),
            "Difficulty".to_string(),
            "Game difficulty".to_string(),
            SettingsCategory::Gameplay,
            SettingValue::String("normal".to_string()),
        ));

        self.add_setting(Setting::new(
            "autosave_interval".to_string(),
            "Autosave Interval".to_string(),
            "Time between autosaves (in minutes)".to_string(),
            SettingsCategory::Gameplay,
            SettingValue::IntRange(5, 1, 60),
        ));
    }

    /// Add a setting
    pub fn add_setting(&mut self, setting: Setting) {
        self.settings.insert(setting.id.clone(), setting);
    }

    /// Get a setting by ID
    pub fn get_setting(&self, id: &str) -> Option<&Setting> {
        self.settings.get(id)
    }

    /// Get settings by category
    pub fn get_settings_by_category(&self, category: &SettingsCategory) -> Vec<&Setting> {
        self.settings.values()
            .filter(|setting| &setting.category == category)
            .collect()
    }

    /// Update a setting value
    pub fn update_setting(&mut self, id: &str, value: SettingValue) -> SettingsResult<bool> {
        if let Some(setting) = self.settings.get_mut(id) {
            let updated = setting.value.update_value(value);
            if !updated {
                return Err(SettingsError::InvalidValue(format!("Invalid value for setting: {}", id)));
            }
            Ok(updated)
        } else {
            Err(SettingsError::SettingNotFound(id.to_string()))
        }
    }

    /// Get a boolean setting value
    pub fn get_bool(&self, id: &str) -> SettingsResult<bool> {
        if let Some(setting) = self.settings.get(id) {
            if let Some(value) = setting.value.as_bool() {
                Ok(value)
            } else {
                Err(SettingsError::InvalidValue(format!("Setting {} is not a boolean", id)))
            }
        } else {
            Err(SettingsError::SettingNotFound(id.to_string()))
        }
    }

    /// Get an integer setting value
    pub fn get_int(&self, id: &str) -> SettingsResult<i32> {
        if let Some(setting) = self.settings.get(id) {
            if let Some(value) = setting.value.as_int() {
                Ok(value)
            } else {
                Err(SettingsError::InvalidValue(format!("Setting {} is not an integer", id)))
            }
        } else {
            Err(SettingsError::SettingNotFound(id.to_string()))
        }
    }

    /// Get a float setting value
    pub fn get_float(&self, id: &str) -> SettingsResult<f32> {
        if let Some(setting) = self.settings.get(id) {
            if let Some(value) = setting.value.as_float() {
                Ok(value)
            } else {
                Err(SettingsError::InvalidValue(format!("Setting {} is not a float", id)))
            }
        } else {
            Err(SettingsError::SettingNotFound(id.to_string()))
        }
    }

    /// Get a string setting value
    pub fn get_string(&self, id: &str) -> SettingsResult<&str> {
        if let Some(setting) = self.settings.get(id) {
            if let Some(value) = setting.value.as_string() {
                Ok(value)
            } else {
                Err(SettingsError::InvalidValue(format!("Setting {} is not a string", id)))
            }
        } else {
            Err(SettingsError::SettingNotFound(id.to_string()))
        }
    }

    /// Set a boolean setting
    pub fn set_bool(&mut self, id: &str, value: bool) -> SettingsResult<bool> {
        self.update_setting(id, SettingValue::Bool(value))
    }

    /// Set an integer setting
    pub fn set_int(&mut self, id: &str, value: i32) -> SettingsResult<bool> {
        self.update_setting(id, SettingValue::Int(value))
    }

    /// Set a float setting
    pub fn set_float(&mut self, id: &str, value: f32) -> SettingsResult<bool> {
        self.update_setting(id, SettingValue::Float(value))
    }

    /// Set a string setting
    pub fn set_string(&mut self, id: &str, value: String) -> SettingsResult<bool> {
        self.update_setting(id, SettingValue::String(value))
    }

    /// Save settings to file
    pub fn save_settings(&self) -> SettingsResult<()> {
        // Create parent directory if it doesn't exist
        if let Some(parent) = self.settings_file.parent() {
            fs::create_dir_all(parent)?;
        }

        // Serialize settings
        let json = serde_json::to_string_pretty(&self.settings)
            .map_err(|e| SettingsError::SerializationError(e.to_string()))?;

        // Write to file
        let mut file = fs::File::create(&self.settings_file)?;
        file.write_all(json.as_bytes())?;

        Ok(())
    }

    /// Load settings from file
    pub fn load_settings(&mut self) -> SettingsResult<()> {
        // Check if file exists
        if !self.settings_file.exists() {
            return Ok(()); // Use defaults
        }

        // Read file
        let mut file = fs::File::open(&self.settings_file)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        // Deserialize settings
        let loaded_settings: HashMap<String, Setting> = serde_json::from_str(&contents)
            .map_err(|e| SettingsError::SerializationError(e.to_string()))?;

        // Update settings, preserving defaults for any that don't exist in the file
        for (id, loaded_setting) in loaded_settings {
            if let Some(existing_setting) = self.settings.get_mut(&id) {
                existing_setting.value = loaded_setting.value;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_settings_system_creation() {
        let system = SettingsSystem::new("test.json");
        assert!(system.settings.len() > 0);
    }

    #[test]
    fn test_setting_value_updates() {
        let mut value = SettingValue::Bool(false);
        assert!(value.update_value(SettingValue::Bool(true)));
        assert_eq!(value.as_bool(), Some(true));
        
        let mut value = SettingValue::IntRange(50, 0, 100);
        assert!(value.update_value(SettingValue::Int(75)));
        assert_eq!(value.as_int(), Some(75));
        
        // Test clamping
        assert!(value.update_value(SettingValue::Int(150)));
        assert_eq!(value.as_int(), Some(100));
    }

    #[test]
    fn test_settings_categories() {
        let system = SettingsSystem::new("test.json");
        
        let graphics_settings = system.get_settings_by_category(&SettingsCategory::Graphics);
        assert!(graphics_settings.len() > 0);
        
        let audio_settings = system.get_settings_by_category(&SettingsCategory::Audio);
        assert!(audio_settings.len() > 0);
    }
}