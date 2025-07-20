use std::io::Write;
use crossterm::{
    cursor,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{Clear, ClearType},
    ExecutableCommand, QueueableCommand,
};

use super::config_system::{
    ConfigManager, LanguageModelConfig, PerformanceSettings, FallbackSettings, 
    UISettings, ModelInfo, PriorityMode
};

/// Configuration UI state
#[derive(Debug, Clone)]
pub enum ConfigUIState {
    MainMenu,
    ModelSelection,
    PerformanceSettings,
    UISettings,
    FallbackSettings,
    ModelDetails(String),
    Confirmation(String),
}

/// Configuration UI for language model settings
pub struct ConfigUI {
    state: ConfigUIState,
    selected_index: usize,
    config_manager: ConfigManager,
    pending_changes: bool,
}

impl ConfigUI {
    /// Create a new configuration UI
    pub fn new(mut config_manager: ConfigManager) -> Result<Self, String> {
        config_manager.initialize().map_err(|e| format!("Failed to initialize config manager: {}", e))?;
        
        Ok(ConfigUI {
            state: ConfigUIState::MainMenu,
            selected_index: 0,
            config_manager,
            pending_changes: false,
        })
    }
    
    /// Handle input for the configuration UI
    pub fn handle_input(&mut self, input: char) -> ConfigUIResult {
        match self.state {
            ConfigUIState::MainMenu => self.handle_main_menu_input(input),
            ConfigUIState::ModelSelection => self.handle_model_selection_input(input),
            ConfigUIState::PerformanceSettings => self.handle_performance_settings_input(input),
            ConfigUIState::UISettings => self.handle_ui_settings_input(input),
            ConfigUIState::FallbackSettings => self.handle_fallback_settings_input(input),
            ConfigUIState::ModelDetails(_) => self.handle_model_details_input(input),
            ConfigUIState::Confirmation(_) => self.handle_confirmation_input(input),
        }
    }
    
    /// Handle main menu input
    fn handle_main_menu_input(&mut self, input: char) -> ConfigUIResult {
        match input {
            'w' | 'k' => {
                if self.selected_index > 0 {
                    self.selected_index -= 1;
                }
                ConfigUIResult::Continue
            },
            's' | 'j' => {
                if self.selected_index < 6 {
                    self.selected_index += 1;
                }
                ConfigUIResult::Continue
            },
            '\n' | ' ' => {
                match self.selected_index {
                    0 => {
                        // Toggle enabled/disabled
                        let enabled = !self.config_manager.is_enabled();
                        self.config_manager.set_enabled(enabled);
                        self.pending_changes = true;
                        ConfigUIResult::Continue
                    },
                    1 => {
                        self.state = ConfigUIState::ModelSelection;
                        self.selected_index = 0;
                        ConfigUIResult::Continue
                    },
                    2 => {
                        self.state = ConfigUIState::PerformanceSettings;
                        self.selected_index = 0;
                        ConfigUIResult::Continue
                    },
                    3 => {
                        self.state = ConfigUIState::UISettings;
                        self.selected_index = 0;
                        ConfigUIResult::Continue
                    },
                    4 => {
                        self.state = ConfigUIState::FallbackSettings;
                        self.selected_index = 0;
                        ConfigUIResult::Continue
                    },
                    5 => {
                        // Save configuration
                        if let Err(e) = self.config_manager.save_config() {
                            return ConfigUIResult::Error(format!("Failed to save config: {}", e));
                        }
                        self.pending_changes = false;
                        ConfigUIResult::Message("Configuration saved successfully!".to_string())
                    },
                    6 => ConfigUIResult::Exit,
                    _ => ConfigUIResult::Continue,
                }
            },
            'q' => ConfigUIResult::Exit,
            'r' => {
                // Reset to defaults
                self.state = ConfigUIState::Confirmation("Reset all settings to defaults?".to_string());
                ConfigUIResult::Continue
            },
            _ => ConfigUIResult::Continue,
        }
    }
    
    /// Handle model selection input
    fn handle_model_selection_input(&mut self, input: char) -> ConfigUIResult {
        let models = self.config_manager.get_available_models();
        
        match input {
            'w' | 'k' => {
                if self.selected_index > 0 {
                    self.selected_index -= 1;
                }
                ConfigUIResult::Continue
            },
            's' | 'j' => {
                if self.selected_index < models.len().saturating_sub(1) {
                    self.selected_index += 1;
                }
                ConfigUIResult::Continue
            },
            '\n' | ' ' => {
                if self.selected_index < models.len() {
                    let model_name = models[self.selected_index].name.clone();
                    match self.config_manager.switch_model(&model_name) {
                        Ok(_) => {
                            self.pending_changes = true;
                            ConfigUIResult::Message(format!("Switched to model: {}", model_name))
                        },
                        Err(e) => ConfigUIResult::Error(e),
                    }
                } else {
                    ConfigUIResult::Continue
                }
            },
            'd' => {
                // Show model details
                if self.selected_index < models.len() {
                    let model_name = models[self.selected_index].name.clone();
                    self.state = ConfigUIState::ModelDetails(model_name);
                }
                ConfigUIResult::Continue
            },
            'q' | '\x1b' => {
                self.state = ConfigUIState::MainMenu;
                self.selected_index = 1;
                ConfigUIResult::Continue
            },
            _ => ConfigUIResult::Continue,
        }
    }
    
    /// Handle performance settings input
    fn handle_performance_settings_input(&mut self, input: char) -> ConfigUIResult {
        match input {
            'w' | 'k' => {
                if self.selected_index > 0 {
                    self.selected_index -= 1;
                }
                ConfigUIResult::Continue
            },
            's' | 'j' => {
                if self.selected_index < 8 {
                    self.selected_index += 1;
                }
                ConfigUIResult::Continue
            },
            '\n' | ' ' => {
                self.toggle_performance_setting();
                ConfigUIResult::Continue
            },
            'q' | '\x1b' => {
                self.state = ConfigUIState::MainMenu;
                self.selected_index = 2;
                ConfigUIResult::Continue
            },
            _ => ConfigUIResult::Continue,
        }
    }
    
    /// Handle UI settings input
    fn handle_ui_settings_input(&mut self, input: char) -> ConfigUIResult {
        match input {
            'w' | 'k' => {
                if self.selected_index > 0 {
                    self.selected_index -= 1;
                }
                ConfigUIResult::Continue
            },
            's' | 'j' => {
                if self.selected_index < 6 {
                    self.selected_index += 1;
                }
                ConfigUIResult::Continue
            },
            '\n' | ' ' => {
                self.toggle_ui_setting();
                ConfigUIResult::Continue
            },
            'q' | '\x1b' => {
                self.state = ConfigUIState::MainMenu;
                self.selected_index = 3;
                ConfigUIResult::Continue
            },
            _ => ConfigUIResult::Continue,
        }
    }
    
    /// Handle fallback settings input
    fn handle_fallback_settings_input(&mut self, input: char) -> ConfigUIResult {
        match input {
            'w' | 'k' => {
                if self.selected_index > 0 {
                    self.selected_index -= 1;
                }
                ConfigUIResult::Continue
            },
            's' | 'j' => {
                if self.selected_index < 4 {
                    self.selected_index += 1;
                }
                ConfigUIResult::Continue
            },
            '\n' | ' ' => {
                self.toggle_fallback_setting();
                ConfigUIResult::Continue
            },
            'q' | '\x1b' => {
                self.state = ConfigUIState::MainMenu;
                self.selected_index = 4;
                ConfigUIResult::Continue
            },
            _ => ConfigUIResult::Continue,
        }
    }
    
    /// Handle model details input
    fn handle_model_details_input(&mut self, input: char) -> ConfigUIResult {
        match input {
            'q' | '\x1b' | '\n' | ' ' => {
                self.state = ConfigUIState::ModelSelection;
                ConfigUIResult::Continue
            },
            _ => ConfigUIResult::Continue,
        }
    }
    
    /// Handle confirmation input
    fn handle_confirmation_input(&mut self, input: char) -> ConfigUIResult {
        match input {
            'y' | 'Y' => {
                self.config_manager.reset_to_defaults();
                self.pending_changes = true;
                self.state = ConfigUIState::MainMenu;
                self.selected_index = 0;
                ConfigUIResult::Message("Settings reset to defaults".to_string())
            },
            'n' | 'N' | 'q' | '\x1b' => {
                self.state = ConfigUIState::MainMenu;
                ConfigUIResult::Continue
            },
            _ => ConfigUIResult::Continue,
        }
    }
    
    /// Toggle performance setting
    fn toggle_performance_setting(&mut self) {
        let mut settings = self.config_manager.get_config().language_model.performance_settings.clone();
        
        match self.selected_index {
            0 => settings.cache_enabled = !settings.cache_enabled,
            1 => settings.background_processing = !settings.background_processing,
            2 => settings.gpu_enabled = !settings.gpu_enabled,
            3 => {
                // Cycle through priority modes
                settings.priority_mode = match settings.priority_mode {
                    PriorityMode::Performance => PriorityMode::Quality,
                    PriorityMode::Quality => PriorityMode::Balanced,
                    PriorityMode::Balanced => PriorityMode::PowerSaving,
                    PriorityMode::PowerSaving => PriorityMode::Performance,
                };
            },
            4 => {
                // Adjust max concurrent requests
                settings.max_concurrent_requests = match settings.max_concurrent_requests {
                    1 => 2,
                    2 => 3,
                    3 => 5,
                    5 => 1,
                    _ => 3,
                };
            },
            5 => {
                // Adjust timeout
                settings.request_timeout_seconds = match settings.request_timeout_seconds {
                    10 => 30,
                    30 => 60,
                    60 => 120,
                    120 => 10,
                    _ => 30,
                };
            },
            6 => {
                // Adjust cache size
                settings.cache_size_mb = match settings.cache_size_mb {
                    50 => 100,
                    100 => 200,
                    200 => 500,
                    500 => 50,
                    _ => 100,
                };
            },
            7 => {
                // Adjust GPU layers
                settings.gpu_layers = match settings.gpu_layers {
                    0 => 10,
                    10 => 20,
                    20 => 35,
                    35 => 0,
                    _ => 0,
                };
            },
            _ => return,
        }
        
        self.config_manager.update_performance_settings(settings);
        self.pending_changes = true;
    }
    
    /// Toggle UI setting
    fn toggle_ui_setting(&mut self) {
        let mut settings = self.config_manager.get_config().ui_settings.clone();
        
        match self.selected_index {
            0 => settings.show_portraits = !settings.show_portraits,
            1 => settings.show_emotions = !settings.show_emotions,
            2 => settings.show_typing_indicator = !settings.show_typing_indicator,
            3 => settings.auto_continue = !settings.auto_continue,
            4 => settings.sound_effects = !settings.sound_effects,
            5 => {
                // Adjust typing speed
                settings.typing_config.chars_per_second = match settings.typing_config.chars_per_second as u32 {
                    10 => 20.0,
                    20 => 30.0,
                    30 => 50.0,
                    50 => 10.0,
                    _ => 30.0,
                };
            },
            6 => {
                // Adjust history length
                settings.dialogue_history_length = match settings.dialogue_history_length {
                    25 => 50,
                    50 => 100,
                    100 => 200,
                    200 => 25,
                    _ => 50,
                };
            },
            _ => return,
        }
        
        self.config_manager.update_ui_settings(settings);
        self.pending_changes = true;
    }
    
    /// Toggle fallback setting
    fn toggle_fallback_setting(&mut self) {
        let mut settings = self.config_manager.get_config().language_model.fallback_settings.clone();
        
        match self.selected_index {
            0 => settings.enabled = !settings.enabled,
            1 => settings.use_predefined_responses = !settings.use_predefined_responses,
            2 => settings.use_simple_ai = !settings.use_simple_ai,
            3 => settings.notify_user_on_fallback = !settings.notify_user_on_fallback,
            4 => {
                // Adjust max attempts
                settings.max_fallback_attempts = match settings.max_fallback_attempts {
                    1 => 3,
                    3 => 5,
                    5 => 1,
                    _ => 3,
                };
            },
            _ => return,
        }
        
        self.config_manager.update_fallback_settings(settings);
        self.pending_changes = true;
    }
    
    /// Render the configuration UI
    pub fn render<W: Write>(&self, writer: &mut W) -> crossterm::Result<()> {
        writer.queue(Clear(ClearType::All))?;
        
        match &self.state {
            ConfigUIState::MainMenu => self.render_main_menu(writer)?,
            ConfigUIState::ModelSelection => self.render_model_selection(writer)?,
            ConfigUIState::PerformanceSettings => self.render_performance_settings(writer)?,
            ConfigUIState::UISettings => self.render_ui_settings(writer)?,
            ConfigUIState::FallbackSettings => self.render_fallback_settings(writer)?,
            ConfigUIState::ModelDetails(model_name) => self.render_model_details(writer, model_name)?,
            ConfigUIState::Confirmation(message) => self.render_confirmation(writer, message)?,
        }
        
        writer.flush()?;
        Ok(())
    }
    
    /// Render main menu
    fn render_main_menu<W: Write>(&self, writer: &mut W) -> crossterm::Result<()> {
        writer.queue(cursor::MoveTo(2, 1))?;
        writer.queue(SetForegroundColor(Color::Yellow))?;
        writer.queue(Print("Language Model Configuration"))?;
        writer.queue(ResetColor)?;
        
        if self.pending_changes {
            writer.queue(cursor::MoveTo(35, 1))?;
            writer.queue(SetForegroundColor(Color::Red))?;
            writer.queue(Print("(Unsaved Changes)"))?;
            writer.queue(ResetColor)?;
        }
        
        let config = self.config_manager.get_config();
        let current_model = self.config_manager.get_current_model_info();
        
        // Status information
        writer.queue(cursor::MoveTo(2, 3))?;
        writer.queue(Print(format!("Status: {}", if config.language_model.enabled { "Enabled" } else { "Disabled" })))?;
        
        writer.queue(cursor::MoveTo(2, 4))?;
        writer.queue(Print(format!("Current Model: {}", 
            current_model.map(|m| m.display_name.as_str()).unwrap_or("None"))))?;
        
        writer.queue(cursor::MoveTo(2, 5))?;
        writer.queue(Print(format!("Available Models: {}", config.available_models.len())))?;
        
        // Menu options
        let menu_items = vec![
            format!("Toggle Language Model ({})", if config.language_model.enabled { "Enabled" } else { "Disabled" }),
            "Select Model".to_string(),
            "Performance Settings".to_string(),
            "UI Settings".to_string(),
            "Fallback Settings".to_string(),
            "Save Configuration".to_string(),
            "Exit".to_string(),
        ];
        
        for (i, item) in menu_items.iter().enumerate() {
            writer.queue(cursor::MoveTo(4, 7 + i as u16))?;
            
            if i == self.selected_index {
                writer.queue(SetForegroundColor(Color::Yellow))?;
                writer.queue(Print(format!("> {}", item)))?;
            } else {
                writer.queue(Print(format!("  {}", item)))?;
            }
            writer.queue(ResetColor)?;
        }
        
        // Help text
        writer.queue(cursor::MoveTo(2, 16))?;
        writer.queue(SetForegroundColor(Color::DarkGrey))?;
        writer.queue(Print("Controls: w/k=up, s/j=down, Enter/Space=select, q=quit, r=reset"))?;
        writer.queue(ResetColor)?;
        
        Ok(())
    }
    
    /// Render model selection
    fn render_model_selection<W: Write>(&self, writer: &mut W) -> crossterm::Result<()> {
        writer.queue(cursor::MoveTo(2, 1))?;
        writer.queue(SetForegroundColor(Color::Yellow))?;
        writer.queue(Print("Model Selection"))?;
        writer.queue(ResetColor)?;
        
        let models = self.config_manager.get_available_models();
        let current_model = &self.config_manager.get_config().language_model.model_name;
        
        for (i, model) in models.iter().enumerate() {
            writer.queue(cursor::MoveTo(4, 3 + i as u16))?;
            
            let prefix = if i == self.selected_index {
                writer.queue(SetForegroundColor(Color::Yellow))?;
                "> "
            } else {
                "  "
            };
            
            let status = if model.name == *current_model {
                " [CURRENT]"
            } else if !model.available {
                " [NOT AVAILABLE]"
            } else {
                ""
            };
            
            let color = if !model.available {
                Color::DarkGrey
            } else if model.name == *current_model {
                Color::Green
            } else {
                Color::White
            };
            
            writer.queue(SetForegroundColor(color))?;
            writer.queue(Print(format!("{}{}{}", prefix, model.display_name, status)))?;
            writer.queue(ResetColor)?;
            
            // Show model size if available
            if let Some(size_mb) = model.size_mb {
                writer.queue(cursor::MoveTo(40, 3 + i as u16))?;
                writer.queue(SetForegroundColor(Color::DarkGrey))?;
                writer.queue(Print(format!("({} MB)", size_mb)))?;
                writer.queue(ResetColor)?;
            }
        }
        
        // Help text
        writer.queue(cursor::MoveTo(2, 3 + models.len() as u16 + 2))?;
        writer.queue(SetForegroundColor(Color::DarkGrey))?;
        writer.queue(Print("Controls: w/k=up, s/j=down, Enter=select, d=details, q/Esc=back"))?;
        writer.queue(ResetColor)?;
        
        Ok(())
    }
    
    /// Render performance settings
    fn render_performance_settings<W: Write>(&self, writer: &mut W) -> crossterm::Result<()> {
        writer.queue(cursor::MoveTo(2, 1))?;
        writer.queue(SetForegroundColor(Color::Yellow))?;
        writer.queue(Print("Performance Settings"))?;
        writer.queue(ResetColor)?;
        
        let settings = &self.config_manager.get_config().language_model.performance_settings;
        
        let options = vec![
            format!("Cache Enabled: {}", settings.cache_enabled),
            format!("Background Processing: {}", settings.background_processing),
            format!("GPU Enabled: {}", settings.gpu_enabled),
            format!("Priority Mode: {:?}", settings.priority_mode),
            format!("Max Concurrent Requests: {}", settings.max_concurrent_requests),
            format!("Request Timeout: {}s", settings.request_timeout_seconds),
            format!("Cache Size: {} MB", settings.cache_size_mb),
            format!("GPU Layers: {}", settings.gpu_layers),
        ];
        
        for (i, option) in options.iter().enumerate() {
            writer.queue(cursor::MoveTo(4, 3 + i as u16))?;
            
            if i == self.selected_index {
                writer.queue(SetForegroundColor(Color::Yellow))?;
                writer.queue(Print(format!("> {}", option)))?;
            } else {
                writer.queue(Print(format!("  {}", option)))?;
            }
            writer.queue(ResetColor)?;
        }
        
        // Help text
        writer.queue(cursor::MoveTo(2, 14))?;
        writer.queue(SetForegroundColor(Color::DarkGrey))?;
        writer.queue(Print("Controls: w/k=up, s/j=down, Enter/Space=toggle, q/Esc=back"))?;
        writer.queue(ResetColor)?;
        
        Ok(())
    }
    
    /// Render UI settings
    fn render_ui_settings<W: Write>(&self, writer: &mut W) -> crossterm::Result<()> {
        writer.queue(cursor::MoveTo(2, 1))?;
        writer.queue(SetForegroundColor(Color::Yellow))?;
        writer.queue(Print("UI Settings"))?;
        writer.queue(ResetColor)?;
        
        let settings = &self.config_manager.get_config().ui_settings;
        
        let options = vec![
            format!("Show Portraits: {}", settings.show_portraits),
            format!("Show Emotions: {}", settings.show_emotions),
            format!("Show Typing Indicator: {}", settings.show_typing_indicator),
            format!("Auto Continue: {}", settings.auto_continue),
            format!("Sound Effects: {}", settings.sound_effects),
            format!("Typing Speed: {} chars/sec", settings.typing_config.chars_per_second as u32),
            format!("History Length: {}", settings.dialogue_history_length),
        ];
        
        for (i, option) in options.iter().enumerate() {
            writer.queue(cursor::MoveTo(4, 3 + i as u16))?;
            
            if i == self.selected_index {
                writer.queue(SetForegroundColor(Color::Yellow))?;
                writer.queue(Print(format!("> {}", option)))?;
            } else {
                writer.queue(Print(format!("  {}", option)))?;
            }
            writer.queue(ResetColor)?;
        }
        
        // Help text
        writer.queue(cursor::MoveTo(2, 12))?;
        writer.queue(SetForegroundColor(Color::DarkGrey))?;
        writer.queue(Print("Controls: w/k=up, s/j=down, Enter/Space=toggle, q/Esc=back"))?;
        writer.queue(ResetColor)?;
        
        Ok(())
    }
    
    /// Render fallback settings
    fn render_fallback_settings<W: Write>(&self, writer: &mut W) -> crossterm::Result<()> {
        writer.queue(cursor::MoveTo(2, 1))?;
        writer.queue(SetForegroundColor(Color::Yellow))?;
        writer.queue(Print("Fallback Settings"))?;
        writer.queue(ResetColor)?;
        
        let settings = &self.config_manager.get_config().language_model.fallback_settings;
        
        let options = vec![
            format!("Fallback Enabled: {}", settings.enabled),
            format!("Use Predefined Responses: {}", settings.use_predefined_responses),
            format!("Use Simple AI: {}", settings.use_simple_ai),
            format!("Notify User on Fallback: {}", settings.notify_user_on_fallback),
            format!("Max Fallback Attempts: {}", settings.max_fallback_attempts),
        ];
        
        for (i, option) in options.iter().enumerate() {
            writer.queue(cursor::MoveTo(4, 3 + i as u16))?;
            
            if i == self.selected_index {
                writer.queue(SetForegroundColor(Color::Yellow))?;
                writer.queue(Print(format!("> {}", option)))?;
            } else {
                writer.queue(Print(format!("  {}", option)))?;
            }
            writer.queue(ResetColor)?;
        }
        
        // Help text
        writer.queue(cursor::MoveTo(2, 10))?;
        writer.queue(SetForegroundColor(Color::DarkGrey))?;
        writer.queue(Print("Controls: w/k=up, s/j=down, Enter/Space=toggle, q/Esc=back"))?;
        writer.queue(ResetColor)?;
        
        Ok(())
    }
    
    /// Render model details
    fn render_model_details<W: Write>(&self, writer: &mut W, model_name: &str) -> crossterm::Result<()> {
        writer.queue(cursor::MoveTo(2, 1))?;
        writer.queue(SetForegroundColor(Color::Yellow))?;
        writer.queue(Print(format!("Model Details: {}", model_name)))?;
        writer.queue(ResetColor)?;
        
        if let Some(model) = self.config_manager.get_available_models().iter().find(|m| m.name == *model_name) {
            writer.queue(cursor::MoveTo(4, 3))?;
            writer.queue(Print(format!("Display Name: {}", model.display_name)))?;
            
            writer.queue(cursor::MoveTo(4, 4))?;
            writer.queue(Print(format!("Path: {}", model.path.display())))?;
            
            writer.queue(cursor::MoveTo(4, 5))?;
            writer.queue(Print(format!("Available: {}", model.available)))?;
            
            if let Some(size) = model.size_mb {
                writer.queue(cursor::MoveTo(4, 6))?;
                writer.queue(Print(format!("Size: {} MB", size)))?;
            }
            
            writer.queue(cursor::MoveTo(4, 8))?;
            writer.queue(Print(format!("Description: {}", model.description)))?;
            
            writer.queue(cursor::MoveTo(4, 10))?;
            writer.queue(SetForegroundColor(Color::Cyan))?;
            writer.queue(Print("Requirements:"))?;
            writer.queue(ResetColor)?;
            
            writer.queue(cursor::MoveTo(6, 11))?;
            writer.queue(Print(format!("Min Memory: {} MB", model.requirements.min_memory_mb)))?;
            
            writer.queue(cursor::MoveTo(6, 12))?;
            writer.queue(Print(format!("Min CPU Threads: {}", model.requirements.min_cpu_threads)))?;
            
            writer.queue(cursor::MoveTo(6, 13))?;
            writer.queue(Print(format!("GPU Required: {}", model.requirements.gpu_required)))?;
            
            writer.queue(cursor::MoveTo(4, 15))?;
            writer.queue(SetForegroundColor(Color::Cyan))?;
            writer.queue(Print("Recommended Settings:"))?;
            writer.queue(ResetColor)?;
            
            writer.queue(cursor::MoveTo(6, 16))?;
            writer.queue(Print(format!("Context Size: {}", model.recommended_settings.context_size)))?;
            
            writer.queue(cursor::MoveTo(6, 17))?;
            writer.queue(Print(format!("Threads: {}", model.recommended_settings.threads)))?;
            
            writer.queue(cursor::MoveTo(6, 18))?;
            writer.queue(Print(format!("Temperature: {}", model.recommended_settings.temperature)))?;
        }
        
        // Help text
        writer.queue(cursor::MoveTo(2, 20))?;
        writer.queue(SetForegroundColor(Color::DarkGrey))?;
        writer.queue(Print("Press any key to return"))?;
        writer.queue(ResetColor)?;
        
        Ok(())
    }
    
    /// Render confirmation dialog
    fn render_confirmation<W: Write>(&self, writer: &mut W, message: &str) -> crossterm::Result<()> {
        writer.queue(cursor::MoveTo(2, 8))?;
        writer.queue(SetForegroundColor(Color::Yellow))?;
        writer.queue(Print("Confirmation"))?;
        writer.queue(ResetColor)?;
        
        writer.queue(cursor::MoveTo(4, 10))?;
        writer.queue(Print(message))?;
        
        writer.queue(cursor::MoveTo(4, 12))?;
        writer.queue(Print("Press Y to confirm, N to cancel"))?;
        
        Ok(())
    }
    
    /// Check if there are pending changes
    pub fn has_pending_changes(&self) -> bool {
        self.pending_changes
    }
}

/// Result of configuration UI operations
#[derive(Debug)]
pub enum ConfigUIResult {
    Continue,
    Exit,
    Message(String),
    Error(String),
}