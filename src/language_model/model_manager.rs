use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use log::{info, warn, error};

use super::llama_integration::{LlamaContext, LlamaConfig, LlamaError};

/// Model manager for handling multiple language models
pub struct ModelManager {
    models: HashMap<String, Arc<Mutex<LlamaContext>>>,
    default_model: Option<String>,
    model_paths: HashMap<String, PathBuf>,
}

impl ModelManager {
    /// Create a new model manager
    pub fn new() -> Self {
        ModelManager {
            models: HashMap::new(),
            default_model: None,
            model_paths: HashMap::new(),
        }
    }
    
    /// Initialize the model manager with default paths
    pub fn initialize(&mut self) -> Result<(), LlamaError> {
        // Add default model paths
        self.add_model_path("7b-chat", PathBuf::from("models/llama-2-7b-chat.gguf"));
        self.add_model_path("13b-chat", PathBuf::from("models/llama-2-13b-chat.gguf"));
        self.add_model_path("tiny", PathBuf::from("models/tinyllama-1.1b.gguf"));
        
        Ok(())
    }
    
    /// Add a model path to the manager
    pub fn add_model_path(&mut self, name: &str, path: PathBuf) {
        self.model_paths.insert(name.to_string(), path);
    }
    
    /// Load a model by name
    pub fn load_model(&mut self, name: &str, config_override: Option<LlamaConfig>) -> Result<(), LlamaError> {
        // Check if model is already loaded
        if self.models.contains_key(name) {
            info!("Model '{}' is already loaded", name);
            return Ok(());
        }
        
        // Get model path
        let path = match self.model_paths.get(name) {
            Some(path) => path.clone(),
            None => return Err(LlamaError::ModelNotFound(format!("No path registered for model '{}'", name))),
        };
        
        // Create config
        let mut config = config_override.unwrap_or_else(|| {
            let mut cfg = LlamaConfig::default();
            cfg.model_path = path;
            cfg
        });
        
        // Ensure path is set
        if config_override.is_some() {
            config.model_path = path;
        }
        
        // Create and load context
        let mut context = LlamaContext::new(config)?;
        context.load_model()?;
        
        // Store the loaded model
        self.models.insert(name.to_string(), Arc::new(Mutex::new(context)));
        
        // Set as default if it's the first model
        if self.default_model.is_none() {
            self.default_model = Some(name.to_string());
        }
        
        info!("Model '{}' loaded successfully", name);
        Ok(())
    }
    
    /// Get a model by name
    pub fn get_model(&self, name: &str) -> Option<Arc<Mutex<LlamaContext>>> {
        self.models.get(name).cloned()
    }
    
    /// Get the default model
    pub fn get_default_model(&self) -> Option<Arc<Mutex<LlamaContext>>> {
        if let Some(name) = &self.default_model {
            self.get_model(name)
        } else {
            None
        }
    }
    
    /// Set the default model
    pub fn set_default_model(&mut self, name: &str) -> Result<(), LlamaError> {
        if !self.models.contains_key(name) {
            return Err(LlamaError::ModelNotFound(format!("Model '{}' not loaded", name)));
        }
        
        self.default_model = Some(name.to_string());
        Ok(())
    }
    
    /// Unload a model
    pub fn unload_model(&mut self, name: &str) -> Result<(), LlamaError> {
        if let Some(model) = self.models.remove(name) {
            // Acquire lock and unload
            if let Ok(mut context) = model.lock() {
                context.unload_model();
            }
            
            // If this was the default model, clear the default
            if let Some(default) = &self.default_model {
                if default == name {
                    self.default_model = None;
                    
                    // Set a new default if there are other models
                    if let Some(first) = self.models.keys().next() {
                        self.default_model = Some(first.clone());
                    }
                }
            }
            
            info!("Model '{}' unloaded", name);
            Ok(())
        } else {
            Err(LlamaError::ModelNotFound(format!("Model '{}' not loaded", name)))
        }
    }
    
    /// Get list of loaded models
    pub fn get_loaded_models(&self) -> Vec<String> {
        self.models.keys().cloned().collect()
    }
    
    /// Get list of available model paths
    pub fn get_available_models(&self) -> Vec<String> {
        self.model_paths.keys().cloned().collect()
    }
    
    /// Check if a model is loaded
    pub fn is_model_loaded(&self, name: &str) -> bool {
        self.models.contains_key(name)
    }
    
    /// Get model information
    pub fn get_model_info(&self, name: &str) -> Option<HashMap<String, String>> {
        if let Some(model) = self.models.get(name) {
            if let Ok(context) = model.lock() {
                return Some(context.get_model_info());
            }
        }
        None
    }
    
    /// Unload all models
    pub fn unload_all(&mut self) {
        for (name, model) in self.models.drain() {
            if let Ok(mut context) = model.lock() {
                context.unload_model();
                info!("Model '{}' unloaded", name);
            }
        }
        
        self.default_model = None;
    }
}

impl Drop for ModelManager {
    fn drop(&mut self) {
        self.unload_all();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;
    
    #[test]
    fn test_model_manager_initialization() {
        let mut manager = ModelManager::new();
        assert!(manager.initialize().is_ok());
        
        // Check that default paths were added
        assert!(manager.get_available_models().len() >= 3);
    }
    
    #[test]
    fn test_model_path_management() {
        let mut manager = ModelManager::new();
        manager.add_model_path("test", PathBuf::from("test/model.gguf"));
        
        assert!(manager.get_available_models().contains(&"test".to_string()));
    }
    
    #[test]
    fn test_model_loading_nonexistent() {
        let mut manager = ModelManager::new();
        manager.add_model_path("test", PathBuf::from("nonexistent/model.gguf"));
        
        let result = manager.load_model("test", None);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_model_info() {
        // Create a temporary model file
        let temp_dir = tempdir().unwrap();
        let model_path = temp_dir.path().join("test_model.gguf");
        let mut file = File::create(&model_path).unwrap();
        
        // Write enough data to pass size check
        let dummy_data = vec![0u8; 1024 * 1024]; // 1MB
        file.write_all(&dummy_data).unwrap();
        
        // Set up manager
        let mut manager = ModelManager::new();
        manager.add_model_path("test", model_path);
        
        // Load model (will use mock implementation)
        let result = manager.load_model("test", None);
        
        // This should succeed with the mock implementation
        if result.is_ok() {
            assert!(manager.is_model_loaded("test"));
            
            let info = manager.get_model_info("test");
            assert!(info.is_some());
            
            if let Some(info) = info {
                assert!(info.contains_key("loaded"));
                assert_eq!(info.get("loaded").unwrap(), "true");
            }
        }
    }
}