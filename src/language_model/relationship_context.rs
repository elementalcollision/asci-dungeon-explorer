use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::fs;
use std::io;
use serde::{Serialize, Deserialize};
use log::{info, warn, error};

/// Relationship type between entities
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum RelationshipType {
    Friend,
    Enemy,
    Ally,
    Rival,
    Family,
    Mentor,
    Student,
    Acquaintance,
    Stranger,
    Lover,
    ExLover,
    Business,
    Guild,
    Custom(String),
}

impl RelationshipType {
    pub fn as_str(&self) -> &str {
        match self {
            RelationshipType::Friend => "friend",
            RelationshipType::Enemy => "enemy",
            RelationshipType::Ally => "ally",
            RelationshipType::Rival => "rival",
            RelationshipType::Family => "family",
            RelationshipType::Mentor => "mentor",
            RelationshipType::Student => "student",
            RelationshipType::Acquaintance => "acquaintance",
            RelationshipType::Stranger => "stranger",
            RelationshipType::Lover => "lover",
            RelationshipType::ExLover => "ex-lover",
            RelationshipType::Business => "business",
            RelationshipType::Guild => "guild",
            RelationshipType::Custom(s) => s,
        }
    }
    
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "friend" => RelationshipType::Friend,
            "enemy" => RelationshipType::Enemy,
            "ally" => RelationshipType::Ally,
            "rival" => RelationshipType::Rival,
            "family" => RelationshipType::Family,
            "mentor" => RelationshipType::Mentor,
            "student" => RelationshipType::Student,
            "acquaintance" => RelationshipType::Acquaintance,
            "stranger" => RelationshipType::Stranger,
            "lover" => RelationshipType::Lover,
            "ex-lover" => RelationshipType::ExLover,
            "business" => RelationshipType::Business,
            "guild" => RelationshipType::Guild,
            _ => RelationshipType::Custom(s.to_string()),
        }
    }
}

/// Relationship event that affected a relationship
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipEvent {
    pub id: String,
    pub description: String,
    pub value_change: i32,
    pub timestamp: u64,
    pub location: Option<String>,
    pub witnesses: Vec<String>,
    pub tags: Vec<String>,
    pub metadata: HashMap<String, String>,
}

/// Relationship between two entities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relationship {
    pub source_id: String,
    pub target_id: String,
    pub relationship_type: RelationshipType,
    pub value: i32, // -100 to 100
    pub history: Vec<RelationshipEvent>,
    pub created_at: u64,
    pub updated_at: u64,
    pub metadata: HashMap<String, String>,
}

/// Relationship network for tracking relationships between entities
pub struct RelationshipNetwork {
    relationships: HashMap<String, HashMap<String, Relationship>>,
    relationship_directory: PathBuf,
}

impl RelationshipNetwork {
    /// Create a new relationship network
    pub fn new(relationship_directory: PathBuf) -> Self {
        RelationshipNetwork {
            relationships: HashMap::new(),
            relationship_directory,
        }
    }
    
    /// Initialize the relationship network
    pub fn initialize(&mut self) -> io::Result<()> {
        // Create directory if it doesn't exist
        if !self.relationship_directory.exists() {
            fs::create_dir_all(&self.relationship_directory)?;
        }
        
        // Load relationships
        self.load_relationships()?;
        
        Ok(())
    }
    
    /// Load relationships from file
    fn load_relationships(&mut self) -> io::Result<()> {
        let relationship_path = self.relationship_directory.join("relationships.json");
        
        if relationship_path.exists() {
            let json = fs::read_to_string(&relationship_path)?;
            let relationships: HashMap<String, HashMap<String, Relationship>> = serde_json::from_str(&json)?;
            
            self.relationships = relationships;
            
            let total_relationships: usize = self.relationships.values().map(|m| m.len()).sum();
            info!("Loaded {} relationships for {} entities", total_relationships, self.relationships.len());
        }
        
        Ok(())
    }
    
    /// Save relationships to file
    pub fn save_relationships(&self) -> io::Result<()> {
        let relationship_path = self.relationship_directory.join("relationships.json");
        
        let json = serde_json::to_string_pretty(&self.relationships)?;
        fs::write(&relationship_path, json)?;
        
        Ok(())
    }
    
    /// Add or update a relationship
    pub fn set_relationship(&mut self, source_id: &str, target_id: &str, relationship_type: RelationshipType, value: i32, description: &str) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        // Get or create the relationship
        let relationship = self.get_or_create_relationship(source_id, target_id);
        
        // Calculate value change
        let value_change = value - relationship.value;
        
        // Update relationship
        relationship.relationship_type = relationship_type;
        relationship.value = value.clamp(-100, 100);
        relationship.updated_at = now;
        
        // Add event to history
        if value_change != 0 {
            let event = RelationshipEvent {
                id: format!("event-{}-{}", source_id, now),
                description: description.to_string(),
                value_change,
                timestamp: now,
                location: None,
                witnesses: Vec::new(),
                tags: Vec::new(),
                metadata: HashMap::new(),
            };
            
            relationship.history.push(event);
        }
    }
    
    /// Update a relationship value
    pub fn update_relationship_value(&mut self, source_id: &str, target_id: &str, value_change: i32, description: &str) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        // Get or create the relationship
        let relationship = self.get_or_create_relationship(source_id, target_id);
        
        // Update value
        let old_value = relationship.value;
        relationship.value = (relationship.value + value_change).clamp(-100, 100);
        relationship.updated_at = now;
        
        // Add event to history
        if relationship.value != old_value {
            let event = RelationshipEvent {
                id: format!("event-{}-{}", source_id, now),
                description: description.to_string(),
                value_change,
                timestamp: now,
                location: None,
                witnesses: Vec::new(),
                tags: Vec::new(),
                metadata: HashMap::new(),
            };
            
            relationship.history.push(event);
        }
    }
    
    /// Get or create a relationship
    fn get_or_create_relationship(&mut self, source_id: &str, target_id: &str) -> &mut Relationship {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        // Ensure source map exists
        if !self.relationships.contains_key(source_id) {
            self.relationships.insert(source_id.to_string(), HashMap::new());
        }
        
        // Get or create relationship
        let source_map = self.relationships.get_mut(source_id).unwrap();
        
        if !source_map.contains_key(target_id) {
            source_map.insert(target_id.to_string(), Relationship {
                source_id: source_id.to_string(),
                target_id: target_id.to_string(),
                relationship_type: RelationshipType::Stranger,
                value: 0, // Neutral by default
                history: Vec::new(),
                created_at: now,
                updated_at: now,
                metadata: HashMap::new(),
            });
        }
        
        source_map.get_mut(target_id).unwrap()
    }
    
    /// Get a relationship
    pub fn get_relationship(&self, source_id: &str, target_id: &str) -> Option<&Relationship> {
        self.relationships
            .get(source_id)
            .and_then(|map| map.get(target_id))
    }
    
    /// Get all relationships for an entity
    pub fn get_all_relationships(&self, entity_id: &str) -> Vec<&Relationship> {
        let mut result = Vec::new();
        
        // Get outgoing relationships
        if let Some(map) = self.relationships.get(entity_id) {
            result.extend(map.values());
        }
        
        // Get incoming relationships
        for (source_id, map) in &self.relationships {
            if source_id != entity_id {
                if let Some(relationship) = map.get(entity_id) {
                    result.push(relationship);
                }
            }
        }
        
        result
    }
    
    /// Get all entities with a specific relationship type to an entity
    pub fn get_entities_with_relationship_type(&self, entity_id: &str, relationship_type: &RelationshipType) -> Vec<String> {
        let mut result = Vec::new();
        
        // Check outgoing relationships
        if let Some(map) = self.relationships.get(entity_id) {
            for (target_id, relationship) in map {
                if relationship.relationship_type == *relationship_type {
                    result.push(target_id.clone());
                }
            }
        }
        
        // Check incoming relationships
        for (source_id, map) in &self.relationships {
            if source_id != entity_id {
                if let Some(relationship) = map.get(entity_id) {
                    if relationship.relationship_type == *relationship_type {
                        result.push(source_id.clone());
                    }
                }
            }
        }
        
        result
    }
    
    /// Get all entities with a relationship value above a threshold
    pub fn get_entities_with_min_value(&self, entity_id: &str, min_value: i32) -> Vec<String> {
        let mut result = Vec::new();
        
        // Check outgoing relationships
        if let Some(map) = self.relationships.get(entity_id) {
            for (target_id, relationship) in map {
                if relationship.value >= min_value {
                    result.push(target_id.clone());
                }
            }
        }
        
        // Check incoming relationships
        for (source_id, map) in &self.relationships {
            if source_id != entity_id {
                if let Some(relationship) = map.get(entity_id) {
                    if relationship.value >= min_value {
                        result.push(source_id.clone());
                    }
                }
            }
        }
        
        result
    }
    
    /// Get all entities with a relationship value below a threshold
    pub fn get_entities_with_max_value(&self, entity_id: &str, max_value: i32) -> Vec<String> {
        let mut result = Vec::new();
        
        // Check outgoing relationships
        if let Some(map) = self.relationships.get(entity_id) {
            for (target_id, relationship) in map {
                if relationship.value <= max_value {
                    result.push(target_id.clone());
                }
            }
        }
        
        // Check incoming relationships
        for (source_id, map) in &self.relationships {
            if source_id != entity_id {
                if let Some(relationship) = map.get(entity_id) {
                    if relationship.value <= max_value {
                        result.push(source_id.clone());
                    }
                }
            }
        }
        
        result
    }
    
    /// Get relationship events for an entity
    pub fn get_relationship_events(&self, entity_id: &str) -> Vec<&RelationshipEvent> {
        let mut result = Vec::new();
        
        // Get events from outgoing relationships
        if let Some(map) = self.relationships.get(entity_id) {
            for relationship in map.values() {
                result.extend(&relationship.history);
            }
        }
        
        // Get events from incoming relationships
        for (source_id, map) in &self.relationships {
            if source_id != entity_id {
                if let Some(relationship) = map.get(entity_id) {
                    result.extend(&relationship.history);
                }
            }
        }
        
        // Sort by timestamp (newest first)
        result.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        
        result
    }
    
    /// Get recent relationship events for an entity
    pub fn get_recent_events(&self, entity_id: &str, count: usize) -> Vec<&RelationshipEvent> {
        let events = self.get_relationship_events(entity_id);
        events.into_iter().take(count).collect()
    }
    
    /// Add a relationship event
    pub fn add_relationship_event(&mut self, source_id: &str, target_id: &str, event: RelationshipEvent) {
        // Get or create the relationship
        let relationship = self.get_or_create_relationship(source_id, target_id);
        
        // Update relationship value
        relationship.value = (relationship.value + event.value_change).clamp(-100, 100);
        relationship.updated_at = event.timestamp;
        
        // Add event to history
        relationship.history.push(event);
    }
    
    /// Generate relationship description
    pub fn generate_relationship_description(&self, source_id: &str, target_id: &str) -> String {
        if let Some(relationship) = self.get_relationship(source_id, target_id) {
            let value_desc = self.value_description(relationship.value);
            let type_desc = relationship.relationship_type.as_str();
            
            format!("{} ({} - {})", type_desc, value_desc, relationship.value)
        } else {
            "Stranger (Neutral - 0)".to_string()
        }
    }
    
    /// Generate relationship context
    pub fn generate_relationship_context(&self, entity_id: &str) -> HashMap<String, String> {
        let mut context = HashMap::new();
        
        // Get all relationships
        let relationships = self.get_all_relationships(entity_id);
        
        // Group by relationship type
        let mut grouped: HashMap<RelationshipType, Vec<&Relationship>> = HashMap::new();
        
        for relationship in relationships {
            let rel_type = if relationship.source_id == entity_id {
                relationship.relationship_type.clone()
            } else {
                // Invert relationship type for incoming relationships
                match relationship.relationship_type {
                    RelationshipType::Mentor => RelationshipType::Student,
                    RelationshipType::Student => RelationshipType::Mentor,
                    _ => relationship.relationship_type.clone(),
                }
            };
            
            grouped.entry(rel_type).or_insert_with(Vec::new).push(relationship);
        }
        
        // Generate context for each relationship type
        for (rel_type, rels) in grouped {
            let mut descriptions = Vec::new();
            
            for rel in rels {
                let other_id = if rel.source_id == entity_id {
                    &rel.target_id
                } else {
                    &rel.source_id
                };
                
                let value_desc = self.value_description(rel.value);
                descriptions.push(format!("{} ({})", other_id, value_desc));
            }
            
            if !descriptions.is_empty() {
                context.insert(rel_type.as_str().to_string(), descriptions.join(", "));
            }
        }
        
        // Add recent events
        let recent_events = self.get_recent_events(entity_id, 5);
        if !recent_events.is_empty() {
            let event_descriptions: Vec<String> = recent_events.iter()
                .map(|e| format!("{}", e.description))
                .collect();
            
            context.insert("recent_events".to_string(), event_descriptions.join(", "));
        }
        
        context
    }
    
    /// Convert relationship value to description
    fn value_description(&self, value: i32) -> &'static str {
        match value {
            v if v < -75 => "Hateful",
            v if v < -50 => "Hostile",
            v if v < -25 => "Unfriendly",
            v if v < 0 => "Wary",
            v if v == 0 => "Neutral",
            v if v < 25 => "Cordial",
            v if v < 50 => "Friendly",
            v if v < 75 => "Trusting",
            v if v < 90 => "Close",
            _ => "Devoted",
        }
    }
    
    /// Create default relationships for testing
    pub fn create_default_relationships(&mut self) {
        // Create some example relationships
        self.set_relationship("player", "wizard", RelationshipType::Mentor, 60, "The wizard has been teaching the player magic");
        self.set_relationship("player", "merchant", RelationshipType::Business, 30, "The player is a regular customer");
        self.set_relationship("player", "guard_captain", RelationshipType::Acquaintance, -10, "The guard captain is suspicious of the player");
        self.set_relationship("player", "innkeeper", RelationshipType::Friend, 40, "The player has helped the innkeeper with several tasks");
        self.set_relationship("player", "bandit_leader", RelationshipType::Enemy, -80, "The player defeated the bandit leader's lieutenants");
        
        self.set_relationship("wizard", "merchant", RelationshipType::Acquaintance, 20, "The wizard occasionally buys rare ingredients from the merchant");
        self.set_relationship("wizard", "guard_captain", RelationshipType::Ally, 50, "The wizard has helped the guard captain with magical problems");
        
        self.set_relationship("merchant", "innkeeper", RelationshipType::Business, 60, "The merchant supplies the inn with goods");
        self.set_relationship("merchant", "guard_captain", RelationshipType::Acquaintance, 10, "The merchant pays taxes regularly");
        
        self.set_relationship("guard_captain", "bandit_leader", RelationshipType::Enemy, -90, "The guard captain has been hunting the bandit leader for years");
        
        // Save relationships
        if let Err(e) = self.save_relationships() {
            error!("Failed to save default relationships: {}", e);
        }
    }
}