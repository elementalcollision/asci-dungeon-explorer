use std::collections::HashMap;
use std::path::PathBuf;
use std::fs;
use std::io;
use serde::{Serialize, Deserialize};
use log::{info, warn, error};

use super::llama_integration::LlamaConfig;
use super::dialogue_system_trait::DialogueConfig;
use super::dialogue_ui::TypingConfig;

/// Language model configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageModelConfig {
    pub enabled: bool,
    pub model_name: String,
    pub model_path: PathBuf,
    pub llama_config: LlamaConfig,
    pub dialogue_config: DialogueConfig,
    pub performance_settings: PerformanceSettings,
    pub fallback_settings: FallbackSettings,
}

impl Default for LanguageModelConfig {
    fn default() -> Self {
        LanguageModelConfig {
            enabled: true,
            model_name: "default".to_string(),
            model_path: PathBuf::from("models/default.gguf"),
            llama_config: LlamaConfig::default(),
            dialogue_config: DialogueConfig::default(),
            performance_settings: PerformanceSettings::default(),
            fallback_settings: FallbackSettings::default(),
        }
    }
}

/// Performance settings for language model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSettings {
    pub max_concurrent_requests: usize,
    pub request_timeout_seconds: u64,
    pub cache_enabled: bool,
    pub cache_size_mb: usize,
    pub background_processing: bool,
    pub priority_mode: PriorityMode,
    pub memory_limit_mb: Option<usize>,
    pub cpu_threads: Option<usize>,
    pub gpu_enabled: bool,
    pub gpu_layers: u32,
}

impl Default for PerformanceSettings {
    fn default() -> Self {
        PerformanceSettings {
            max_concurrent_requests: 3,
            request_timeout_seconds: 30,
            cache_enabled: true,
            cache_size_mb: 100,
            background_processing: true,
            priority_mode: PriorityMode::Balanced,
            memory_limit_mb: Some(512),
            cpu_threads: None, // Auto-detect
            gpu_enabled: false,
            gpu_layers: 0,
        }
    }
}

/// Priority mode for processing requests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PriorityMode {
    Performance, // Prioritize speed
    Quality,     // Prioritize response quality
    Balanced,    // Balance between speed and quality
    PowerSaving, // Minimize resource usage
}

/// Fallback settings when language model fails
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FallbackSettings {
    pub enabled: bool,
    pub use_predefined_responses: bool,
    pub use_simple_ai: bool,
    pub response_templates: HashMap<String, Vec<String>>,
    pub fallback_delay_ms: u64,
    pub max_fallback_attempts: usize,
    pub notify_user_on_fallback: bool,
}

impl Default for FallbackSettings {
    fn default() -> Self {
        let mut response_templates = HashMap::new();
        
        // Default response templates
        response_templates.insert("greeting".to_string(), vec![
            "Hello there, traveler.".to_string(),
            "Greetings, adventurer.".to_string(),
            "Welcome, friend.".to_string(),
        ]);
        
        response_templates.insert("farewell".to_string(), vec![
            "Farewell, and safe travels.".to_string(),
            "Until we meet again.".to_string(),
            "May your journey be prosperous.".to_string(),
        ]);
        
        response_templates.insert("help".to_string(), vec![
            "I'm here to assist you.".to_string(),
            "How may I help you?".to_string(),
            "What do you need?".to_string(),
        ]);
        
        response_templates.insert("confused".to_string(), vec![
            "I'm not sure I understand.".to_string(),
            "Could you rephrase that?".to_string(),
            "I'm having trouble following.".to_string(),
        ]);
        
        FallbackSettings {
            enabled: true,
            use_predefined_responses: true,
            use_simple_ai: false,
            response_templates,
            fallback_delay_ms: 500,
            max_fallback_attempts: 3,
            notify_user_on_fallback: false,
        }
    }
}

/// Available language models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub name: String,
    pub display_name: String,
    pub path: PathBuf,
    pub size_mb: Option<u64>,
    pub description: String,
    pub recommended_settings: LlamaConfig,
    pub requirements: ModelRequirements,
    pub available: bool,
}

/// Model requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelRequirements {
    pub min_memory_mb: usize,
    pub min_cpu_threads: usize,
    pub gpu_required: bool,
    pub min_gpu_memory_mb: Option<usize>,
}

/// UI settings for dialogue interface
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UISettings {
    pub typing_config: TypingConfig,
    pub show_portraits: bool,
    pub show_emotions: bool,
    pub show_typing_indicator: bool,
    pub dialogue_history_length: usize,
    pub auto_continue: bool,
    pub sound_effects: bool,
}

impl Default for UISettings {
    fn default() -> Self {
        UISettings {
            typing_config: TypingConfig::default(),
            show_portraits: true,
            show_emotions: true,
            show_typing_indicator: true,
            dialogue_history_length: 50,
            auto_continue: false,
            sound_effects: false,
        }
    }
}

/// Complete configuration for language model system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageModelSystemConfig {
    pub version: u32,
    pub language_model: LanguageModelConfig,
    pub ui_settings: UISettings,
    pub available_models: Vec<ModelInfo>,
    pub last_updated: u64,
}

impl Default for LanguageModelSystemConfig {
    fn default() -> Self {
        LanguageModelSystemConfig {
            version: 1,
            language_model: LanguageModelConfig::default(),
            ui_settings: UISettings::default(),
            available_models: Vec::new(),
            last_updated: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }
}

/// Configuration manager for language model system
pub struct ConfigManager {
    config: LanguageModelSystemConfig,
    config_path: PathBuf,
    models_directory: PathBuf,
}

impl ConfigManager {
    /// Create a new configuration manager
    pub fn new(config_path: PathBuf, models_directory: PathBuf) -> Self {
        ConfigManager {
            config: LanguageModelSystemConfig::default(),
            config_path,
            models_directory,
        }
    }
    
    /// Initialize the configuration manager
    pub fn initialize(&mut self) -> io::Result<()> {
        // Create directories if they don't exist
        if let Some(parent) = self.config_path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }
        
        if !self.models_directory.exists() {
            fs::create_dir_all(&self.models_directory)?;
        }
        
        // Load configuration
        self.load_config()?;
        
        // Scan for available models
        self.scan_available_models()?;
        
        // Validate current configuration
        self.validate_config();
        
        Ok(())
    }
    
    /// Load configuration from file
    pub fn load_config(&mut self) -> io::Result<()> {
        if self.config_path.exists() {
            let json = fs::read_to_string(&self.config_path)?;
            match serde_json::from_str::<LanguageModelSystemConfig>(&json) {
                Ok(config) => {
                    self.config = config;
                    info!("Loaded language model configuration from {:?}", self.config_path);
                },
                Err(e) => {
                    warn!("Failed to parse configuration file: {}. Using defaults.", e);
                    self.config = LanguageModelSystemConfig::default();
                }
            }
        } else {
            info!("Configuration file not found. Creating default configuration.");
            self.config = LanguageModelSystemConfig::default();
            self.save_config()?;
        }
        
        Ok(())
    }
    
    /// Save configuration to file
    pub fn save_config(&self) -> io::Result<()> {
        let mut config = self.config.clone();
        config.last_updated = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        let json = serde_json::to_string_pretty(&config)?;
        fs::write(&self.config_path, json)?;
        
        info!("Saved language model configuration to {:?}", self.config_path);
        Ok(())
    }
    
    /// Scan for available models in the models directory
    pub fn scan_available_models(&mut self) -> io::Result<()> {
        let mut available_models = Vec::new();
        
        if self.models_directory.exists() {
            for entry in fs::read_dir(&self.models_directory)? {
                let entry = entry?;
                let path = entry.path();
                
                if path.is_file() {
                    if let Some(extension) = path.extension() {
                        let ext = extension.to_string_lossy().to_lowercase();
                        if matches!(ext.as_str(), "gguf" | "ggml" | "bin") {
                            let model_info = self.create_model_info(&path)?;
                            available_models.push(model_info);
                        }
                    }
                }
            }
        }
        
        // Add some default model configurations
        self.add_default_models(&mut available_models);
        
        self.config.available_models = available_models;
        
        info!("Found {} available models", self.config.available_models.len());
        Ok(())
    }
    
    /// Create model info from file path
    fn create_model_info(&self, path: &PathBuf) -> io::Result<ModelInfo> {
        let name = path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();
        
        let display_name = name.replace("-", " ").replace("_", " ");
        
        let size_mb = match fs::metadata(path) {
            Ok(metadata) => Some(metadata.len() / 1024 / 1024),
            Err(_) => None,
        };
        
        let (description, recommended_settings, requirements) = self.get_model_defaults(&name);
        
        Ok(ModelInfo {
            name: name.clone(),
            display_name,
            path: path.clone(),
            size_mb,
            description,
            recommended_settings,
            requirements,
            available: path.exists(),
        })
    }
    
    /// Get default settings for a model based on its name
    fn get_model_defaults(&self, name: &str) -> (String, LlamaConfig, ModelRequirements) {
        let name_lower = name.to_lowercase();
        
        if name_lower.contains("7b") {
            (
                "7 billion parameter model - good balance of quality and performance".to_string(),
                LlamaConfig {
                    context_size: 2048,
                    threads: 4,
                    gpu_layers: 0,
                    temperature: 0.7,
                    top_p: 0.9,
                    max_tokens: 256,
                    ..LlamaConfig::default()
                },
                ModelRequirements {
                    min_memory_mb: 8192,
                    min_cpu_threads: 4,
                    gpu_required: false,
                    min_gpu_memory_mb: None,
                }
            )
        } else if name_lower.contains("13b") {
            (
                "13 billion parameter model - higher quality, requires more resources".to_string(),
                LlamaConfig {
                    context_size: 2048,
                    threads: 6,
                    gpu_layers: 0,
                    temperature: 0.7,
                    top_p: 0.9,
                    max_tokens: 256,
                    ..LlamaConfig::default()
                },
                ModelRequirements {
                    min_memory_mb: 16384,
                    min_cpu_threads: 6,
                    gpu_required: false,
                    min_gpu_memory_mb: None,
                }
            )
        } else if name_lower.contains("tiny") || name_lower.contains("1b") {
            (
                "Small model - fast and lightweight, suitable for testing".to_string(),
                LlamaConfig {
                    context_size: 1024,
                    threads: 2,
                    gpu_layers: 0,
                    temperature: 0.8,
                    top_p: 0.9,
                    max_tokens: 128,
                    ..LlamaConfig::default()
                },
                ModelRequirements {
                    min_memory_mb: 2048,
                    min_cpu_threads: 2,
                    gpu_required: false,
                    min_gpu_memory_mb: None,
                }
            )
        } else {
            (
                "Language model for dialogue generation".to_string(),
                LlamaConfig::default(),
                ModelRequirements {
                    min_memory_mb: 4096,
                    min_cpu_threads: 2,
                    gpu_required: false,
                    min_gpu_memory_mb: None,
                }
            )
        }
    }
    
    /// Add default model configurations
    fn add_default_models(&self, models: &mut Vec<ModelInfo>) {
        let default_models = vec![
            ("tinyllama-1.1b", "TinyLlama 1.1B", "models/tinyllama-1.1b.gguf"),
            ("llama-2-7b-chat", "Llama 2 7B Chat", "models/llama-2-7b-chat.gguf"),
            ("llama-2-13b-chat", "Llama 2 13B Chat", "models/llama-2-13b-chat.gguf"),
        ];
        
        for (name, display_name, path_str) in default_models {
            let path = PathBuf::from(path_str);
            let (description, recommended_settings, requirements) = self.get_model_defaults(name);
            
            // Only add if not already in the list
            if !models.iter().any(|m| m.name == name) {
                models.push(ModelInfo {
                    name: name.to_string(),
                    display_name: display_name.to_string(),
                    path: path.clone(),
                    size_mb: None,
                    description,
                    recommended_settings,
                    requirements,
                    available: path.exists(),
                });
            }
        }
    }
    
    /// Validate current configuration
    fn validate_config(&mut self) {
        // Check if current model is available
        let current_model = &self.config.language_model.model_name;
        let model_available = self.config.available_models.iter()
            .any(|m| m.name == *current_model && m.available);
        
        if !model_available {
            warn!("Current model '{}' is not available", current_model);
            
            // Try to find a fallback model
            if let Some(fallback) = self.config.available_models.iter().find(|m| m.available) {
                info!("Switching to fallback model: {}", fallback.name);
                self.config.language_model.model_name = fallback.name.clone();
                self.config.language_model.model_path = fallback.path.clone();
                self.config.language_model.llama_config = fallback.recommended_settings.clone();
            } else {
                warn!("No available models found, disabling language model");
                self.config.language_model.enabled = false;
            }
        }
        
        // Validate performance settings
        self.validate_performance_settings();
    }
    
    /// Validate performance settings
    fn validate_performance_settings(&mut self) {
        let perf = &mut self.config.language_model.performance_settings;
        
        // Ensure reasonable limits
        perf.max_concurrent_requests = perf.max_concurrent_requests.clamp(1, 10);
        perf.request_timeout_seconds = perf.request_timeout_seconds.clamp(5, 300);
        perf.cache_size_mb = perf.cache_size_mb.clamp(10, 1024);
        
        if let Some(memory_limit) = perf.memory_limit_mb {
            perf.memory_limit_mb = Some(memory_limit.clamp(512, 32768));
        }
        
        if let Some(cpu_threads) = perf.cpu_threads {
            let max_threads = std::thread::available_parallelism()
                .map(|n| n.get())
                .unwrap_or(4);
            perf.cpu_threads = Some(cpu_threads.clamp(1, max_threads));
        }
        
        perf.gpu_layers = perf.gpu_layers.clamp(0, 100);
    }
    
    /// Get current configuration
    pub fn get_config(&self) -> &LanguageModelSystemConfig {
        &self.config
    }
    
    /// Get mutable configuration
    pub fn get_config_mut(&mut self) -> &mut LanguageModelSystemConfig {
        &mut self.config
    }
    
    /// Switch to a different model
    pub fn switch_model(&mut self, model_name: &str) -> Result<(), String> {
        if let Some(model_info) = self.config.available_models.iter().find(|m| m.name == model_name) {
            if !model_info.available {
                return Err(format!("Model '{}' is not available", model_name));
            }
            
            // Check system requirements
            if let Err(msg) = self.check_system_requirements(&model_info.requirements) {
                return Err(format!("System requirements not met for '{}': {}", model_name, msg));
            }
            
            // Update configuration
            self.config.language_model.model_name = model_name.to_string();
            self.config.language_model.model_path = model_info.path.clone();
            self.config.language_model.llama_config = model_info.recommended_settings.clone();
            
            info!("Switched to model: {}", model_name);
            Ok(())
        } else {
            Err(format!("Model '{}' not found", model_name))
        }
    }
    
    /// Check if system meets model requirements
    fn check_system_requirements(&self, requirements: &ModelRequirements) -> Result<(), String> {
        // Check available memory (simplified check)
        // In a real implementation, you'd use system APIs to check actual available memory
        
        if requirements.gpu_required && !self.config.language_model.performance_settings.gpu_enabled {
            return Err("GPU required but not enabled".to_string());
        }
        
        if let Some(cpu_threads) = self.config.language_model.performance_settings.cpu_threads {
            if cpu_threads < requirements.min_cpu_threads {
                return Err(format!("Insufficient CPU threads: {} required, {} configured", 
                    requirements.min_cpu_threads, cpu_threads));
            }
        }
        
        Ok(())
    }
    
    /// Update performance settings
    pub fn update_performance_settings(&mut self, settings: PerformanceSettings) {
        self.config.language_model.performance_settings = settings;
        self.validate_performance_settings();
    }
    
    /// Update UI settings
    pub fn update_ui_settings(&mut self, settings: UISettings) {
        self.config.ui_settings = settings;
    }
    
    /// Update fallback settings
    pub fn update_fallback_settings(&mut self, settings: FallbackSettings) {
        self.config.language_model.fallback_settings = settings;
    }
    
    /// Get available models
    pub fn get_available_models(&self) -> &[ModelInfo] {
        &self.config.available_models
    }
    
    /// Get current model info
    pub fn get_current_model_info(&self) -> Option<&ModelInfo> {
        let current_name = &self.config.language_model.model_name;
        self.config.available_models.iter().find(|m| m.name == *current_name)
    }
    
    /// Enable or disable language model
    pub fn set_enabled(&mut self, enabled: bool) {
        self.config.language_model.enabled = enabled;
    }
    
    /// Check if language model is enabled
    pub fn is_enabled(&self) -> bool {
        self.config.language_model.enabled
    }
    
    /// Reset to default configuration
    pub fn reset_to_defaults(&mut self) {
        let available_models = self.config.available_models.clone();
        self.config = LanguageModelSystemConfig::default();
        self.config.available_models = available_models;
        self.validate_config();
    }
    
    /// Export configuration to a file
    pub fn export_config(&self, path: &PathBuf) -> io::Result<()> {
        let json = serde_json::to_string_pretty(&self.config)?;
        fs::write(path, json)?;
        Ok(())
    }
    
    /// Import configuration from a file
    pub fn import_config(&mut self, path: &PathBuf) -> io::Result<()> {
        let json = fs::read_to_string(path)?;
        let imported_config: LanguageModelSystemConfig = serde_json::from_str(&json)?;
        
        // Keep current available models but update other settings
        let available_models = self.config.available_models.clone();
        self.config = imported_config;
        self.config.available_models = available_models;
        
        self.validate_config();
        Ok(())
    }
    
    /// Get configuration summary for display
    pub fn get_config_summary(&self) -> HashMap<String, String> {
        let mut summary = HashMap::new();
        
        summary.insert("Enabled".to_string(), self.config.language_model.enabled.to_string());
        summary.insert("Current Model".to_string(), self.config.language_model.model_name.clone());
        summary.insert("Available Models".to_string(), self.config.available_models.len().to_string());
        summary.insert("Context Size".to_string(), self.config.language_model.llama_config.context_size.to_string());
        summary.insert("Temperature".to_string(), self.config.language_model.llama_config.temperature.to_string());
        summary.insert("Max Tokens".to_string(), self.config.language_model.llama_config.max_tokens.to_string());
        summary.insert("GPU Enabled".to_string(), self.config.language_model.performance_settings.gpu_enabled.to_string());
        summary.insert("Cache Enabled".to_string(), self.config.language_model.performance_settings.cache_enabled.to_string());
        summary.insert("Fallback Enabled".to_string(), self.config.language_model.fallback_settings.enabled.to_string());
        
        summary
    }
}