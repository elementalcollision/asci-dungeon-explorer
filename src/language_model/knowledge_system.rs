use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::fs;
use std::io;
use serde::{Serialize, Deserialize};
use log::{info, warn, error};

use super::context_builder::{KnowledgeNode, KnowledgeConnection};

/// Knowledge category
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum KnowledgeCategory {
    Character,
    Location,
    Item,
    Quest,
    Faction,
    Lore,
    Event,
    Secret,
    Custom(String),
}

impl KnowledgeCategory {
    pub fn as_str(&self) -> &str {
        match self {
            KnowledgeCategory::Character => "character",
            KnowledgeCategory::Location => "location",
            KnowledgeCategory::Item => "item",
            KnowledgeCategory::Quest => "quest",
            KnowledgeCategory::Faction => "faction",
            KnowledgeCategory::Lore => "lore",
            KnowledgeCategory::Event => "event",
            KnowledgeCategory::Secret => "secret",
            KnowledgeCategory::Custom(s) => s,
        }
    }
    
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "character" => KnowledgeCategory::Character,
            "location" => KnowledgeCategory::Location,
            "item" => KnowledgeCategory::Item,
            "quest" => KnowledgeCategory::Quest,
            "faction" => KnowledgeCategory::Faction,
            "lore" => KnowledgeCategory::Lore,
            "event" => KnowledgeCategory::Event,
            "secret" => KnowledgeCategory::Secret,
            _ => KnowledgeCategory::Custom(s.to_string()),
        }
    }
}

/// Knowledge base entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeEntry {
    pub id: String,
    pub category: KnowledgeCategory,
    pub name: String,
    pub description: String,
    pub tags: Vec<String>,
    pub related_ids: Vec<String>,
    pub metadata: HashMap<String, String>,
    pub created_at: u64,
    pub updated_at: u64,
}

/// Character knowledge entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterKnowledge {
    pub character_id: String,
    pub knowledge_id: String,
    pub certainty: u8, // 0-100
    pub source: String,
    pub discovered_at: u64,
    pub last_recalled: u64,
    pub recall_count: u32,
    pub importance: u8, // 0-100
    pub notes: String,
}

/// Knowledge system for managing game knowledge
pub struct KnowledgeSystem {
    knowledge_base: HashMap<String, KnowledgeEntry>,
    character_knowledge: HashMap<String, HashMap<String, CharacterKnowledge>>,
    knowledge_directory: PathBuf,
    knowledge_graph: HashMap<String, KnowledgeNode>,
}

impl KnowledgeSystem {
    /// Create a new knowledge system
    pub fn new(knowledge_directory: PathBuf) -> Self {
        KnowledgeSystem {
            knowledge_base: HashMap::new(),
            character_knowledge: HashMap::new(),
            knowledge_directory,
            knowledge_graph: HashMap::new(),
        }
    }
    
    /// Initialize the knowledge system
    pub fn initialize(&mut self) -> io::Result<()> {
        // Create directory if it doesn't exist
        if !self.knowledge_directory.exists() {
            fs::create_dir_all(&self.knowledge_directory)?;
        }
        
        // Load knowledge base
        self.load_knowledge_base()?;
        
        // Build knowledge graph
        self.build_knowledge_graph();
        
        Ok(())
    }
    
    /// Load the knowledge base from files
    fn load_knowledge_base(&mut self) -> io::Result<()> {
        let knowledge_path = self.knowledge_directory.join("knowledge_base.json");
        
        if knowledge_path.exists() {
            let json = fs::read_to_string(&knowledge_path)?;
            let entries: Vec<KnowledgeEntry> = serde_json::from_str(&json)?;
            
            for entry in entries {
                self.knowledge_base.insert(entry.id.clone(), entry);
            }
            
            info!("Loaded {} knowledge entries", self.knowledge_base.len());
        }
        
        // Load character knowledge
        let character_knowledge_path = self.knowledge_directory.join("character_knowledge.json");
        
        if character_knowledge_path.exists() {
            let json = fs::read_to_string(&character_knowledge_path)?;
            let entries: HashMap<String, HashMap<String, CharacterKnowledge>> = serde_json::from_str(&json)?;
            
            self.character_knowledge = entries;
            
            let total_entries: usize = self.character_knowledge.values().map(|m| m.len()).sum();
            info!("Loaded {} character knowledge entries for {} characters", total_entries, self.character_knowledge.len());
        }
        
        Ok(())
    }
    
    /// Save the knowledge base to files
    pub fn save_knowledge_base(&self) -> io::Result<()> {
        let knowledge_path = self.knowledge_directory.join("knowledge_base.json");
        let entries: Vec<KnowledgeEntry> = self.knowledge_base.values().cloned().collect();
        
        let json = serde_json::to_string_pretty(&entries)?;
        fs::write(&knowledge_path, json)?;
        
        // Save character knowledge
        let character_knowledge_path = self.knowledge_directory.join("character_knowledge.json");
        let json = serde_json::to_string_pretty(&self.character_knowledge)?;
        fs::write(&character_knowledge_path, json)?;
        
        Ok(())
    }
    
    /// Build the knowledge graph from the knowledge base
    fn build_knowledge_graph(&mut self) {
        self.knowledge_graph.clear();
        
        // Create nodes
        for entry in self.knowledge_base.values() {
            let node = KnowledgeNode {
                id: entry.id.clone(),
                content: format!("{}: {}", entry.name, entry.description),
                node_type: entry.category.as_str().to_string(),
                connections: Vec::new(),
                metadata: entry.metadata.clone(),
            };
            
            self.knowledge_graph.insert(entry.id.clone(), node);
        }
        
        // Create connections
        for entry in self.knowledge_base.values() {
            if let Some(node) = self.knowledge_graph.get_mut(&entry.id) {
                for related_id in &entry.related_ids {
                    if self.knowledge_graph.contains_key(related_id) {
                        node.connections.push(KnowledgeConnection {
                            target_id: related_id.clone(),
                            relationship_type: "related".to_string(),
                            strength: 50, // Default medium strength
                            metadata: HashMap::new(),
                        });
                    }
                }
            }
        }
        
        info!("Built knowledge graph with {} nodes", self.knowledge_graph.len());
    }
    
    /// Add a knowledge entry
    pub fn add_knowledge_entry(&mut self, entry: KnowledgeEntry) {
        self.knowledge_base.insert(entry.id.clone(), entry);
        self.build_knowledge_graph();
    }
    
    /// Get a knowledge entry
    pub fn get_knowledge_entry(&self, id: &str) -> Option<&KnowledgeEntry> {
        self.knowledge_base.get(id)
    }
    
    /// Add character knowledge
    pub fn add_character_knowledge(&mut self, character_knowledge: CharacterKnowledge) {
        self.character_knowledge
            .entry(character_knowledge.character_id.clone())
            .or_insert_with(HashMap::new)
            .insert(character_knowledge.knowledge_id.clone(), character_knowledge);
    }
    
    /// Get character knowledge
    pub fn get_character_knowledge(&self, character_id: &str, knowledge_id: &str) -> Option<&CharacterKnowledge> {
        self.character_knowledge
            .get(character_id)
            .and_then(|map| map.get(knowledge_id))
    }
    
    /// Get all knowledge for a character
    pub fn get_all_character_knowledge(&self, character_id: &str) -> Vec<(&KnowledgeEntry, &CharacterKnowledge)> {
        let mut result = Vec::new();
        
        if let Some(knowledge_map) = self.character_knowledge.get(character_id) {
            for (knowledge_id, character_knowledge) in knowledge_map {
                if let Some(entry) = self.knowledge_base.get(knowledge_id) {
                    result.push((entry, character_knowledge));
                }
            }
        }
        
        result
    }
    
    /// Get knowledge entries by category
    pub fn get_entries_by_category(&self, category: &KnowledgeCategory) -> Vec<&KnowledgeEntry> {
        self.knowledge_base.values()
            .filter(|entry| entry.category == *category)
            .collect()
    }
    
    /// Search knowledge entries by text
    pub fn search_entries(&self, query: &str) -> Vec<&KnowledgeEntry> {
        let query = query.to_lowercase();
        
        self.knowledge_base.values()
            .filter(|entry| {
                entry.name.to_lowercase().contains(&query) ||
                entry.description.to_lowercase().contains(&query) ||
                entry.tags.iter().any(|tag| tag.to_lowercase().contains(&query))
            })
            .collect()
    }
    
    /// Get related knowledge entries
    pub fn get_related_entries(&self, id: &str) -> Vec<&KnowledgeEntry> {
        if let Some(entry) = self.knowledge_base.get(id) {
            entry.related_ids.iter()
                .filter_map(|related_id| self.knowledge_base.get(related_id))
                .collect()
        } else {
            Vec::new()
        }
    }
    
    /// Update character knowledge certainty
    pub fn update_knowledge_certainty(&mut self, character_id: &str, knowledge_id: &str, certainty: u8) {
        if let Some(knowledge_map) = self.character_knowledge.get_mut(character_id) {
            if let Some(character_knowledge) = knowledge_map.get_mut(knowledge_id) {
                character_knowledge.certainty = certainty;
                character_knowledge.last_recalled = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();
                character_knowledge.recall_count += 1;
            }
        }
    }
    
    /// Record knowledge recall
    pub fn record_knowledge_recall(&mut self, character_id: &str, knowledge_id: &str) {
        if let Some(knowledge_map) = self.character_knowledge.get_mut(character_id) {
            if let Some(character_knowledge) = knowledge_map.get_mut(knowledge_id) {
                character_knowledge.last_recalled = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();
                character_knowledge.recall_count += 1;
            }
        }
    }
    
    /// Get knowledge node
    pub fn get_knowledge_node(&self, id: &str) -> Option<&KnowledgeNode> {
        self.knowledge_graph.get(id)
    }
    
    /// Get all knowledge nodes
    pub fn get_all_knowledge_nodes(&self) -> Vec<&KnowledgeNode> {
        self.knowledge_graph.values().collect()
    }
    
    /// Get knowledge nodes by type
    pub fn get_nodes_by_type(&self, node_type: &str) -> Vec<&KnowledgeNode> {
        self.knowledge_graph.values()
            .filter(|node| node.node_type == node_type)
            .collect()
    }
    
    /// Get connected nodes
    pub fn get_connected_nodes(&self, id: &str) -> Vec<&KnowledgeNode> {
        if let Some(node) = self.knowledge_graph.get(id) {
            node.connections.iter()
                .filter_map(|conn| self.knowledge_graph.get(&conn.target_id))
                .collect()
        } else {
            Vec::new()
        }
    }
    
    /// Get nodes by content search
    pub fn search_nodes(&self, query: &str) -> Vec<&KnowledgeNode> {
        let query = query.to_lowercase();
        
        self.knowledge_graph.values()
            .filter(|node| node.content.to_lowercase().contains(&query))
            .collect()
    }
    
    /// Create a knowledge entry from a node
    pub fn create_entry_from_node(&self, node: &KnowledgeNode) -> KnowledgeEntry {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        // Extract name and description from content
        let parts: Vec<&str> = node.content.splitn(2, ": ").collect();
        let (name, description) = if parts.len() == 2 {
            (parts[0].to_string(), parts[1].to_string())
        } else {
            (format!("Node {}", node.id), node.content.clone())
        };
        
        // Get related IDs from connections
        let related_ids = node.connections.iter()
            .map(|conn| conn.target_id.clone())
            .collect();
        
        KnowledgeEntry {
            id: node.id.clone(),
            category: KnowledgeCategory::from_str(&node.node_type),
            name,
            description,
            tags: Vec::new(),
            related_ids,
            metadata: node.metadata.clone(),
            created_at: now,
            updated_at: now,
        }
    }
    
    /// Create a character knowledge entry
    pub fn create_character_knowledge(&self, character_id: &str, knowledge_id: &str, certainty: u8, source: &str) -> CharacterKnowledge {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        CharacterKnowledge {
            character_id: character_id.to_string(),
            knowledge_id: knowledge_id.to_string(),
            certainty,
            source: source.to_string(),
            discovered_at: now,
            last_recalled: now,
            recall_count: 1,
            importance: 50, // Default medium importance
            notes: String::new(),
        }
    }
    
    /// Generate knowledge text for a character
    pub fn generate_knowledge_text(&self, character_id: &str) -> Vec<String> {
        let mut result = Vec::new();
        
        // Get all knowledge for the character
        let character_knowledge = self.get_all_character_knowledge(character_id);
        
        // Sort by importance and certainty
        let mut sorted_knowledge = character_knowledge;
        sorted_knowledge.sort_by(|(_, a), (_, b)| {
            let a_score = a.importance as u16 + a.certainty as u16;
            let b_score = b.importance as u16 + b.certainty as u16;
            b_score.cmp(&a_score)
        });
        
        // Generate text for each knowledge entry
        for (entry, character_knowledge) in sorted_knowledge {
            // Only include knowledge with reasonable certainty
            if character_knowledge.certainty >= 30 {
                let certainty_prefix = if character_knowledge.certainty < 50 {
                    "I think "
                } else if character_knowledge.certainty < 80 {
                    "I know "
                } else {
                    "I am certain "
                };
                
                result.push(format!("{}{}", certainty_prefix, entry.description));
            }
        }
        
        result
    }
    
    /// Generate knowledge text for a specific topic
    pub fn generate_topic_knowledge(&self, character_id: &str, topic: &str) -> Vec<String> {
        let topic = topic.to_lowercase();
        let mut result = Vec::new();
        
        // Get all knowledge for the character
        let character_knowledge = self.get_all_character_knowledge(character_id);
        
        // Filter by topic relevance
        let relevant_knowledge: Vec<_> = character_knowledge.into_iter()
            .filter(|(entry, _)| {
                entry.name.to_lowercase().contains(&topic) ||
                entry.description.to_lowercase().contains(&topic) ||
                entry.tags.iter().any(|tag| tag.to_lowercase().contains(&topic))
            })
            .collect();
        
        // Sort by importance and certainty
        let mut sorted_knowledge = relevant_knowledge;
        sorted_knowledge.sort_by(|(_, a), (_, b)| {
            let a_score = a.importance as u16 + a.certainty as u16;
            let b_score = b.importance as u16 + b.certainty as u16;
            b_score.cmp(&a_score)
        });
        
        // Generate text for each knowledge entry
        for (entry, character_knowledge) in sorted_knowledge {
            // Only include knowledge with reasonable certainty
            if character_knowledge.certainty >= 30 {
                let certainty_prefix = if character_knowledge.certainty < 50 {
                    "I think "
                } else if character_knowledge.certainty < 80 {
                    "I know "
                } else {
                    "I am certain "
                };
                
                result.push(format!("{}{}", certainty_prefix, entry.description));
            }
        }
        
        result
    }
    
    /// Create a default knowledge base with some example entries
    pub fn create_default_knowledge_base(&mut self) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        // Create some example knowledge entries
        let entries = vec![
            KnowledgeEntry {
                id: "location-town".to_string(),
                category: KnowledgeCategory::Location,
                name: "Ravenhollow".to_string(),
                description: "A small town nestled in the shadow of the Misty Mountains, known for its skilled blacksmiths and suspicious townsfolk.".to_string(),
                tags: vec!["town".to_string(), "starting area".to_string()],
                related_ids: vec!["faction-townspeople".to_string(), "location-mountains".to_string()],
                metadata: HashMap::new(),
                created_at: now,
                updated_at: now,
            },
            KnowledgeEntry {
                id: "location-mountains".to_string(),
                category: KnowledgeCategory::Location,
                name: "Misty Mountains".to_string(),
                description: "A treacherous mountain range that separates the civilized lands from the wild territories. Home to dangerous creatures and hidden treasures.".to_string(),
                tags: vec!["mountains".to_string(), "dangerous".to_string()],
                related_ids: vec!["location-town".to_string(), "location-dungeon".to_string()],
                metadata: HashMap::new(),
                created_at: now,
                updated_at: now,
            },
            KnowledgeEntry {
                id: "location-dungeon".to_string(),
                category: KnowledgeCategory::Location,
                name: "Forgotten Depths".to_string(),
                description: "An ancient dungeon complex beneath the Misty Mountains, rumored to contain powerful artifacts from a lost civilization.".to_string(),
                tags: vec!["dungeon".to_string(), "treasure".to_string()],
                related_ids: vec!["location-mountains".to_string(), "item-orb".to_string()],
                metadata: HashMap::new(),
                created_at: now,
                updated_at: now,
            },
            KnowledgeEntry {
                id: "item-orb".to_string(),
                category: KnowledgeCategory::Item,
                name: "Orb of Zephyr".to_string(),
                description: "A legendary artifact said to grant control over the winds and weather. Last seen in the possession of the ancient mage Zephyrus.".to_string(),
                tags: vec!["artifact".to_string(), "magic".to_string()],
                related_ids: vec!["location-dungeon".to_string(), "character-zephyrus".to_string()],
                metadata: HashMap::new(),
                created_at: now,
                updated_at: now,
            },
            KnowledgeEntry {
                id: "character-zephyrus".to_string(),
                category: KnowledgeCategory::Character,
                name: "Zephyrus".to_string(),
                description: "An ancient mage who mastered the art of weather manipulation. Disappeared centuries ago, but legends say he sealed himself in a hidden chamber with his greatest creation.".to_string(),
                tags: vec!["mage".to_string(), "historical".to_string()],
                related_ids: vec!["item-orb".to_string(), "faction-mages".to_string()],
                metadata: HashMap::new(),
                created_at: now,
                updated_at: now,
            },
            KnowledgeEntry {
                id: "faction-townspeople".to_string(),
                category: KnowledgeCategory::Faction,
                name: "Ravenhollow Townsfolk".to_string(),
                description: "The people of Ravenhollow are hardy and suspicious of outsiders, but loyal to those who earn their trust. They have a long tradition of metalworking.".to_string(),
                tags: vec!["faction".to_string(), "neutral".to_string()],
                related_ids: vec!["location-town".to_string(), "faction-mages".to_string()],
                metadata: HashMap::new(),
                created_at: now,
                updated_at: now,
            },
            KnowledgeEntry {
                id: "faction-mages".to_string(),
                category: KnowledgeCategory::Faction,
                name: "Order of the Azure Flame".to_string(),
                description: "A secretive order of mages dedicated to the study and preservation of ancient magical knowledge. They have a hidden enclave somewhere in the Misty Mountains.".to_string(),
                tags: vec!["faction".to_string(), "magic".to_string()],
                related_ids: vec!["character-zephyrus".to_string(), "faction-townspeople".to_string()],
                metadata: HashMap::new(),
                created_at: now,
                updated_at: now,
            },
        ];
        
        // Add entries to knowledge base
        for entry in entries {
            self.knowledge_base.insert(entry.id.clone(), entry);
        }
        
        // Build knowledge graph
        self.build_knowledge_graph();
        
        // Save knowledge base
        if let Err(e) = self.save_knowledge_base() {
            error!("Failed to save default knowledge base: {}", e);
        }
    }
}