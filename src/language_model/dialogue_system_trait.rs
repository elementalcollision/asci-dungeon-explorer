use std::collections::HashMap;
use serde::{Serialize, Deserialize};

use super::llama_integration::LlamaError;

/// Dialogue entry representing a conversation turn
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogueEntry {
    pub speaker: String,
    pub text: String,
    pub emotion: Option<String>,
    pub timestamp: u64,
    pub metadata: HashMap<String, String>,
}

/// Dialogue context for a conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogueContext {
    pub character_id: String,
    pub character_name: String,
    pub character_description: String,
    pub location: String,
    pub history: Vec<DialogueEntry>,
    pub relationship: i32, // -100 to 100, negative is hostile, positive is friendly
    pub knowledge: Vec<String>, // Things the character knows about
    pub traits: Vec<String>, // Character personality traits
    pub faction: Option<String>,
    pub metadata: HashMap<String, String>,
}

/// Character persona for dialogue generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterPersona {
    pub id: String,
    pub name: String,
    pub description: String,
    pub background: String,
    pub traits: Vec<String>,
    pub speech_style: String,
    pub knowledge_base: Vec<String>,
    pub faction: Option<String>,
    pub default_emotion: String,
    pub available_emotions: Vec<String>,
    pub dialogue_templates: HashMap<String, Vec<String>>,
    pub metadata: HashMap<String, String>,
}

/// Dialogue response options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogueOptions {
    pub options: Vec<DialogueOption>,
    pub timeout_seconds: Option<u32>,
    pub default_option: Option<usize>,
}

/// Single dialogue option
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogueOption {
    pub text: String,
    pub next_state: Option<String>,
    pub effects: Vec<DialogueEffect>,
    pub requirements: Vec<DialogueRequirement>,
    pub metadata: HashMap<String, String>,
}

/// Effect of selecting a dialogue option
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DialogueEffect {
    RelationshipChange(i32),
    AddKnowledge(String),
    RemoveKnowledge(String),
    GiveItem(String),
    TakeItem(String),
    TriggerEvent(String),
    SetFlag(String, String),
    Custom(String, String),
}

/// Requirement for a dialogue option to be available
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DialogueRequirement {
    MinRelationship(i32),
    MaxRelationship(i32),
    HasKnowledge(String),
    LacksKnowledge(String),
    HasItem(String),
    HasFlag(String, String),
    Custom(String, String),
}

/// Configuration for dialogue generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogueConfig {
    pub max_history_length: usize,
    pub system_prompt: String,
    pub temperature: f32,
    pub max_tokens: u32,
    pub stop_sequences: Vec<String>,
    pub model_name: Option<String>,
    pub timeout_seconds: u64,
}

impl Default for DialogueConfig {
    fn default() -> Self {
        DialogueConfig {
            max_history_length: 10,
            system_prompt: "You are an NPC in a fantasy roguelike game. Respond in character based on your personality, knowledge, and relationship with the player. Keep responses concise (1-3 sentences) and appropriate to the fantasy setting.".to_string(),
            temperature: 0.7,
            max_tokens: 100,
            stop_sequences: vec!["\n".to_string(), "Player:".to_string()],
            model_name: None,
            timeout_seconds: 10,
        }
    }
}

/// Dialogue system trait for generating NPC dialogue
pub trait DialogueSystem {
    /// Initialize the dialogue system
    fn initialize(&mut self) -> Result<(), LlamaError>;
    
    /// Generate a dialogue response
    fn generate_response(&self, context: &DialogueContext, player_input: &str) -> Result<DialogueEntry, LlamaError>;
    
    /// Generate dialogue options for the player
    fn generate_options(&self, context: &DialogueContext, current_topic: &str) -> Result<DialogueOptions, LlamaError>;
    
    /// Create a new dialogue context
    fn create_context(&self, persona: &CharacterPersona, location: &str) -> DialogueContext;
    
    /// Update a dialogue context with a new entry
    fn update_context(&self, context: &mut DialogueContext, entry: DialogueEntry);
    
    /// Apply the effects of a dialogue option
    fn apply_option_effects(&self, context: &mut DialogueContext, option: &DialogueOption) -> Vec<String>;
    
    /// Check if a dialogue option is available based on requirements
    fn check_option_requirements(&self, context: &DialogueContext, option: &DialogueOption) -> bool;
    
    /// Get a fallback response when generation fails
    fn fallback_response(&self, context: &DialogueContext) -> DialogueEntry;
    
    /// Set the dialogue configuration
    fn set_config(&mut self, config: DialogueConfig);
    
    /// Get the current dialogue configuration
    fn get_config(&self) -> &DialogueConfig;
    
    /// Load a character persona from a file
    fn load_persona(&self, id: &str) -> Result<CharacterPersona, LlamaError>;
    
    /// Save a character persona to a file
    fn save_persona(&self, persona: &CharacterPersona) -> Result<(), LlamaError>;
    
    /// Create a new character persona
    fn create_persona(&self, name: &str, description: &str) -> CharacterPersona;
}