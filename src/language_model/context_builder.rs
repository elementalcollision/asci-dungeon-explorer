use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use log::{info, warn, error};

use super::dialogue_system_trait::{DialogueContext, CharacterPersona, DialogueEntry};
use super::character_persona::{ExtendedPersona, KnowledgeItem, Relationship};

/// Context source for gathering information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContextSource {
    Character(String),
    Location(String),
    Item(String),
    Quest(String),
    Faction(String),
    WorldState(String),
    GameEvent(String),
    Custom(String, String),
}

/// Context element with priority
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextElement {
    pub source: ContextSource,
    pub content: String,
    pub priority: u8, // 0-100, higher is more important
    pub timestamp: u64,
    pub metadata: HashMap<String, String>,
}

/// Knowledge graph node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeNode {
    pub id: String,
    pub content: String,
    pub node_type: String,
    pub connections: Vec<KnowledgeConnection>,
    pub metadata: HashMap<String, String>,
}

/// Connection between knowledge nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeConnection {
    pub target_id: String,
    pub relationship_type: String,
    pub strength: u8, // 0-100
    pub metadata: HashMap<String, String>,
}

/// Context builder for creating dialogue contexts
pub struct ContextBuilder {
    character_knowledge: HashMap<String, Vec<KnowledgeItem>>,
    location_descriptions: HashMap<String, String>,
    faction_relationships: HashMap<String, HashMap<String, i32>>,
    world_state: HashMap<String, String>,
    recent_events: Vec<String>,
    knowledge_graph: HashMap<String, KnowledgeNode>,
}

impl ContextBuilder {
    /// Create a new context builder
    pub fn new() -> Self {
        ContextBuilder {
            character_knowledge: HashMap::new(),
            location_descriptions: HashMap::new(),
            faction_relationships: HashMap::new(),
            world_state: HashMap::new(),
            recent_events: Vec::new(),
            knowledge_graph: HashMap::new(),
        }
    }
    
    /// Add character knowledge
    pub fn add_character_knowledge(&mut self, character_id: &str, topic: &str, content: &str, certainty: u8, source: &str) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        let knowledge = KnowledgeItem {
            topic: topic.to_string(),
            content: content.to_string(),
            certainty,
            source: source.to_string(),
            timestamp: now,
        };
        
        self.character_knowledge
            .entry(character_id.to_string())
            .or_insert_with(Vec::new)
            .push(knowledge);
    }
    
    /// Add location description
    pub fn add_location_description(&mut self, location_id: &str, description: &str) {
        self.location_descriptions.insert(location_id.to_string(), description.to_string());
    }
    
    /// Add faction relationship
    pub fn add_faction_relationship(&mut self, faction_id: &str, other_faction_id: &str, relationship: i32) {
        self.faction_relationships
            .entry(faction_id.to_string())
            .or_insert_with(HashMap::new)
            .insert(other_faction_id.to_string(), relationship);
    }
    
    /// Add world state
    pub fn add_world_state(&mut self, key: &str, value: &str) {
        self.world_state.insert(key.to_string(), value.to_string());
    }
    
    /// Add recent event
    pub fn add_recent_event(&mut self, event: &str) {
        self.recent_events.push(event.to_string());
        
        // Keep only the most recent events
        if self.recent_events.len() > 10 {
            self.recent_events.remove(0);
        }
    }
    
    /// Add knowledge node
    pub fn add_knowledge_node(&mut self, node: KnowledgeNode) {
        self.knowledge_graph.insert(node.id.clone(), node);
    }
    
    /// Connect knowledge nodes
    pub fn connect_knowledge_nodes(&mut self, source_id: &str, target_id: &str, relationship_type: &str, strength: u8) {
        if let Some(node) = self.knowledge_graph.get_mut(source_id) {
            // Check if connection already exists
            if !node.connections.iter().any(|c| c.target_id == target_id && c.relationship_type == relationship_type) {
                node.connections.push(KnowledgeConnection {
                    target_id: target_id.to_string(),
                    relationship_type: relationship_type.to_string(),
                    strength,
                    metadata: HashMap::new(),
                });
            }
        }
    }
    
    /// Build a dialogue context for a character
    pub fn build_dialogue_context(&self, persona: &CharacterPersona, location: &str) -> DialogueContext {
        let mut context = DialogueContext {
            character_id: persona.id.clone(),
            character_name: persona.name.clone(),
            character_description: persona.description.clone(),
            location: location.to_string(),
            history: Vec::new(),
            relationship: 0, // Default neutral
            knowledge: Vec::new(),
            traits: persona.traits.clone(),
            faction: persona.faction.clone(),
            metadata: HashMap::new(),
        };
        
        // Add location description if available
        if let Some(description) = self.location_descriptions.get(location) {
            context.metadata.insert("location_description".to_string(), description.clone());
        }
        
        // Add character knowledge
        if let Some(knowledge_items) = self.character_knowledge.get(&persona.id) {
            for item in knowledge_items {
                // Only add knowledge with high certainty
                if item.certainty > 50 {
                    context.knowledge.push(item.content.clone());
                }
            }
        }
        
        // Add faction relationships if character has a faction
        if let Some(faction_id) = &persona.faction {
            if let Some(relationships) = self.faction_relationships.get(faction_id) {
                let relationships_str = relationships.iter()
                    .map(|(faction, value)| format!("{}: {}", faction, self.relationship_description(*value)))
                    .collect::<Vec<_>>()
                    .join(", ");
                
                context.metadata.insert("faction_relationships".to_string(), relationships_str);
            }
        }
        
        // Add relevant world state
        let mut relevant_world_state = Vec::new();
        for (key, value) in &self.world_state {
            // Add world state that might be relevant to this character
            // This is a simple implementation - in a real game, you'd have more sophisticated relevance checks
            if persona.knowledge_base.iter().any(|k| k.contains(key)) {
                relevant_world_state.push(format!("{}: {}", key, value));
            }
        }
        
        if !relevant_world_state.is_empty() {
            context.metadata.insert("world_state".to_string(), relevant_world_state.join(", "));
        }
        
        // Add recent events
        if !self.recent_events.is_empty() {
            context.metadata.insert("recent_events".to_string(), self.recent_events.join(", "));
        }
        
        context
    }
    
    /// Build a dialogue context from an extended persona
    pub fn build_context_from_extended_persona(&self, persona: &ExtendedPersona, location: &str) -> DialogueContext {
        let mut context = self.build_dialogue_context(&persona.base, location);
        
        // Add knowledge from extended persona
        for item in &persona.knowledge_base {
            if item.certainty > 50 {
                context.knowledge.push(item.content.clone());
            }
        }
        
        // Add player relationship if it exists
        if let Some(relationship) = persona.relationships.get("player") {
            context.relationship = relationship.value;
        }
        
        // Add dominant emotion if any
        if let Some(emotion) = persona.get_dominant_emotion() {
            context.metadata.insert("current_emotion".to_string(), emotion.name.clone());
            context.metadata.insert("emotion_intensity".to_string(), emotion.intensity.to_string());
        }
        
        // Add personality stats
        if !persona.personality_stats.is_empty() {
            let stats_str = persona.personality_stats.iter()
                .map(|(stat, value)| format!("{}: {}", stat, value))
                .collect::<Vec<_>>()
                .join(", ");
            
            context.metadata.insert("personality_stats".to_string(), stats_str);
        }
        
        // Add goals
        if !persona.goals.is_empty() {
            context.metadata.insert("goals".to_string(), persona.goals.join(", "));
        }
        
        // Add fears
        if !persona.fears.is_empty() {
            context.metadata.insert("fears".to_string(), persona.fears.join(", "));
        }
        
        context
    }
    
    /// Format a prompt for dialogue generation
    pub fn format_dialogue_prompt(&self, context: &DialogueContext, player_input: &str) -> String {
        let mut prompt = String::new();
        
        // Add character information
        prompt.push_str(&format!("Character: {}\n", context.character_name));
        prompt.push_str(&format!("Description: {}\n", context.character_description));
        prompt.push_str(&format!("Location: {}\n", context.location));
        
        // Add location description if available
        if let Some(description) = context.metadata.get("location_description") {
            prompt.push_str(&format!("Location Description: {}\n", description));
        }
        
        // Add relationship with player
        prompt.push_str(&format!("Relationship with player: {}\n", self.relationship_description(context.relationship)));
        
        // Add current emotion if available
        if let Some(emotion) = context.metadata.get("current_emotion") {
            if let Some(intensity) = context.metadata.get("emotion_intensity") {
                prompt.push_str(&format!("Current Emotion: {} (Intensity: {})\n", emotion, intensity));
            } else {
                prompt.push_str(&format!("Current Emotion: {}\n", emotion));
            }
        }
        
        // Add traits
        if !context.traits.is_empty() {
            prompt.push_str("Traits: ");
            prompt.push_str(&context.traits.join(", "));
            prompt.push_str("\n");
        }
        
        // Add faction
        if let Some(faction) = &context.faction {
            prompt.push_str(&format!("Faction: {}\n", faction));
            
            // Add faction relationships if available
            if let Some(relationships) = context.metadata.get("faction_relationships") {
                prompt.push_str(&format!("Faction Relationships: {}\n", relationships));
            }
        }
        
        // Add goals if available
        if let Some(goals) = context.metadata.get("goals") {
            prompt.push_str(&format!("Goals: {}\n", goals));
        }
        
        // Add fears if available
        if let Some(fears) = context.metadata.get("fears") {
            prompt.push_str(&format!("Fears: {}\n", fears));
        }
        
        // Add knowledge
        if !context.knowledge.is_empty() {
            prompt.push_str("Knowledge:\n");
            for item in &context.knowledge {
                prompt.push_str(&format!("- {}\n", item));
            }
        }
        
        // Add world state if available
        if let Some(world_state) = context.metadata.get("world_state") {
            prompt.push_str(&format!("World State: {}\n", world_state));
        }
        
        // Add recent events if available
        if let Some(events) = context.metadata.get("recent_events") {
            prompt.push_str(&format!("Recent Events: {}\n", events));
        }
        
        // Add conversation history
        prompt.push_str("\nConversation:\n");
        
        // Limit history to a reasonable number of entries
        let max_history = 10;
        let start_idx = if context.history.len() > max_history {
            context.history.len() - max_history
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
    
    /// Format a system prompt based on character persona
    pub fn format_system_prompt(&self, persona: &CharacterPersona) -> String {
        let mut prompt = String::new();
        
        prompt.push_str("You are an NPC in a fantasy roguelike game. ");
        prompt.push_str(&format!("Your name is {} and you are {}. ", persona.name, persona.description));
        
        if !persona.traits.is_empty() {
            prompt.push_str(&format!("Your personality traits include: {}. ", persona.traits.join(", ")));
        }
        
        if !persona.speech_style.is_empty() {
            prompt.push_str(&format!("You speak in a {} style. ", persona.speech_style));
        }
        
        if let Some(faction) = &persona.faction {
            prompt.push_str(&format!("You belong to the {} faction. ", faction));
        }
        
        prompt.push_str("Respond in character based on your personality, knowledge, and relationship with the player. ");
        prompt.push_str("Keep responses concise (1-3 sentences) and appropriate to the fantasy setting. ");
        prompt.push_str("You may express emotions by starting your response with [emotion], e.g. [happy], [angry], [confused], etc. ");
        
        prompt
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
    
    /// Get relevant knowledge nodes for a context
    pub fn get_relevant_knowledge(&self, context: &DialogueContext, topic: &str) -> Vec<&KnowledgeNode> {
        let mut relevant_nodes = Vec::new();
        
        // Find nodes directly related to the topic
        for node in self.knowledge_graph.values() {
            if node.content.to_lowercase().contains(&topic.to_lowercase()) || 
               node.node_type.to_lowercase() == topic.to_lowercase() {
                relevant_nodes.push(node);
                
                // Also add connected nodes
                for connection in &node.connections {
                    if let Some(connected_node) = self.knowledge_graph.get(&connection.target_id) {
                        if !relevant_nodes.contains(&connected_node) {
                            relevant_nodes.push(connected_node);
                        }
                    }
                }
            }
        }
        
        // Sort by relevance (for now, just by number of connections)
        relevant_nodes.sort_by(|a, b| b.connections.len().cmp(&a.connections.len()));
        
        relevant_nodes
    }
    
    /// Create a context element from a knowledge node
    pub fn create_context_element_from_node(&self, node: &KnowledgeNode) -> ContextElement {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        ContextElement {
            source: ContextSource::Custom(node.node_type.clone(), node.id.clone()),
            content: node.content.clone(),
            priority: 50, // Default medium priority
            timestamp: now,
            metadata: node.metadata.clone(),
        }
    }
    
    /// Build a context for a specific topic
    pub fn build_topic_context(&self, context: &DialogueContext, topic: &str) -> Vec<ContextElement> {
        let mut elements = Vec::new();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        // Add character knowledge about the topic
        for knowledge in &context.knowledge {
            if knowledge.to_lowercase().contains(&topic.to_lowercase()) {
                elements.push(ContextElement {
                    source: ContextSource::Character(context.character_id.clone()),
                    content: knowledge.clone(),
                    priority: 80, // High priority for character's own knowledge
                    timestamp: now,
                    metadata: HashMap::new(),
                });
            }
        }
        
        // Add location information if topic is related to location
        if context.location.to_lowercase().contains(&topic.to_lowercase()) {
            if let Some(description) = self.location_descriptions.get(&context.location) {
                elements.push(ContextElement {
                    source: ContextSource::Location(context.location.clone()),
                    content: description.clone(),
                    priority: 70,
                    timestamp: now,
                    metadata: HashMap::new(),
                });
            }
        }
        
        // Add faction information if topic is related to factions
        if let Some(faction) = &context.faction {
            if faction.to_lowercase().contains(&topic.to_lowercase()) || topic.to_lowercase().contains("faction") {
                if let Some(relationships) = self.faction_relationships.get(faction) {
                    for (other_faction, relationship) in relationships {
                        elements.push(ContextElement {
                            source: ContextSource::Faction(faction.clone()),
                            content: format!("Relationship with {}: {}", other_faction, self.relationship_description(*relationship)),
                            priority: 60,
                            timestamp: now,
                            metadata: HashMap::new(),
                        });
                    }
                }
            }
        }
        
        // Add world state information related to the topic
        for (key, value) in &self.world_state {
            if key.to_lowercase().contains(&topic.to_lowercase()) || value.to_lowercase().contains(&topic.to_lowercase()) {
                elements.push(ContextElement {
                    source: ContextSource::WorldState(key.clone()),
                    content: format!("{}: {}", key, value),
                    priority: 50,
                    timestamp: now,
                    metadata: HashMap::new(),
                });
            }
        }
        
        // Add recent events related to the topic
        for event in &self.recent_events {
            if event.to_lowercase().contains(&topic.to_lowercase()) {
                elements.push(ContextElement {
                    source: ContextSource::GameEvent("recent".to_string()),
                    content: event.clone(),
                    priority: 90, // Very high priority for recent events
                    timestamp: now,
                    metadata: HashMap::new(),
                });
            }
        }
        
        // Add knowledge graph nodes
        for node in self.get_relevant_knowledge(context, topic) {
            elements.push(self.create_context_element_from_node(node));
        }
        
        // Sort by priority (highest first)
        elements.sort_by(|a, b| b.priority.cmp(&a.priority));
        
        elements
    }
    
    /// Format a prompt with specific context elements
    pub fn format_prompt_with_elements(&self, context: &DialogueContext, elements: &[ContextElement], player_input: &str) -> String {
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
        
        // Add context elements
        if !elements.is_empty() {
            prompt.push_str("\nRelevant Information:\n");
            
            // Limit to top 5 elements to avoid overloading the context
            for element in elements.iter().take(5) {
                match &element.source {
                    ContextSource::Character(_) => prompt.push_str("You know: "),
                    ContextSource::Location(_) => prompt.push_str("About this location: "),
                    ContextSource::Faction(_) => prompt.push_str("Faction information: "),
                    ContextSource::WorldState(_) => prompt.push_str("World state: "),
                    ContextSource::GameEvent(_) => prompt.push_str("Recent event: "),
                    _ => prompt.push_str("Information: "),
                }
                
                prompt.push_str(&format!("{}\n", element.content));
            }
        }
        
        // Add conversation history
        prompt.push_str("\nConversation:\n");
        
        // Limit history to a reasonable number of entries
        let max_history = 5; // Smaller history when we have specific context elements
        let start_idx = if context.history.len() > max_history {
            context.history.len() - max_history
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
}