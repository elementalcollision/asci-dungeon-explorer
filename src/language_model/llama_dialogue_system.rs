use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs;
use std::io;
use std::sync::{Arc, Mutex};
use serde::{Serialize, Deserialize};
use log::{info, warn, error};
use rand::{Rng, thread_rng};

use super::dialogue_system_trait::{
    DialogueSystem, DialogueEntry, DialogueContext, CharacterPersona,
    DialogueOptions, DialogueOption, DialogueEffect, DialogueRequirement, DialogueConfig
};
use super::llama_integration::{LlamaError, LlamaRequest, LlamaConfig};
use super::model_manager::ModelManager;

/// Llama-based dialogue system implementation
pub struct LlamaDialogueSystem {
    model_manager: Arc<Mutex<ModelManager>>,
    config: DialogueConfig,
    persona_directory: PathBuf,
}

impl LlamaDialogueSystem {
    /// Create a new Llama dialogue system
    pub fn new(model_manager: Arc<Mutex<ModelManager>>) -> Self {
        LlamaDialogueSystem {
            model_manager,
            config: DialogueConfig::default(),
            persona_directory: PathBuf::from("data/personas"),
        }
    }
    
    /// Set the persona directory
    pub fn set_persona_directory(&mut self, path: PathBuf) {
        self.persona_directory = path;
    }
    
    /// Build a prompt from the dialogue context
    fn build_prompt(&self, context: &DialogueContext, player_input: &str) -> String {
        let mut prompt = String::new();
        
        // Add character information
        prompt.push_str(&format!("Character: {}\n", context.character_name));
        prompt.push_str(&format!("Description: {}\n", context.character_description));
        prompt.push_str(&format!("Location: {}\n", context.location));
        prompt.push_str(&format!("Relationship with player: {}\n", self.relationship_description(context.relationship)));
        
        // Add traits
        if !context.traits.is_empty() {
            prompt.push_str("Traits: ");
            prompt.push_str(&context.traits.join(", "));
            prompt.push_str("\n");
        }
        
        // Add faction
        if let Some(faction) = &context.faction {
            prompt.push_str(&format!("Faction: {}\n", faction));
        }
        
        // Add knowledge
        if !context.knowledge.is_empty() {
            prompt.push_str("Knowledge:\n");
            for item in &context.knowledge {
                prompt.push_str(&format!("- {}\n", item));
            }
        }
        
        // Add conversation history
        prompt.push_str("\nConversation:\n");
        
        // Limit history to max_history_length
        let start_idx = if context.history.len() > self.config.max_history_length {
            context.history.len() - self.config.max_history_length
        } else {
            0
        };
        
        for entry in &context.history[start_idx..] {
            if let Some(emotion) = &entry.emotion {
                prompt.push_str(&format!("{} [{}]: {}\n", entry.speaker, emotion, entry.text));
            } else {
                prompt.push_str(&format!("{}: {}\n", entry.speaker, entry.text));
            }
        }
        
        // Add player input
        prompt.push_str(&format!("Player: {}\n", player_input));
        prompt.push_str(&format!("{}:", context.character_name));
        
        prompt
    }
    
    /// Build a prompt for generating dialogue options
    fn build_options_prompt(&self, context: &DialogueContext, current_topic: &str) -> String {
        let mut prompt = String::new();
        
        // Add character information
        prompt.push_str(&format!("Character: {}\n", context.character_name));
        prompt.push_str(&format!("Description: {}\n", context.character_description));
        prompt.push_str(&format!("Location: {}\n", context.location));
        prompt.push_str(&format!("Relationship with player: {}\n", self.relationship_description(context.relationship)));
        prompt.push_str(&format!("Current topic: {}\n", current_topic));
        
        // Add traits
        if !context.traits.is_empty() {
            prompt.push_str("Traits: ");
            prompt.push_str(&context.traits.join(", "));
            prompt.push_str("\n");
        }
        
        // Add knowledge
        if !context.knowledge.is_empty() {
            prompt.push_str("Knowledge:\n");
            for item in &context.knowledge {
                prompt.push_str(&format!("- {}\n", item));
            }
        }
        
        // Add conversation history (last few entries)
        prompt.push_str("\nRecent Conversation:\n");
        
        let history_len = context.history.len();
        let start_idx = if history_len > 5 { history_len - 5 } else { 0 };
        
        for entry in &context.history[start_idx..] {
            prompt.push_str(&format!("{}: {}\n", entry.speaker, entry.text));
        }
        
        // Request for dialogue options
        prompt.push_str("\nGenerate 3-4 dialogue options for the player to choose from when speaking to this character about the current topic. Each option should be a short sentence or question. Format the response as a numbered list:\n");
        prompt.push_str("1. [First dialogue option]\n");
        prompt.push_str("2. [Second dialogue option]\n");
        prompt.push_str("3. [Third dialogue option]\n");
        prompt.push_str("4. [Optional fourth dialogue option]");
        
        prompt
    }
    
    /// Parse dialogue options from generated text
    fn parse_options(&self, text: &str) -> DialogueOptions {
        let mut options = Vec::new();
        
        // Split by newlines and look for numbered options
        for line in text.lines() {
            let line = line.trim();
            
            // Look for lines starting with a number followed by a period or parenthesis
            if let Some(option_text) = line.strip_prefix(|c: char| c.is_ascii_digit() && c != '0')
                .and_then(|s| s.strip_prefix(|c: char| c == '.' || c == ')' || c == ':'))
            {
                let option_text = option_text.trim();
                if !option_text.is_empty() {
                    options.push(DialogueOption {
                        text: option_text.to_string(),
                        next_state: None,
                        effects: Vec::new(),
                        requirements: Vec::new(),
                        metadata: HashMap::new(),
                    });
                }
            }
        }
        
        // If no options were found, create a default one
        if options.is_empty() {
            options.push(DialogueOption {
                text: "Continue the conversation".to_string(),
                next_state: None,
                effects: Vec::new(),
                requirements: Vec::new(),
                metadata: HashMap::new(),
            });
        }
        
        DialogueOptions {
            options,
            timeout_seconds: Some(30),
            default_option: Some(0),
        }
    }
    
    /// Convert relationship value to description
    fn relationship_description(&self, relationship: i32) -> &'static str {
        match relationship {
            r if r < -75 => "Hostile",
            r if r < -50 => "Antagonistic",
            r if r < -25 => "Unfriendly",
            r if r < 0 => "Wary",
            r if r < 25 => "Neutral",
            r if r < 50 => "Friendly",
            r if r < 75 => "Trusting",
            _ => "Loyal",
        }
    }
    
    /// Parse the response from the model
    fn parse_response(&self, text: &str) -> (String, Option<String>) {
        let text = text.trim();
        
        // Extract emotion if present (format: [emotion] text)
        if text.starts_with('[') {
            if let Some(end_bracket) = text.find(']') {
                let emotion = text[1..end_bracket].trim().to_string();
                let remaining = text[end_bracket + 1..].trim().to_string();
                return (remaining, Some(emotion));
            }
        }
        
        (text.to_string(), None)
    }
}

impl DialogueSystem for LlamaDialogueSystem {
    /// Initialize the dialogue system
    fn initialize(&mut self) -> Result<(), LlamaError> {
        // Create persona directory if it doesn't exist
        if !self.persona_directory.exists() {
            if let Err(e) = fs::create_dir_all(&self.persona_directory) {
                return Err(LlamaError::InitializationFailed(
                    format!("Failed to create persona directory: {}", e)
                ));
            }
        }
        
        Ok(())
    }
    
    /// Generate a dialogue response
    fn generate_response(&self, context: &DialogueContext, player_input: &str) -> Result<DialogueEntry, LlamaError> {
        // Get the model manager
        let model_manager = match self.model_manager.lock() {
            Ok(manager) => manager,
            Err(_) => return Err(LlamaError::ResourceExhausted("Failed to lock model manager".to_string())),
        };
        
        // Get the model
        let model_name = self.config.model_name.as_deref().unwrap_or("default");
        let model = match model_manager.get_model(model_name) {
            Some(model) => model,
            None => {
                // Try the default model
                match model_manager.get_default_model() {
                    Some(model) => model,
                    None => return Err(LlamaError::InitializationFailed("No model available".to_string())),
                }
            }
        };
        
        // Build the prompt
        let prompt = self.build_prompt(context, player_input);
        
        // Create the request
        let request = LlamaRequest {
            id: format!("dialogue_{}", context.character_id),
            prompt,
            config: Some(LlamaConfig {
                temperature: self.config.temperature,
                max_tokens: self.config.max_tokens,
                timeout_seconds: self.config.timeout_seconds,
                ..LlamaConfig::default()
            }),
            system_prompt: Some(self.config.system_prompt.clone()),
            stop_sequences: self.config.stop_sequences.clone(),
        };
        
        // Generate the response
        let response = match model.lock() {
            Ok(context) => context.generate(&request)?,
            Err(_) => return Err(LlamaError::ResourceExhausted("Failed to lock model context".to_string())),
        };
        
        if !response.success {
            return Err(response.error.unwrap_or(LlamaError::InferenceFailed("Unknown error".to_string())));
        }
        
        // Parse the response
        let (text, emotion) = self.parse_response(&response.text);
        
        // Create dialogue entry
        let entry = DialogueEntry {
            speaker: context.character_name.clone(),
            text,
            emotion,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            metadata: HashMap::new(),
        };
        
        Ok(entry)
    }
    
    /// Generate dialogue options for the player
    fn generate_options(&self, context: &DialogueContext, current_topic: &str) -> Result<DialogueOptions, LlamaError> {
        // Get the model manager
        let model_manager = match self.model_manager.lock() {
            Ok(manager) => manager,
            Err(_) => return Err(LlamaError::ResourceExhausted("Failed to lock model manager".to_string())),
        };
        
        // Get the model
        let model_name = self.config.model_name.as_deref().unwrap_or("default");
        let model = match model_manager.get_model(model_name) {
            Some(model) => model,
            None => {
                // Try the default model
                match model_manager.get_default_model() {
                    Some(model) => model,
                    None => return Err(LlamaError::InitializationFailed("No model available".to_string())),
                }
            }
        };
        
        // Build the prompt
        let prompt = self.build_options_prompt(context, current_topic);
        
        // Create the request
        let request = LlamaRequest {
            id: format!("options_{}", context.character_id),
            prompt,
            config: Some(LlamaConfig {
                temperature: self.config.temperature,
                max_tokens: 150, // Options should be short
                timeout_seconds: self.config.timeout_seconds,
                ..LlamaConfig::default()
            }),
            system_prompt: Some("You are a helpful assistant generating dialogue options for a player in a fantasy roguelike game. Generate 3-4 concise, relevant dialogue options based on the context.".to_string()),
            stop_sequences: vec![],
        };
        
        // Generate the response
        let response = match model.lock() {
            Ok(context) => context.generate(&request)?,
            Err(_) => return Err(LlamaError::ResourceExhausted("Failed to lock model context".to_string())),
        };
        
        if !response.success {
            return Err(response.error.unwrap_or(LlamaError::InferenceFailed("Unknown error".to_string())));
        }
        
        // Parse the options
        let options = self.parse_options(&response.text);
        
        Ok(options)
    }
    
    /// Create a new dialogue context
    fn create_context(&self, persona: &CharacterPersona, location: &str) -> DialogueContext {
        DialogueContext {
            character_id: persona.id.clone(),
            character_name: persona.name.clone(),
            character_description: persona.description.clone(),
            location: location.to_string(),
            history: Vec::new(),
            relationship: 0, // Neutral by default
            knowledge: persona.knowledge_base.clone(),
            traits: persona.traits.clone(),
            faction: persona.faction.clone(),
            metadata: HashMap::new(),
        }
    }
    
    /// Update a dialogue context with a new entry
    fn update_context(&self, context: &mut DialogueContext, entry: DialogueEntry) {
        context.history.push(entry);
        
        // Trim history if it exceeds max length
        while context.history.len() > self.config.max_history_length {
            context.history.remove(0);
        }
    }
    
    /// Apply the effects of a dialogue option
    fn apply_option_effects(&self, context: &mut DialogueContext, option: &DialogueOption) -> Vec<String> {
        let mut applied_effects = Vec::new();
        
        for effect in &option.effects {
            match effect {
                DialogueEffect::RelationshipChange(value) => {
                    context.relationship = (context.relationship + value).clamp(-100, 100);
                    applied_effects.push(format!("Relationship changed by {}", value));
                },
                DialogueEffect::AddKnowledge(knowledge) => {
                    if !context.knowledge.contains(knowledge) {
                        context.knowledge.push(knowledge.clone());
                        applied_effects.push(format!("Added knowledge: {}", knowledge));
                    }
                },
                DialogueEffect::RemoveKnowledge(knowledge) => {
                    if let Some(pos) = context.knowledge.iter().position(|k| k == knowledge) {
                        context.knowledge.remove(pos);
                        applied_effects.push(format!("Removed knowledge: {}", knowledge));
                    }
                },
                DialogueEffect::SetFlag(key, value) => {
                    context.metadata.insert(key.clone(), value.clone());
                    applied_effects.push(format!("Set flag: {} = {}", key, value));
                },
                _ => {
                    // Other effects would be handled by the game engine
                    applied_effects.push(format!("Effect applied: {:?}", effect));
                }
            }
        }
        
        applied_effects
    }
    
    /// Check if a dialogue option is available based on requirements
    fn check_option_requirements(&self, context: &DialogueContext, option: &DialogueOption) -> bool {
        for requirement in &option.requirements {
            match requirement {
                DialogueRequirement::MinRelationship(value) => {
                    if context.relationship < *value {
                        return false;
                    }
                },
                DialogueRequirement::MaxRelationship(value) => {
                    if context.relationship > *value {
                        return false;
                    }
                },
                DialogueRequirement::HasKnowledge(knowledge) => {
                    if !context.knowledge.contains(knowledge) {
                        return false;
                    }
                },
                DialogueRequirement::LacksKnowledge(knowledge) => {
                    if context.knowledge.contains(knowledge) {
                        return false;
                    }
                },
                DialogueRequirement::HasFlag(key, value) => {
                    if context.metadata.get(key) != Some(value) {
                        return false;
                    }
                },
                _ => {
                    // Other requirements would be handled by the game engine
                }
            }
        }
        
        true
    }
    
    /// Get a fallback response when generation fails
    fn fallback_response(&self, context: &DialogueContext) -> DialogueEntry {
        // Simple fallback responses based on relationship
        let text = if context.relationship < -50 {
            "I have nothing to say to you."
        } else if context.relationship < 0 {
            "I'm busy right now."
        } else if context.relationship < 50 {
            "I'm sorry, I can't talk right now."
        } else {
            "Forgive me, but I need a moment to gather my thoughts."
        };
        
        DialogueEntry {
            speaker: context.character_name.clone(),
            text: text.to_string(),
            emotion: Some("confused".to_string()),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            metadata: HashMap::new(),
        }
    }
    
    /// Set the dialogue configuration
    fn set_config(&mut self, config: DialogueConfig) {
        self.config = config;
    }
    
    /// Get the current dialogue configuration
    fn get_config(&self) -> &DialogueConfig {
        &self.config
    }
    
    /// Load a character persona from a file
    fn load_persona(&self, id: &str) -> Result<CharacterPersona, LlamaError> {
        let persona_path = self.persona_directory.join(format!("{}.json", id));
        
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
        
        Ok(persona)
    }
    
    /// Save a character persona to a file
    fn save_persona(&self, persona: &CharacterPersona) -> Result<(), LlamaError> {
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
    fn create_persona(&self, name: &str, description: &str) -> CharacterPersona {
        // Generate a unique ID
        let id = format!("{}-{:08x}", name.to_lowercase().replace(" ", "-"), thread_rng().gen::<u32>());
        
        CharacterPersona {
            id,
            name: name.to_string(),
            description: description.to_string(),
            background: "".to_string(),
            traits: Vec::new(),
            speech_style: "".to_string(),
            knowledge_base: Vec::new(),
            faction: None,
            default_emotion: "neutral".to_string(),
            available_emotions: vec!["happy".to_string(), "sad".to_string(), "angry".to_string(), "neutral".to_string()],
            dialogue_templates: HashMap::new(),
            metadata: HashMap::new(),
        }
    }
}