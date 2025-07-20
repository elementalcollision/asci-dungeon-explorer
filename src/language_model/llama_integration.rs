use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;
use std::sync::mpsc::{self, Receiver, Sender};
use log::{info, warn, error};

#[cfg(feature = "language_model")]
use llama_cpp_rs::{
    LLama, LLamaModel, LLamaParams, LLamaToken, LLamaContext,
    LLamaTokenDataArray, LLamaTokenData, LLamaContextParams,
    llama_token_data_array_t, llama_token_data_t, llama_token_t,
    llama_context_t, llama_model_t
};

/// Llama.cpp integration errors
#[derive(Debug, Clone)]
pub enum LlamaError {
    ModelNotFound(String),
    ModelLoadFailed(String),
    InitializationFailed(String),
    InferenceFailed(String),
    InvalidParameters(String),
    ResourceExhausted(String),
    Timeout(String),
}

impl std::fmt::Display for LlamaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LlamaError::ModelNotFound(msg) => write!(f, "Model not found: {}", msg),
            LlamaError::ModelLoadFailed(msg) => write!(f, "Model load failed: {}", msg),
            LlamaError::InitializationFailed(msg) => write!(f, "Initialization failed: {}", msg),
            LlamaError::InferenceFailed(msg) => write!(f, "Inference failed: {}", msg),
            LlamaError::InvalidParameters(msg) => write!(f, "Invalid parameters: {}", msg),
            LlamaError::ResourceExhausted(msg) => write!(f, "Resource exhausted: {}", msg),
            LlamaError::Timeout(msg) => write!(f, "Timeout: {}", msg),
        }
    }
}

impl std::error::Error for LlamaError {}

/// Llama model configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlamaConfig {
    pub model_path: PathBuf,
    pub context_size: u32,
    pub batch_size: u32,
    pub threads: u32,
    pub gpu_layers: u32,
    pub use_mmap: bool,
    pub use_mlock: bool,
    pub seed: i32,
    pub temperature: f32,
    pub top_p: f32,
    pub top_k: u32,
    pub repeat_penalty: f32,
    pub max_tokens: u32,
    pub timeout_seconds: u64,
}

impl Default for LlamaConfig {
    fn default() -> Self {
        LlamaConfig {
            model_path: PathBuf::from("models/llama-2-7b-chat.gguf"),
            context_size: 2048,
            batch_size: 512,
            threads: 4,
            gpu_layers: 0,
            use_mmap: true,
            use_mlock: false,
            seed: -1,
            temperature: 0.7,
            top_p: 0.9,
            top_k: 40,
            repeat_penalty: 1.1,
            max_tokens: 256,
            timeout_seconds: 30,
        }
    }
}

/// Llama inference request
#[derive(Debug, Clone)]
pub struct LlamaRequest {
    pub id: String,
    pub prompt: String,
    pub config: Option<LlamaConfig>,
    pub system_prompt: Option<String>,
    pub stop_sequences: Vec<String>,
}

/// Llama inference response
#[derive(Debug, Clone)]
pub struct LlamaResponse {
    pub id: String,
    pub text: String,
    pub tokens_generated: u32,
    pub processing_time_ms: u64,
    pub success: bool,
    pub error: Option<LlamaError>,
}

/// Llama context wrapper for llama.cpp
pub struct LlamaContext {
    config: LlamaConfig,
    model_loaded: bool,
    context_size: u32,
    vocab_size: u32,
    
    #[cfg(feature = "language_model")]
    llama_model: Option<Box<LLamaModel>>,
    
    #[cfg(feature = "language_model")]
    llama_context: Option<Box<LLamaContext>>,
}

impl LlamaContext {
    /// Create a new Llama context
    pub fn new(config: LlamaConfig) -> Result<Self, LlamaError> {
        // Validate model file
        if !config.model_path.exists() {
            return Err(LlamaError::ModelNotFound(
                format!("Model file not found: {:?}", config.model_path)
            ));
        }
        
        // Validate configuration
        if config.context_size == 0 {
            return Err(LlamaError::InvalidParameters("Context size cannot be zero".to_string()));
        }
        
        if config.threads == 0 {
            return Err(LlamaError::InvalidParameters("Thread count cannot be zero".to_string()));
        }
        
        #[cfg(not(feature = "language_model"))]
        {
            Ok(LlamaContext {
                config,
                model_loaded: false,
                context_size: 0,
                vocab_size: 0,
            })
        }
        
        #[cfg(feature = "language_model")]
        {
            Ok(LlamaContext {
                config,
                model_loaded: false,
                context_size: 0,
                vocab_size: 0,
                llama_model: None,
                llama_context: None,
            })
        }
    }
    
    /// Load the model
    pub fn load_model(&mut self) -> Result<(), LlamaError> {
        if self.model_loaded {
            return Ok(());
        }
        
        info!("Loading model from {:?}", self.config.model_path);
        
        #[cfg(not(feature = "language_model"))]
        {
            // Fallback implementation when llama.cpp is not available
            warn!("llama.cpp integration is not enabled. Using mock implementation.");
            std::thread::sleep(std::time::Duration::from_millis(100));
            
            self.model_loaded = true;
            self.context_size = self.config.context_size;
            self.vocab_size = 32000; // Typical vocab size for Llama models
            
            info!("Mock model loaded successfully");
            return Ok(());
        }
        
        #[cfg(feature = "language_model")]
        {
            // Create llama.cpp model parameters
            let model_path = self.config.model_path.to_string_lossy().to_string();
            
            let model_params = LLamaParams::default()
                .use_mmap(self.config.use_mmap)
                .use_mlock(self.config.use_mlock)
                .n_gpu_layers(self.config.gpu_layers as i32);
            
            // Load the model
            match LLamaModel::load_from_file(&model_path, &model_params) {
                Ok(model) => {
                    // Create context parameters
                    let context_params = LLamaContextParams::default()
                        .n_ctx(self.config.context_size as i32)
                        .seed(self.config.seed)
                        .n_batch(self.config.batch_size as i32)
                        .n_threads(self.config.threads as i32);
                    
                    // Create context
                    match LLamaContext::new(&model, &context_params) {
                        Ok(context) => {
                            self.llama_model = Some(Box::new(model));
                            self.llama_context = Some(Box::new(context));
                            self.model_loaded = true;
                            self.context_size = self.config.context_size;
                            
                            // Get vocabulary size
                            if let Some(ctx) = &self.llama_context {
                                self.vocab_size = ctx.n_vocab() as u32;
                            }
                            
                            info!("Model loaded successfully with vocab size: {}", self.vocab_size);
                            Ok(())
                        },
                        Err(e) => {
                            error!("Failed to create context: {}", e);
                            Err(LlamaError::InitializationFailed(format!("Context creation failed: {}", e)))
                        }
                    }
                },
                Err(e) => {
                    error!("Failed to load model: {}", e);
                    Err(LlamaError::ModelLoadFailed(format!("Model loading failed: {}", e)))
                }
            }
        }
    }
    
    /// Generate text from a prompt
    pub fn generate(&self, request: &LlamaRequest) -> Result<LlamaResponse, LlamaError> {
        if !self.model_loaded {
            return Err(LlamaError::InitializationFailed("Model not loaded".to_string()));
        }
        
        let start_time = std::time::Instant::now();
        
        #[cfg(not(feature = "language_model"))]
        {
            // Fallback implementation when llama.cpp is not available
            let response_text = self.generate_mock_response(&request.prompt);
            let processing_time = start_time.elapsed().as_millis() as u64;
            
            return Ok(LlamaResponse {
                id: request.id.clone(),
                text: response_text,
                tokens_generated: 50, // Mock token count
                processing_time_ms: processing_time,
                success: true,
                error: None,
            });
        }
        
        #[cfg(feature = "language_model")]
        {
            // Get the context
            let context = match &self.llama_context {
                Some(ctx) => ctx,
                None => return Err(LlamaError::InitializationFailed("Context not initialized".to_string())),
            };
            
            // Format the prompt with system instructions if provided
            let formatted_prompt = match &request.system_prompt {
                Some(system) => format!("System: {}\n\nUser: {}\n\nAssistant:", system, request.prompt),
                None => format!("User: {}\n\nAssistant:", request.prompt),
            };
            
            // Tokenize the prompt
            let tokens = match context.tokenize(&formatted_prompt, false) {
                Ok(tokens) => tokens,
                Err(e) => return Err(LlamaError::InferenceFailed(format!("Tokenization failed: {}", e))),
            };
            
            // Set up generation parameters
            let config = request.config.as_ref().unwrap_or(&self.config);
            
            // Initialize the context with the prompt
            if let Err(e) = context.eval(&tokens, 0) {
                return Err(LlamaError::InferenceFailed(format!("Context evaluation failed: {}", e)));
            }
            
            // Generate tokens
            let mut generated_text = String::new();
            let mut tokens_generated = 0;
            let max_tokens = config.max_tokens as usize;
            let timeout = std::time::Duration::from_secs(config.timeout_seconds);
            let start = std::time::Instant::now();
            
            // Create stop token sequences
            let stop_sequences = request.stop_sequences.iter()
                .map(|s| context.tokenize(s, false).unwrap_or_default())
                .collect::<Vec<_>>();
            
            // Token generation loop
            while tokens_generated < max_tokens && start.elapsed() < timeout {
                // Sample a token
                let token = match context.sample(
                    config.top_k as i32,
                    config.top_p,
                    config.temperature,
                    config.repeat_penalty,
                ) {
                    Ok(token) => token,
                    Err(e) => return Err(LlamaError::InferenceFailed(format!("Token sampling failed: {}", e))),
                };
                
                // Convert token to string
                let piece = match context.token_to_piece(token) {
                    Ok(piece) => piece,
                    Err(e) => return Err(LlamaError::InferenceFailed(format!("Token to piece conversion failed: {}", e))),
                };
                
                // Add to generated text
                generated_text.push_str(&piece);
                tokens_generated += 1;
                
                // Evaluate the new token
                if let Err(e) = context.eval(&[token], 0) {
                    return Err(LlamaError::InferenceFailed(format!("Token evaluation failed: {}", e)));
                }
                
                // Check for stop sequences
                let should_stop = stop_sequences.iter().any(|stop_seq| {
                    if stop_seq.is_empty() {
                        return false;
                    }
                    
                    // Get the last n tokens where n is the length of the stop sequence
                    let n = stop_seq.len();
                    if tokens_generated < n {
                        return false;
                    }
                    
                    // Check if the last n tokens match the stop sequence
                    let last_n_tokens = context.get_last_tokens(n as i32);
                    last_n_tokens == *stop_seq
                });
                
                if should_stop {
                    break;
                }
            }
            
            // Check for timeout
            if start.elapsed() >= timeout {
                warn!("Generation timed out after {} seconds", config.timeout_seconds);
            }
            
            let processing_time = start_time.elapsed().as_millis() as u64;
            
            Ok(LlamaResponse {
                id: request.id.clone(),
                text: generated_text,
                tokens_generated: tokens_generated as u32,
                processing_time_ms: processing_time,
                success: true,
                error: None,
            })
        }
    }
    
    /// Generate a mock response (placeholder for when llama.cpp is not available)
    fn generate_mock_response(&self, prompt: &str) -> String {
        // This is a placeholder implementation used when llama.cpp is not available
        if prompt.contains("hello") || prompt.contains("greet") {
            "Greetings, adventurer! How may I assist you on your journey?".to_string()
        } else if prompt.contains("quest") || prompt.contains("mission") {
            "I have heard tales of ancient treasures hidden in the depths below. Perhaps you seek such challenges?".to_string()
        } else if prompt.contains("help") {
            "Fear not, for I am here to guide you. What knowledge do you seek?".to_string()
        } else if prompt.contains("goodbye") || prompt.contains("farewell") {
            "May your path be safe and your blade stay sharp. Farewell!".to_string()
        } else {
            "I understand your words, though the meaning eludes me. Could you speak more plainly?".to_string()
        }
    }
    
    /// Check if model is loaded
    pub fn is_loaded(&self) -> bool {
        self.model_loaded
    }
    
    /// Get model information
    pub fn get_model_info(&self) -> HashMap<String, String> {
        let mut info = HashMap::new();
        info.insert("model_path".to_string(), self.config.model_path.to_string_lossy().to_string());
        info.insert("context_size".to_string(), self.context_size.to_string());
        info.insert("vocab_size".to_string(), self.vocab_size.to_string());
        info.insert("loaded".to_string(), self.model_loaded.to_string());
        
        #[cfg(feature = "language_model")]
        {
            if let Some(ctx) = &self.llama_context {
                info.insert("n_ctx".to_string(), ctx.n_ctx().to_string());
                info.insert("n_embd".to_string(), ctx.n_embd().to_string());
                info.insert("n_vocab".to_string(), ctx.n_vocab().to_string());
                info.insert("n_batch".to_string(), ctx.n_batch().to_string());
            }
        }
        
        info
    }
    
    /// Unload the model
    pub fn unload_model(&mut self) {
        if self.model_loaded {
            #[cfg(feature = "language_model")]
            {
                // Free llama.cpp resources
                self.llama_context = None;
                self.llama_model = None;
            }
            
            self.model_loaded = false;
            self.context_size = 0;
            self.vocab_size = 0;
            info!("Model unloaded");
        }
    }
}

impl Drop for LlamaContext {
    fn drop(&mut self) {
        self.unload_model();
    }
}

/// Llama model manager for handling multiple models and async inference
pub struct LlamaManager {
    contexts: HashMap<String, Arc<Mutex<LlamaContext>>>,
    request_sender: Option<Sender<LlamaRequest>>,
    response_receiver: Option<Receiver<LlamaResponse>>,
    worker_handle: Option<thread::JoinHandle<()>>,
    default_config: LlamaConfig,
}

impl LlamaManager {
    /// Create a new Llama manager
    pub fn new() -> Self {
        LlamaManager {
            contexts: HashMap::new(),
            request_sender: None,
            response_receiver: None,
            worker_handle: None,
            default_config: LlamaConfig::default(),
        }
    }
    
    /// Initialize the manager with a default model
    pub fn initialize(&mut self, config: LlamaConfig) -> Result<(), LlamaError> {
        self.default_config = config.clone();
        
        // Create and load the default context
        let mut context = LlamaContext::new(config)?;
        context.load_model()?;
        
        self.contexts.insert("default".to_string(), Arc::new(Mutex::new(context)));
        
        // Set up async processing
        self.setup_async_processing();
        
        Ok(())
    }
    
    /// Set up async processing for inference requests
    fn setup_async_processing(&mut self) {
        let (request_tx, request_rx) = mpsc::channel::<LlamaRequest>();
        let (response_tx, response_rx) = mpsc::channel::<LlamaResponse>();
        
        self.request_sender = Some(request_tx);
        self.response_receiver = Some(response_rx);
        
        // Clone contexts for the worker thread
        let contexts = self.contexts.clone();
        
        // Spawn worker thread for processing requests
        let handle = thread::spawn(move || {
            while let Ok(request) = request_rx.recv() {
                let response = if let Some(context_arc) = contexts.get("default") {
                    match context_arc.lock() {
                        Ok(context) => context.generate(&request),
                        Err(_) => Err(LlamaError::ResourceExhausted("Context lock failed".to_string())),
                    }
                } else {
                    Err(LlamaError::InitializationFailed("No default context".to_string()))
                };
                
                let final_response = match response {
                    Ok(resp) => resp,
                    Err(error) => LlamaResponse {
                        id: request.id,
                        text: "I apologize, but I cannot respond at this time.".to_string(),
                        tokens_generated: 0,
                        processing_time_ms: 0,
                        success: false,
                        error: Some(error),
                    },
                };
                
                if response_tx.send(final_response).is_err() {
                    break; // Receiver dropped, exit thread
                }
            }
        });
        
        self.worker_handle = Some(handle);
    }
    
    /// Submit an async inference request
    pub fn submit_request(&self, request: LlamaRequest) -> Result<(), LlamaError> {
        if let Some(sender) = &self.request_sender {
            sender.send(request).map_err(|_| {
                LlamaError::ResourceExhausted("Request queue full".to_string())
            })?;
            Ok(())
        } else {
            Err(LlamaError::InitializationFailed("Manager not initialized".to_string()))
        }
    }
    
    /// Try to receive a response (non-blocking)
    pub fn try_receive_response(&self) -> Option<LlamaResponse> {
        if let Some(receiver) = &self.response_receiver {
            receiver.try_recv().ok()
        } else {
            None
        }
    }
    
    /// Perform synchronous inference
    pub fn generate_sync(&self, prompt: &str) -> Result<String, LlamaError> {
        if let Some(context_arc) = self.contexts.get("default") {
            let context = context_arc.lock().map_err(|_| {
                LlamaError::ResourceExhausted("Context lock failed".to_string())
            })?;
            
            let request = LlamaRequest {
                id: "sync".to_string(),
                prompt: prompt.to_string(),
                config: None,
                system_prompt: None,
                stop_sequences: vec![],
            };
            
            let response = context.generate(&request)?;
            Ok(response.text)
        } else {
            Err(LlamaError::InitializationFailed("No default context".to_string()))
        }
    }
    
    /// Load a new model with a specific name
    pub fn load_model(&mut self, name: &str, config: LlamaConfig) -> Result<(), LlamaError> {
        let mut context = LlamaContext::new(config)?;
        context.load_model()?;
        
        self.contexts.insert(name.to_string(), Arc::new(Mutex::new(context)));
        Ok(())
    }
    
    /// Unload a model
    pub fn unload_model(&mut self, name: &str) -> bool {
        self.contexts.remove(name).is_some()
    }
    
    /// Get list of loaded models
    pub fn get_loaded_models(&self) -> Vec<String> {
        self.contexts.keys().cloned().collect()
    }
    
    /// Check if a model is loaded
    pub fn is_model_loaded(&self, name: &str) -> bool {
        if let Some(context_arc) = self.contexts.get(name) {
            if let Ok(context) = context_arc.lock() {
                context.is_loaded()
            } else {
                false
            }
        } else {
            false
        }
    }
    
    /// Get model information
    pub fn get_model_info(&self, name: &str) -> Option<HashMap<String, String>> {
        if let Some(context_arc) = self.contexts.get(name) {
            if let Ok(context) = context_arc.lock() {
                Some(context.get_model_info())
            } else {
                None
            }
        } else {
            None
        }
    }
    
    /// Shutdown the manager
    pub fn shutdown(&mut self) {
        // Close the request channel
        self.request_sender = None;
        
        // Wait for worker thread to finish
        if let Some(handle) = self.worker_handle.take() {
            let _ = handle.join();
        }
        
        // Clear contexts
        self.contexts.clear();
        self.response_receiver = None;
    }
}

impl Drop for LlamaManager {
    fn drop(&mut self) {
        self.shutdown();
    }
}

/// Utility functions for llama.cpp integration
pub mod utils {
    use super::*;
    
    /// Check if a model file exists and is valid
    pub fn validate_model_file(path: &Path) -> Result<(), LlamaError> {
        if !path.exists() {
            return Err(LlamaError::ModelNotFound(
                format!("Model file does not exist: {:?}", path)
            ));
        }
        
        if !path.is_file() {
            return Err(LlamaError::ModelNotFound(
                format!("Path is not a file: {:?}", path)
            ));
        }
        
        // Check file extension
        if let Some(extension) = path.extension() {
            let ext = extension.to_string_lossy().to_lowercase();
            if !["gguf", "ggml", "bin"].contains(&ext.as_str()) {
                return Err(LlamaError::InvalidParameters(
                    format!("Unsupported model format: {}", ext)
                ));
            }
        } else {
            return Err(LlamaError::InvalidParameters(
                "Model file has no extension".to_string()
            ));
        }
        
        // Check file size (should be at least 1MB for a valid model)
        if let Ok(metadata) = path.metadata() {
            if metadata.len() < 1024 * 1024 {
                return Err(LlamaError::InvalidParameters(
                    "Model file appears to be too small".to_string()
                ));
            }
        }
        
        Ok(())
    }
    
    /// Get recommended configuration for a model based on system resources
    pub fn get_recommended_config(model_path: PathBuf) -> LlamaConfig {
        let mut config = LlamaConfig::default();
        config.model_path = model_path;
        
        // Adjust based on available system resources
        // In a real implementation, you would query system memory, CPU cores, etc.
        let available_threads = std::thread::available_parallelism()
            .map(|n| n.get() as u32)
            .unwrap_or(4);
        
        config.threads = (available_threads / 2).max(1).min(8);
        
        // Adjust context size based on available memory
        // This is a simplified heuristic
        config.context_size = 2048; // Conservative default
        
        config
    }
    
    /// Create a prompt with system instructions
    pub fn format_prompt(system_prompt: Option<&str>, user_prompt: &str) -> String {
        match system_prompt {
            Some(system) => format!("System: {}\n\nUser: {}\n\nAssistant:", system, user_prompt),
            None => format!("User: {}\n\nAssistant:", user_prompt),
        }
    }
    
    /// Clean up generated text
    pub fn clean_response(text: &str) -> String {
        text.trim()
            .lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>()
            .join(" ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_llama_config_default() {
        let config = LlamaConfig::default();
        assert_eq!(config.context_size, 2048);
        assert_eq!(config.temperature, 0.7);
        assert_eq!(config.threads, 4);
    }

    #[test]
    fn test_llama_context_creation() {
        // Create a temporary model file
        let temp_dir = tempdir().unwrap();
        let model_path = temp_dir.path().join("test_model.gguf");
        let mut file = File::create(&model_path).unwrap();
        writeln!(file, "dummy model content").unwrap();
        
        // Write enough data to pass size check
        let dummy_data = vec![0u8; 1024 * 1024]; // 1MB
        file.write_all(&dummy_data).unwrap();
        
        let mut config = LlamaConfig::default();
        config.model_path = model_path;
        
        let context = LlamaContext::new(config);
        assert!(context.is_ok());
    }

    #[test]
    fn test_llama_context_model_not_found() {
        let mut config = LlamaConfig::default();
        config.model_path = PathBuf::from("nonexistent_model.gguf");
        
        let context = LlamaContext::new(config);
        assert!(context.is_err());
        
        if let Err(LlamaError::ModelNotFound(_)) = context {
            // Expected error type
        } else {
            panic!("Expected ModelNotFound error");
        }
    }

    #[test]
    fn test_llama_manager() {
        let mut manager = LlamaManager::new();
        
        // Test that manager starts empty
        assert!(manager.get_loaded_models().is_empty());
        assert!(!manager.is_model_loaded("default"));
    }

    #[test]
    fn test_utils_validate_model_file() {
        // Test with non-existent file
        let result = utils::validate_model_file(Path::new("nonexistent.gguf"));
        assert!(result.is_err());
        
        // Test with valid file
        let temp_dir = tempdir().unwrap();
        let model_path = temp_dir.path().join("test_model.gguf");
        let mut file = File::create(&model_path).unwrap();
        let dummy_data = vec![0u8; 1024 * 1024]; // 1MB
        file.write_all(&dummy_data).unwrap();
        
        let result = utils::validate_model_file(&model_path);
        assert!(result.is_ok());
    }

    #[test]
    fn test_utils_format_prompt() {
        let prompt = utils::format_prompt(None, "Hello");
        assert!(prompt.contains("User: Hello"));
        
        let prompt_with_system = utils::format_prompt(Some("You are helpful"), "Hello");
        assert!(prompt_with_system.contains("System: You are helpful"));
        assert!(prompt_with_system.contains("User: Hello"));
    }

    #[test]
    fn test_utils_clean_response() {
        let messy_text = "  Hello  \n\n  World  \n  ";
        let cleaned = utils::clean_response(messy_text);
        assert_eq!(cleaned, "Hello World");
    }
}