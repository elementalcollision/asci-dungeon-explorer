use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs;
use std::io;
use serde::{Serialize, Deserialize};
use log::{info, warn, error};
use rand::{Rng, thread_rng};

use super::dialogue_system_trait::{CharacterPersona, DialogueEntry};
use super::llama_integration::LlamaError;

/// Character emotion with intensity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Emotion {
    pub name: String,
    pub intensity: u8, // 0-100
    pub description: String,
}

/// Character relationship with the player or other NPCs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relationship {
    pub target_id: String,
    pub value: i32, // -100 to 100
    pub history: Vec<RelationshipEvent>,
}

/// Event that affected a relationship
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipEvent {
    pub description: String,
    pub value_change: i32,
    pub timestamp: u64,
}

/// Character knowledge item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeItem {
    pub topic: String,
    pub content: String,
    pub certainty: u8, // 0-100
    pub source: String,
    pub timestamp: u64,
}

/// Extended character persona with more detailed information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtendedPersona {
    pub base: CharacterPersona,
    pub emotions: HashMap<String, Emotion>,
    pub relationships: HashMap<String, Relationship>,
    pub knowledge_base: Vec<KnowledgeItem>,
    pub dialogue_history: Vec<DialogueEntry>,
    pub personality_stats: HashMap<String, i32>,
    pub backstory_events: Vec<String>,
    pub goals: Vec<String>,
    pub fears: Vec<String>,
    pub secrets: Vec<String>,
    pub creation_date: u64,
    pub last_updated: u64,
    pub version: u32,
}

impl ExtendedPersona {
    /// Create a new extended persona from a base persona
    pub fn from_base(base: CharacterPersona) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        ExtendedPersona {
            base,
            emotions: HashMap::new(),
            relationships: HashMap::new(),
            knowledge_base: Vec::new(),
            dialogue_history: Vec::new(),
            personality_stats: HashMap::new(),
            backstory_events: Vec::new(),
            goals: Vec::new(),
            fears: Vec::new(),
            secrets: Vec::new(),
            creation_date: now,
            last_updated: now,
            version: 1,
        }
    }
    
    /// Add an emotion to the persona
    pub fn add_emotion(&mut self, name: &str, intensity: u8, description: &str) {
        self.emotions.insert(name.to_string(), Emotion {
            name: name.to_string(),
            intensity,
            description: description.to_string(),
        });
        self.update_timestamp();
    }
    
    /// Add a relationship to the persona
    pub fn add_relationship(&mut self, target_id: &str, value: i32, description: &str) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        let event = RelationshipEvent {
            description: description.to_string(),
            value_change: value,
            timestamp: now,
        };
        
        if let Some(relationship) = self.relationships.get_mut(target_id) {
            relationship.value = value;
            relationship.history.push(event);
        } else {
            self.relationships.insert(target_id.to_string(), Relationship {
                target_id: target_id.to_string(),
                value,
                history: vec![event],
            });
        }
        
        self.update_timestamp();
    }
    
    /// Update a relationship value
    pub fn update_relationship(&mut self, target_id: &str, value_change: i32, description: &str) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        let event = RelationshipEvent {
            description: description.to_string(),
            value_change,
            timestamp: now,
        };
        
        if let Some(relationship) = self.relationships.get_mut(target_id) {
            relationship.value = (relationship.value + value_change).clamp(-100, 100);
            relationship.history.push(event);
        } else {
            self.relationships.insert(target_id.to_string(), Relationship {
                target_id: target_id.to_string(),
                value: value_change.clamp(-100, 100),
                history: vec![event],
            });
        }
        
        self.update_timestamp();
    }
    
    /// Add a knowledge item to the persona
    pub fn add_knowledge(&mut self, topic: &str, content: &str, certainty: u8, source: &str) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        self.knowledge_base.push(KnowledgeItem {
            topic: topic.to_string(),
            content: content.to_string(),
            certainty,
            source: source.to_string(),
            timestamp: now,
        });
        
        self.update_timestamp();
    }
    
    /// Add a dialogue entry to the persona's history
    pub fn add_dialogue_entry(&mut self, entry: DialogueEntry) {
        self.dialogue_history.push(entry);
        self.update_timestamp();
    }
    
    /// Add a backstory event to the persona
    pub fn add_backstory_event(&mut self, event: &str) {
        self.backstory_events.push(event.to_string());
        self.update_timestamp();
    }
    
    /// Add a goal to the persona
    pub fn add_goal(&mut self, goal: &str) {
        self.goals.push(goal.to_string());
        self.update_timestamp();
    }
    
    /// Add a fear to the persona
    pub fn add_fear(&mut self, fear: &str) {
        self.fears.push(fear.to_string());
        self.update_timestamp();
    }
    
    /// Add a secret to the persona
    pub fn add_secret(&mut self, secret: &str) {
        self.secrets.push(secret.to_string());
        self.update_timestamp();
    }
    
    /// Set a personality stat
    pub fn set_personality_stat(&mut self, name: &str, value: i32) {
        self.personality_stats.insert(name.to_string(), value);
        self.update_timestamp();
    }
    
    /// Get the current dominant emotion
    pub fn get_dominant_emotion(&self) -> Option<&Emotion> {
        self.emotions.values()
            .max_by_key(|e| e.intensity)
    }
    
    /// Get the relationship with a target
    pub fn get_relationship(&self, target_id: &str) -> Option<&Relationship> {
        self.relationships.get(target_id)
    }
    
    /// Get knowledge items about a topic
    pub fn get_knowledge_about(&self, topic: &str) -> Vec<&KnowledgeItem> {
        self.knowledge_base.iter()
            .filter(|item| item.topic.contains(topic))
            .collect()
    }
    
    /// Update the last updated timestamp
    fn update_timestamp(&mut self) {
        self.last_updated = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
    }
    
    /// Save the extended persona to a file
    pub fn save_to_file(&self, path: &Path) -> io::Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json)?;
        Ok(())
    }
    
    /// Load an extended persona from a file
    pub fn load_from_file(path: &Path) -> io::Result<Self> {
        let json = fs::read_to_string(path)?;
        let persona: ExtendedPersona = serde_json::from_str(&json)?;
        Ok(persona)
    }
    
    /// Convert to a base persona
    pub fn to_base(&self) -> CharacterPersona {
        self.base.clone()
    }
}

/// Manager for character personas
pub struct PersonaManager {
    personas: HashMap<String, ExtendedPersona>,
    persona_directory: PathBuf,
}

impl PersonaManager {
    /// Create a new persona manager
    pub fn new(persona_directory: PathBuf) -> Self {
        PersonaManager {
            personas: HashMap::new(),
            persona_directory,
        }
    }
    
    /// Initialize the persona manager
    pub fn initialize(&mut self) -> Result<(), LlamaError> {
        // Create directory if it doesn't exist
        if !self.persona_directory.exists() {
            if let Err(e) = fs::create_dir_all(&self.persona_directory) {
                return Err(LlamaError::InitializationFailed(
                    format!("Failed to create persona directory: {}", e)
                ));
            }
        }
        
        // Load all personas
        self.load_all_personas()?;
        
        Ok(())
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
    
    /// Load a persona from a file
    pub fn load_persona(&mut self, character_id: &str) -> Result<ExtendedPersona, LlamaError> {
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
        
        let persona = match ExtendedPersona::load_from_file(&persona_path) {
            Ok(persona) => persona,
            Err(e) => return Err(LlamaError::ModelLoadFailed(
                format!("Failed to load persona: {}", e)
            )),
        };
        
        // Store the persona
        self.personas.insert(character_id.to_string(), persona.clone());
        
        Ok(persona)
    }
    
    /// Save a persona to a file
    pub fn save_persona(&mut self, persona: &ExtendedPersona) -> Result<(), LlamaError> {
        let persona_path = self.persona_directory.join(format!("{}.json", persona.base.id));
        
        match persona.save_to_file(&persona_path) {
            Ok(_) => {
                // Update the stored persona
                self.personas.insert(persona.base.id.clone(), persona.clone());
                Ok(())
            },
            Err(e) => Err(LlamaError::ResourceExhausted(
                format!("Failed to save persona: {}", e)
            )),
        }
    }
    
    /// Create a new persona
    pub fn create_persona(&mut self, name: &str, description: &str) -> Result<ExtendedPersona, LlamaError> {
        // Generate a unique ID
        let id = format!("{}-{:08x}", name.to_lowercase().replace(" ", "-"), thread_rng().gen::<u32>());
        
        // Create the base persona
        let base = CharacterPersona {
            id: id.clone(),
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
        };
        
        // Create the extended persona
        let persona = ExtendedPersona::from_base(base);
        
        // Save the persona
        self.save_persona(&persona)?;
        
        Ok(persona)
    }
    
    /// Get a persona by ID
    pub fn get_persona(&self, character_id: &str) -> Option<&ExtendedPersona> {
        self.personas.get(character_id)
    }
    
    /// Get a mutable reference to a persona by ID
    pub fn get_persona_mut(&mut self, character_id: &str) -> Option<&mut ExtendedPersona> {
        self.personas.get_mut(character_id)
    }
    
    /// Get all personas
    pub fn get_all_personas(&self) -> Vec<&ExtendedPersona> {
        self.personas.values().collect()
    }
    
    /// Delete a persona
    pub fn delete_persona(&mut self, character_id: &str) -> Result<(), LlamaError> {
        // Remove from memory
        self.personas.remove(character_id);
        
        // Remove from disk
        let persona_path = self.persona_directory.join(format!("{}.json", character_id));
        
        if persona_path.exists() {
            match fs::remove_file(&persona_path) {
                Ok(_) => Ok(()),
                Err(e) => Err(LlamaError::ResourceExhausted(
                    format!("Failed to delete persona file: {}", e)
                )),
            }
        } else {
            Ok(())
        }
    }
    
    /// Generate a random persona for testing
    pub fn generate_random_persona(&mut self) -> Result<ExtendedPersona, LlamaError> {
        let mut rng = thread_rng();
        
        // Random name generation
        let first_names = ["Elric", "Lyra", "Thorne", "Aria", "Krag", "Seraphina", "Dorn", "Zephyr"];
        let last_names = ["Stormborn", "Nightshade", "Ironfist", "Swiftblade", "Flameheart", "Moonshadow"];
        
        let first_name = first_names[rng.gen_range(0..first_names.len())];
        let last_name = last_names[rng.gen_range(0..last_names.len())];
        let name = format!("{} {}", first_name, last_name);
        
        // Random description
        let descriptions = [
            "A grizzled warrior with scars across their face",
            "A mysterious mage with glowing eyes",
            "A nimble rogue with a mischievous smile",
            "A wise healer with a gentle demeanor",
            "A stoic ranger with keen eyes",
        ];
        
        let description = descriptions[rng.gen_range(0..descriptions.len())];
        
        // Create the persona
        let mut persona = self.create_persona(&name, description)?;
        
        // Add random traits
        let traits = ["Brave", "Cautious", "Curious", "Loyal", "Greedy", "Honorable", "Suspicious"];
        for _ in 0..rng.gen_range(2..5) {
            let trait_name = traits[rng.gen_range(0..traits.len())];
            persona.base.traits.push(trait_name.to_string());
        }
        
        // Add random knowledge
        let knowledge = [
            "The ancient ruins hold a powerful artifact",
            "Dragons have been spotted in the northern mountains",
            "The king's advisor is secretly a necromancer",
            "The forest is home to dangerous creatures",
            "There's a hidden passage beneath the old tavern",
        ];
        
        for _ in 0..rng.gen_range(2..4) {
            let k = knowledge[rng.gen_range(0..knowledge.len())];
            persona.add_knowledge("world", k, rng.gen_range(50..100), "experience");
        }
        
        // Add random goals
        let goals = [
            "Find the legendary sword of truth",
            "Avenge the death of their family",
            "Discover the secret of eternal life",
            "Become the greatest warrior in the land",
            "Find a cure for the mysterious plague",
        ];
        
        for _ in 0..rng.gen_range(1..3) {
            let goal = goals[rng.gen_range(0..goals.len())];
            persona.add_goal(goal);
        }
        
        // Save the persona
        self.save_persona(&persona)?;
        
        Ok(persona)
    }
}