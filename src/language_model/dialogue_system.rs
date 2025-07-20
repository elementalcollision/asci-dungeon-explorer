use std::sync::{Arc, Mutex};
use log::{info, warn, error};

use super::llama_integration::{LlamaContext, LlamaRequest, LlamaResponse, LlamaError};
use super::model_manager::ModelManager;

/// Dialogue system for generating NPC dialogue
pub struct DialogueSystem {
    model_manager: Arc<Mutex<ModelManager>>,
    system_prompt: String,
    max_history_length: usize,
}

/// Dialogue entry representing a conversation turn
#[derive(Debug, Clone)]
pub struct DialogueEntry {
    pub speaker: String,
    pub text: String,
    pub emotion: Option<String>,
    pub timestamp: u64,
}

/// Dialogue context for a conversation
#[derive(Debug, Clone)]
pub struct DialogueContext {
    pub character_name: String,
    pub character_description: String,
    pub location: String,
    pub history: Vec<DialogueEntry>,
    pub relationship: i32, // -100 to 100, negative is hostile, positive is friendly
    pub knowledge: Vec<String>, // Things the character knows about
}

impl DialogueSystem {
    /// Create a new dialogue system
    pub fn new(model_manager: Arc<Mutex<ModelManager>>) -> Self {
        DialogueSystem {
            model_manager,
            system_prompt: "You are an NPC in a fantasy roguelike game. Respond in character based on your personality, knowledge, and relationship with the player. Keep responses concise (1-3 sentences) and appropriate to the fantasy setting.".to_string(),
            max_history_length: 10,
        }
    }
    
    /// Set the system prompt
    pub fn set_system_prompt(&mut self, prompt: String) {
        self.system_prompt = prompt;
    }
    
    /// Set the maximum history length
    pub fn set_max_history_length(&mut self, length: usize) {
        self.max_history_length = length;
    }
    
    /// Generate a dialogue response
    pub fn generate_response(&self, context: &DialogueContext, player_input: &str) -> Result<DialogueEntry, LlamaError> {
        // Get the model manager
        let model_manager = match self.model_manager.lock() {
            Ok(manager) => manager,
            Err(_) => return Err(LlamaError::ResourceExhausted("Failed to lock model manager".to_string())),
        };
        
        // Get the default model
        let model = match model_manager.get_default_model() {
            Some(model) => model,
            None => return Err(LlamaError::InitializationFailed("No default model set".to_string())),
        };
        
        // Build the prompt
        let prompt = self.build_prompt(context, player_input);
        
        // Create the request
        let request = LlamaRequest {
            id: format!("dialogue_{}", context.character_name),
            prompt,
            config: None,
            system_prompt: Some(self.system_prompt.clone()),
            stop_sequences: vec!["\n".to_string(), "Player:".to_string()],
        };
        
        // Generate the response
        let response = match model.lock() {
            Ok(context) => context.generate(&request)?,
            Err(_) => return Err(LlamaError::ResourceExhausted("Failed to lock model context".to_string())),
        };
        
        // Parse the response
        self.parse_response(context, response)
    }
    
    /// Build a prompt from the dialogue context
    fn build_prompt(&self, context: &DialogueContext, player_input: &str) -> String {
        let mut prompt = String::new();
        
        // Add character information
        prompt.push_str(&format!("Character: {}\n", context.character_name));
        prompt.push_str(&format!("Description: {}\n", context.character_description));
        prompt.push_str(&format!("Location: {}\n", context.location));
        prompt.push_str(&format!("Relationship with player: {}\n", self.relationship_description(context.relationship)));
        
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
        let start_idx = if context.history.len() > self.max_history_length {
            context.history.len() - self.max_history_length
        } else {
            0
        };
        
        for entry in &context.history[start_idx..] {
            prompt.push_str(&format!("{}: {}\n", entry.speaker, entry.text));
        }
        
        // Add player input
        prompt.push_str(&format!("Player: {}\n", player_input));
        prompt.push_str(&format!("{}:", context.character_name));
        
        prompt
    }
    
    /// Parse the response from the model
    fn parse_response(&self, context: &DialogueContext, response: LlamaResponse) -> Result<DialogueEntry, LlamaError> {
        if !response.success {
            return Err(response.error.unwrap_or(LlamaError::InferenceFailed("Unknown error".to_string())));
        }
        
        // Clean up the response
        let text = response.text.trim();
        
        // Extract emotion if present (format: [emotion] text)
        let (emotion, clean_text) = if text.starts_with('[') {
            if let Some(end_bracket) = text.find(']') {
                let emotion = text[1..end_bracket].trim().to_string();
                let remaining = text[end_bracket + 1..].trim().to_string();
                (Some(emotion), remaining)
            } else {
                (None, text.to_string())
            }
        } else {
            (None, text.to_string())
        };
        
        // Create dialogue entry
        Ok(DialogueEntry {
            speaker: context.character_name.clone(),
            text: clean_text,
            emotion,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        })
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
    
    /// Create a fallback response when model generation fails
    pub fn fallback_response(&self, context: &DialogueContext) -> DialogueEntry {
        DialogueEntry {
            speaker: context.character_name.clone(),
            text: "I'm sorry, I cannot speak right now.".to_string(),
            emotion: Some("confused".to_string()),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use super::super::llama_integration::LlamaConfig;
    
    #[test]
    fn test_dialogue_system_creation() {
        let model_manager = Arc::new(Mutex::new(ModelManager::new()));
        let dialogue_system = DialogueSystem::new(model_manager);
        
        assert_eq!(dialogue_system.max_history_length, 10);
        assert!(!dialogue_system.system_prompt.is_empty());
    }
    
    #[test]
    fn test_build_prompt() {
        let model_manager = Arc::new(Mutex::new(ModelManager::new()));
        let dialogue_system = DialogueSystem::new(model_manager);
        
        let context = DialogueContext {
            character_name: "Eldrin".to_string(),
            character_description: "An old wizard with a long beard".to_string(),
            location: "Tower of Magic".to_string(),
            history: vec![
                DialogueEntry {
                    speaker: "Player".to_string(),
                    text: "Hello there".to_string(),
                    emotion: None,
                    timestamp: 0,
                },
                DialogueEntry {
                    speaker: "Eldrin".to_string(),
                    text: "Greetings, traveler".to_string(),
                    emotion: Some("friendly".to_string()),
                    timestamp: 1,
                },
            ],
            relationship: 25,
            knowledge: vec!["The player is new to the tower".to_string()],
        };
        
        let prompt = dialogue_system.build_prompt(&context, "Can you teach me magic?");
        
        assert!(prompt.contains("Character: Eldrin"));
        assert!(prompt.contains("Description: An old wizard with a long beard"));
        assert!(prompt.contains("Location: Tower of Magic"));
        assert!(prompt.contains("Relationship with player: Friendly"));
        assert!(prompt.contains("Knowledge:"));
        assert!(prompt.contains("- The player is new to the tower"));
        assert!(prompt.contains("Player: Hello there"));
        assert!(prompt.contains("Eldrin: Greetings, traveler"));
        assert!(prompt.contains("Player: Can you teach me magic?"));
        assert!(prompt.contains("Eldrin:"));
    }
    
    #[test]
    fn test_relationship_description() {
        let model_manager = Arc::new(Mutex::new(ModelManager::new()));
        let dialogue_system = DialogueSystem::new(model_manager);
        
        assert_eq!(dialogue_system.relationship_description(-100), "Hostile");
        assert_eq!(dialogue_system.relationship_description(-60), "Antagonistic");
        assert_eq!(dialogue_system.relationship_description(-30), "Unfriendly");
        assert_eq!(dialogue_system.relationship_description(-10), "Wary");
        assert_eq!(dialogue_system.relationship_description(0), "Neutral");
        assert_eq!(dialogue_system.relationship_description(30), "Friendly");
        assert_eq!(dialogue_system.relationship_description(60), "Trusting");
        assert_eq!(dialogue_system.relationship_description(90), "Loyal");
    }
    
    #[test]
    fn test_fallback_response() {
        let model_manager = Arc::new(Mutex::new(ModelManager::new()));
        let dialogue_system = DialogueSystem::new(model_manager);
        
        let context = DialogueContext {
            character_name: "Eldrin".to_string(),
            character_description: "An old wizard".to_string(),
            location: "Tower".to_string(),
            history: vec![],
            relationship: 0,
            knowledge: vec![],
        };
        
        let response = dialogue_system.fallback_response(&context);
        
        assert_eq!(response.speaker, "Eldrin");
        assert!(!response.text.is_empty());
        assert_eq!(response.emotion, Some("confused".to_string()));
    }
}