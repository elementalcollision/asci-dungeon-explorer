use std::collections::{HashMap, VecDeque};
use std::path::{Path, PathBuf};
use std::fs;
use std::io;
use std::sync::{Arc, Mutex};
use serde::{Serialize, Deserialize};
use log::{info, warn, error};

use super::dialogue_system_trait::{
    DialogueSystem, DialogueEntry, DialogueContext, CharacterPersona,
    DialogueOptions, DialogueOption, DialogueEffect, DialogueRequirement, DialogueConfig
};
use super::llama_integration::LlamaError;
use super::model_manager::ModelManager;

/// Conversation history entry with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationHistoryEntry {
    pub entry: DialogueEntry,
    pub context_snapshot: Option<DialogueContext>,
    pub timestamp: u64,
}

/// Conversation history for tracking dialogue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationHistory {
    pub character_id: String,
    pub entries: VecDeque<ConversationHistoryEntry>,
    pub max_entries: usize,
    pub metadata: HashMap<String, String>,
}

impl ConversationHistory {
    /// Create a new conversation history
    pub fn new(character_id: &str, max_entries: usize) -> Self {
        ConversationHistory {
            character_id: character_id.to_string(),
            entries: VecDeque::new(),
            max_entries,
            metadata: HashMap::new(),
        }
    }
    
    /// Add an entry to the conversation history
    pub fn add_entry(&mut self, entry: DialogueEntry, context: Option<&DialogueContext>) {
        let history_entry = ConversationHistoryEntry {
            entry,
            context_snapshot: context.cloned(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        };
        
        self.entries.push_back(history_entry);
        
        // Trim history if it exceeds max entries
        while self.entries.len() > self.max_entries {
            self.entries.pop_front();
        }
    }
    
    /// Get the last n entries
    pub fn get_last_entries(&self, n: usize) -> Vec<&DialogueEntry> {
        self.entries.iter()
            .rev()
            .take(n)
            .map(|entry| &entry.entry)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect()
    }
    
    /// Clear the conversation history
    pub fn clear(&mut self) {
        self.entries.clear();
    }
    
    /// Save the conversation history to a file
    pub fn save_to_file(&self, path: &Path) -> io::Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json)?;
        Ok(())
    }
    
    /// Load the conversation history from a file
    pub fn load_from_file(path: &Path) -> io::Result<Self> {
        let json = fs::read_to_string(path)?;
        let history: ConversationHistory = serde_json::from_str(&json)?;
        Ok(history)
    }
}

/// Manager for character conversations
pub struct ConversationManager {
    model_manager: Arc<Mutex<ModelManager>>,
    dialogue_systems: HashMap<String, Box<dyn DialogueSystem + Send + Sync>>,
    active_conversations: HashMap<String, DialogueContext>,
    conversation_histories: HashMap<String, ConversationHistory>,
    personas: HashMap<String, CharacterPersona>,
    persona_directory: PathBuf,
    history_directory: PathBuf,
    config: DialogueConfig,
}

impl ConversationManager {
    /// Create a new conversation manager
    pub fn new(model_manager: Arc<Mutex<ModelManager>>) -> Self {
        ConversationManager {
            model_manager,
            dialogue_systems: HashMap::new(),
            active_conversations: HashMap::new(),
            conversation_histories: HashMap::new(),
            personas: HashMap::new(),
            persona_directory: PathBuf::from("data/personas"),
            history_directory: PathBuf::from("data/conversation_history"),
            config: DialogueConfig::default(),
        }
    }
    
    /// Initialize the conversation manager
    pub fn initialize(&mut self) -> Result<(), LlamaError> {
        // Create directories if they don't exist
        if !self.persona_directory.exists() {
            if let Err(e) = fs::create_dir_all(&self.persona_directory) {
                return Err(LlamaError::InitializationFailed(
                    format!("Failed to create persona directory: {}", e)
                ));
            }
        }
        
        if !self.history_directory.exists() {
            if let Err(e) = fs::create_dir_all(&self.history_directory) {
                return Err(LlamaError::InitializationFailed(
                    format!("Failed to create history directory: {}", e)
                ));
            }
        }
        
        // Load all personas
        self.load_all_personas()?;
        
        Ok(())
    }
    
    /// Register a dialogue system
    pub fn register_dialogue_system<T: DialogueSystem + Send + Sync + 'static>(
        &mut self,
        name: &str,
        system: T
    ) {
        self.dialogue_systems.insert(name.to_string(), Box::new(system));
    }
    
    /// Get a dialogue system by name
    pub fn get_dialogue_system(&self, name: &str) -> Option<&Box<dyn DialogueSystem + Send + Sync>> {
        self.dialogue_systems.get(name)
    }
    
    /// Start a conversation with a character
    pub fn start_conversation(&mut self, character_id: &str, location: &str) -> Result<DialogueContext, LlamaError> {
        // Check if the conversation is already active
        if let Some(context) = self.active_conversations.get(character_id) {
            return Ok(context.clone());
        }
        
        // Get the character persona
        let persona = match self.personas.get(character_id) {
            Some(persona) => persona.clone(),
            None => {
                // Try to load the persona
                match self.load_persona(character_id) {
                    Ok(persona) => persona,
                    Err(_) => return Err(LlamaError::InvalidParameters(
                        format!("Character persona not found: {}", character_id)
                    )),
                }
            }
        };
        
        // Get the default dialogue system
        let dialogue_system = match self.dialogue_systems.get("default") {
            Some(system) => system,
            None => return Err(LlamaError::InitializationFailed(
                "No default dialogue system registered".to_string()
            )),
        };
        
        // Create a new dialogue context
        let context = dialogue_system.create_context(&persona, location);
        
        // Create or load conversation history
        if !self.conversation_histories.contains_key(character_id) {
            let history_path = self.history_directory.join(format!("{}.json", character_id));
            
            if history_path.exists() {
                match ConversationHistory::load_from_file(&history_path) {
                    Ok(history) => {
                        self.conversation_histories.insert(character_id.to_string(), history);
                    },
                    Err(e) => {
                        warn!("Failed to load conversation history for {}: {}", character_id, e);
                        self.conversation_histories.insert(
                            character_id.to_string(),
                            ConversationHistory::new(character_id, 100)
                        );
                    }
                }
            } else {
                self.conversation_histories.insert(
                    character_id.to_string(),
                    ConversationHistory::new(character_id, 100)
                );
            }
        }
        
        // Store the active conversation
        self.active_conversations.insert(character_id.to_string(), context.clone());
        
        Ok(context)
    }
    
    /// End a conversation with a character
    pub fn end_conversation(&mut self, character_id: &str) -> Result<(), LlamaError> {
        // Remove the active conversation
        if let Some(context) = self.active_conversations.remove(character_id) {
            // Save the conversation history
            if let Some(history) = self.conversation_histories.get(character_id) {
                let history_path = self.history_directory.join(format!("{}.json", character_id));
                
                if let Err(e) = history.save_to_file(&history_path) {
                    warn!("Failed to save conversation history for {}: {}", character_id, e);
                }
            }
            
            Ok(())
        } else {
            Err(LlamaError::InvalidParameters(
                format!("No active conversation with character: {}", character_id)
            ))
        }
    }
    
    /// Generate a response from a character
    pub fn generate_response(&mut self, character_id: &str, player_input: &str) -> Result<DialogueEntry, LlamaError> {
        // Get the active conversation
        let context = match self.active_conversations.get(character_id) {
            Some(context) => context.clone(),
            None => return Err(LlamaError::InvalidParameters(
                format!("No active conversation with character: {}", character_id)
            )),
        };
        
        // Get the dialogue system
        let dialogue_system = match self.dialogue_systems.get("default") {
            Some(system) => system,
            None => return Err(LlamaError::InitializationFailed(
                "No default dialogue system registered".to_string()
            )),
        };
        
        // Add player input to history
        let player_entry = DialogueEntry {
            speaker: "Player".to_string(),
            text: player_input.to_string(),
            emotion: None,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            metadata: HashMap::new(),
        };
        
        if let Some(history) = self.conversation_histories.get_mut(character_id) {
            history.add_entry(player_entry.clone(), Some(&context));
        }
        
        // Generate response
        let response = match dialogue_system.generate_response(&context, player_input) {
            Ok(response) => response,
            Err(e) => {
                warn!("Failed to generate response for {}: {}", character_id, e);
                dialogue_system.fallback_response(&context)
            }
        };
        
        // Add response to history
        if let Some(history) = self.conversation_histories.get_mut(character_id) {
            history.add_entry(response.clone(), Some(&context));
        }
        
        // Update the active conversation
        if let Some(context) = self.active_conversations.get_mut(character_id) {
            context.history.push(player_entry);
            context.history.push(response.clone());
            
            // Trim history if it exceeds max length
            let max_history = dialogue_system.get_config().max_history_length;
            while context.history.len() > max_history {
                context.history.remove(0);
            }
        }
        
        Ok(response)
    }
    
    /// Generate dialogue options for the player
    pub fn generate_options(&self, character_id: &str, current_topic: &str) -> Result<DialogueOptions, LlamaError> {
        // Get the active conversation
        let context = match self.active_conversations.get(character_id) {
            Some(context) => context,
            None => return Err(LlamaError::InvalidParameters(
                format!("No active conversation with character: {}", character_id)
            )),
        };
        
        // Get the dialogue system
        let dialogue_system = match self.dialogue_systems.get("default") {
            Some(system) => system,
            None => return Err(LlamaError::InitializationFailed(
                "No default dialogue system registered".to_string()
            )),
        };
        
        // Generate options
        dialogue_system.generate_options(context, current_topic)
    }
    
    /// Select a dialogue option
    pub fn select_option(&mut self, character_id: &str, option_index: usize) -> Result<DialogueEntry, LlamaError> {
        // Get the active conversation
        let context = match self.active_conversations.get_mut(character_id) {
            Some(context) => context,
            None => return Err(LlamaError::InvalidParameters(
                format!("No active conversation with character: {}", character_id)
            )),
        };
        
        // Get the dialogue system
        let dialogue_system = match self.dialogue_systems.get("default") {
            Some(system) => system,
            None => return Err(LlamaError::InitializationFailed(
                "No default dialogue system registered".to_string()
            )),
        };
        
        // Get the last generated options from metadata
        let options_str = context.metadata.get("last_options")
            .ok_or_else(|| LlamaError::InvalidParameters("No dialogue options available".to_string()))?;
        
        let options: DialogueOptions = serde_json::from_str(options_str)
            .map_err(|e| LlamaError::InvalidParameters(format!("Failed to parse dialogue options: {}", e)))?;
        
        // Check if the option index is valid
        if option_index >= options.options.len() {
            return Err(LlamaError::InvalidParameters(
                format!("Invalid option index: {}", option_index)
            ));
        }
        
        // Get the selected option
        let option = &options.options[option_index];
        
        // Apply option effects
        let effects = dialogue_system.apply_option_effects(context, option);
        
        // Generate response based on the selected option
        let player_text = option.text.clone();
        
        // Add player input to history
        let player_entry = DialogueEntry {
            speaker: "Player".to_string(),
            text: player_text.clone(),
            emotion: None,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            metadata: HashMap::new(),
        };
        
        if let Some(history) = self.conversation_histories.get_mut(character_id) {
            history.add_entry(player_entry.clone(), Some(context));
        }
        
        // Update context history
        context.history.push(player_entry);
        
        // Generate response
        let response = match dialogue_system.generate_response(context, &player_text) {
            Ok(response) => response,
            Err(e) => {
                warn!("Failed to generate response for {}: {}", character_id, e);
                dialogue_system.fallback_response(context)
            }
        };
        
        // Add response to history
        if let Some(history) = self.conversation_histories.get_mut(character_id) {
            history.add_entry(response.clone(), Some(context));
        }
        
        // Update context history
        context.history.push(response.clone());
        
        // Trim history if it exceeds max length
        let max_history = dialogue_system.get_config().max_history_length;
        while context.history.len() > max_history {
            context.history.remove(0);
        }
        
        Ok(response)
    }
    
    /// Load a character persona
    pub fn load_persona(&mut self, character_id: &str) -> Result<CharacterPersona, LlamaError> {
        // Check if the persona is already loaded
        if let Some(persona) = self.personas.get(character_id) {
            return Ok(persona.clone());
        }
        
        // Load the persona from file
        let persona_path = self.persona_directory.join(format!("{}.json", character_id));
        
        if !persona_path.exists() {
            return Err(LlamaError::ModelNotFound(
                format!("Character persona file not found: {:?}", persona_path)
            ));
        }
        
        let json = match fs::read_to_string(&persona_path) {
            Ok(json) => json,
            Err(e) => return Err(LlamaError::ModelLoadFailed(
                format!("Failed to read persona file: {}", e)
            )),
        };
        
        let persona: CharacterPersona = match serde_json::from_str(&json) {
            Ok(persona) => persona,
            Err(e) => return Err(LlamaError::ModelLoadFailed(
                format!("Failed to parse persona JSON: {}", e)
            )),
        };
        
        // Store the persona
        self.personas.insert(character_id.to_string(), persona.clone());
        
        Ok(persona)
    }
    
    /// Save a character persona
    pub fn save_persona(&self, persona: &CharacterPersona) -> Result<(), LlamaError> {
        let persona_path = self.persona_directory.join(format!("{}.json", persona.id));
        
        let json = match serde_json::to_string_pretty(persona) {
            Ok(json) => json,
            Err(e) => return Err(LlamaError::InvalidParameters(
                format!("Failed to serialize persona: {}", e)
            )),
        };
        
        match fs::write(&persona_path, json) {
            Ok(_) => Ok(()),
            Err(e) => Err(LlamaError::ResourceExhausted(
                format!("Failed to write persona file: {}", e)
            )),
        }
    }
    
    /// Create a new character persona
    pub fn create_persona(&mut self, name: &str, description: &str) -> Result<CharacterPersona, LlamaError> {
        // Get the dialogue system
        let dialogue_system = match self.dialogue_systems.get("default") {
            Some(system) => system,
            None => return Err(LlamaError::InitializationFailed(
                "No default dialogue system registered".to_string()
            )),
        };
        
        // Create the persona
        let persona = dialogue_system.create_persona(name, description);
        
        // Save the persona
        self.save_persona(&persona)?;
        
        // Store the persona
        self.personas.insert(persona.id.clone(), persona.clone());
        
        Ok(persona)
    }
    
    /// Load all personas from the persona directory
    fn load_all_personas(&mut self) -> Result<(), LlamaError> {
        if !self.persona_directory.exists() {
            return Ok(());
        }
        
        let entries = match fs::read_dir(&self.persona_directory) {
            Ok(entries) => entries,
            Err(e) => return Err(LlamaError::InitializationFailed(
                format!("Failed to read persona directory: {}", e)
            )),
        };
        
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                
                if path.is_file() && path.extension().map_or(false, |ext| ext == "json") {
                    if let Some(stem) = path.file_stem() {
                        if let Some(character_id) = stem.to_str() {
                            match self.load_persona(character_id) {
                                Ok(_) => info!("Loaded persona: {}", character_id),
                                Err(e) => warn!("Failed to load persona {}: {}", character_id, e),
                            }
                        }
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Get all loaded personas
    pub fn get_all_personas(&self) -> Vec<CharacterPersona> {
        self.personas.values().cloned().collect()
    }
    
    /// Get conversation history for a character
    pub fn get_conversation_history(&self, character_id: &str) -> Option<&ConversationHistory> {
        self.conversation_histories.get(character_id)
    }
    
    /// Set the dialogue configuration
    pub fn set_config(&mut self, config: DialogueConfig) {
        self.config = config.clone();
        
        // Update all dialogue systems
        for system in self.dialogue_systems.values_mut() {
            system.set_config(config.clone());
        }
    }
    
    /// Get the dialogue configuration
    pub fn get_config(&self) -> &DialogueConfig {
        &self.config
    }
}