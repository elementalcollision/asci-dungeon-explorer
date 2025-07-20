use std::collections::{HashMap, HashSet};
use serde::{Serialize, Deserialize};
use crate::progression::milestone_system::MilestoneReward;

/// Types of unlockable content
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ContentType {
    Area,
    Feature,
    Item,
    Skill,
    Ability,
    Character,
    GameMode,
    Cosmetic,
    Title,
    Achievement,
    Recipe,
    Vendor,
}

/// Content rarity levels
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ContentRarity {
    Common,
    Uncommon,
    Rare,
    Epic,
    Legendary,
}

/// Unlock conditions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UnlockCondition {
    Milestone(String),
    Achievement(String),
    Level(u32),
    Multiple(Vec<UnlockCondition>), // All conditions must be met
    Any(Vec<UnlockCondition>), // Any condition can be met
    Custom(String), // Custom condition identifier
}

/// Unlockable content definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnlockableContent {
    pub id: String,
    pub name: String,
    pub description: String,
    pub content_type: ContentType,
    pub rarity: ContentRarity,
    pub unlock_condition: UnlockCondition,
    pub prerequisites: Vec<String>,
    pub icon: String,
    pub preview_available: bool,
    pub hint: Option<String>, // Hint for how to unlock
    pub unlock_message: String,
    pub metadata: HashMap<String, String>, // Additional content-specific data
}

impl UnlockableContent {
    pub fn new(
        id: String,
        name: String,
        description: String,
        content_type: ContentType,
        rarity: ContentRarity,
        unlock_condition: UnlockCondition,
    ) -> Self {
        UnlockableContent {
            id: id.clone(),
            name: name.clone(),
            description,
            content_type,
            rarity,
            unlock_condition,
            prerequisites: Vec::new(),
            icon: "🔒".to_string(),
            preview_available: false,
            hint: None,
            unlock_message: format!("Unlocked: {}", name),
            metadata: HashMap::new(),
        }
    }

    pub fn with_prerequisites(mut self, prerequisites: Vec<String>) -> Self {
        self.prerequisites = prerequisites;
        self
    }

    pub fn with_icon(mut self, icon: String) -> Self {
        self.icon = icon;
        self
    }

    pub fn with_preview(mut self, preview_available: bool) -> Self {
        self.preview_available = preview_available;
        self
    }

    pub fn with_hint(mut self, hint: String) -> Self {
        self.hint = Some(hint);
        self
    }

    pub fn with_unlock_message(mut self, message: String) -> Self {
        self.unlock_message = message;
        self
    }

    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

/// Unlocked content record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnlockedContentRecord {
    pub content_id: String,
    pub unlocked_at: u64,
    pub unlock_source: String, // What unlocked this content
    pub first_accessed: Option<u64>,
    pub access_count: u32,
}

impl UnlockedContentRecord {
    pub fn new(content_id: String, unlock_source: String) -> Self {
        UnlockedContentRecord {
            content_id,
            unlocked_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            unlock_source,
            first_accessed: None,
            access_count: 0,
        }
    }

    pub fn access(&mut self) {
        if self.first_accessed.is_none() {
            self.first_accessed = Some(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs()
            );
        }
        self.access_count += 1;
    }
}

/// Content unlock statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentUnlockStatistics {
    pub total_content: usize,
    pub unlocked_content: usize,
    pub locked_content: usize,
    pub unlock_percentage: f32,
    pub type_unlocks: HashMap<ContentType, (usize, usize)>, // (total, unlocked)
    pub rarity_unlocks: HashMap<ContentRarity, (usize, usize)>,
    pub recent_unlocks: Vec<String>,
    pub most_accessed: Vec<(String, u32)>,
}

/// Unlockable content system
pub struct UnlockableContentSystem {
    content: HashMap<String, UnlockableContent>,
    unlocked_content: HashMap<String, UnlockedContentRecord>,
    content_dependencies: HashMap<String, Vec<String>>, // What each content unlocks
    statistics: ContentUnlockStatistics,
}

impl UnlockableContentSystem {
    pub fn new() -> Self {
        let mut system = UnlockableContentSystem {
            content: HashMap::new(),
            unlocked_content: HashMap::new(),
            content_dependencies: HashMap::new(),
            statistics: ContentUnlockStatistics {
                total_content: 0,
                unlocked_content: 0,
                locked_content: 0,
                unlock_percentage: 0.0,
                type_unlocks: HashMap::new(),
                rarity_unlocks: HashMap::new(),
                recent_unlocks: Vec::new(),
                most_accessed: Vec::new(),
            },
        };

        // Initialize with default unlockable content
        system.initialize_default_content();
        system.update_statistics();

        system
    }

    /// Initialize default unlockable content
    fn initialize_default_content(&mut self) {
        let content_items = vec![
            // Areas
            UnlockableContent::new(
                "deep_levels".to_string(),
                "Deep Dungeon Levels".to_string(),
                "Access to dungeon levels 11-20".to_string(),
                ContentType::Area,
                ContentRarity::Uncommon,
                UnlockCondition::Milestone("deep_explorer".to_string()),
            ).with_icon("🏰".to_string())
            .with_hint("Explore deep into the dungeon".to_string())
            .with_metadata("min_level".to_string(), "11".to_string())
            .with_metadata("max_level".to_string(), "20".to_string()),

            UnlockableContent::new(
                "veteran_areas".to_string(),
                "Veteran Areas".to_string(),
                "Special areas for experienced adventurers".to_string(),
                ContentType::Area,
                ContentRarity::Rare,
                UnlockCondition::Milestone("veteran_adventurer".to_string()),
            ).with_icon("🎖️".to_string())
            .with_hint("Reach veteran status".to_string()),

            UnlockableContent::new(
                "dragon_lair".to_string(),
                "Dragon's Lair".to_string(),
                "The ancient dragon's treasure chamber".to_string(),
                ContentType::Area,
                ContentRarity::Legendary,
                UnlockCondition::Milestone("dragon_slayer".to_string()),
            ).with_icon("🐉".to_string())
            .with_hint("Defeat the ancient dragon".to_string()),

            // Features
            UnlockableContent::new(
                "combat_tutorial".to_string(),
                "Combat Tutorial".to_string(),
                "Learn advanced combat techniques".to_string(),
                ContentType::Feature,
                ContentRarity::Common,
                UnlockCondition::Milestone("first_blood".to_string()),
            ).with_icon("⚔️".to_string())
            .with_preview(true),

            UnlockableContent::new(
                "map_system".to_string(),
                "Dungeon Map".to_string(),
                "Track your exploration progress".to_string(),
                ContentType::Feature,
                ContentRarity::Common,
                UnlockCondition::Milestone("first_steps".to_string()),
            ).with_icon("🗺️".to_string())
            .with_preview(true),

            UnlockableContent::new(
                "skill_tree".to_string(),
                "Skill Tree".to_string(),
                "Advanced character development system".to_string(),
                ContentType::Feature,
                ContentRarity::Uncommon,
                UnlockCondition::Milestone("growing_stronger".to_string()),
            ).with_icon("🌳".to_string())
            .with_hint("Advance your character".to_string()),

            UnlockableContent::new(
                "merchant_discounts".to_string(),
                "Merchant Discounts".to_string(),
                "Better prices from all merchants".to_string(),
                ContentType::Feature,
                ContentRarity::Uncommon,
                UnlockCondition::Milestone("treasure_seeker".to_string()),
            ).with_icon("💰".to_string())
            .with_metadata("discount_percent".to_string(), "15".to_string()),

            // Items
            UnlockableContent::new(
                "depth_compass".to_string(),
                "Depth Compass".to_string(),
                "Always know your current depth".to_string(),
                ContentType::Item,
                ContentRarity::Rare,
                UnlockCondition::Milestone("deep_explorer".to_string()),
            ).with_icon("🧭".to_string())
            .with_metadata("item_type".to_string(), "tool".to_string()),

            UnlockableContent::new(
                "dragon_scale_armor".to_string(),
                "Dragon Scale Armor".to_string(),
                "Legendary armor crafted from dragon scales".to_string(),
                ContentType::Item,
                ContentRarity::Legendary,
                UnlockCondition::Milestone("dragon_slayer".to_string()),
            ).with_icon("🛡️".to_string())
            .with_metadata("armor_class".to_string(), "20".to_string())
            .with_metadata("fire_resistance".to_string(), "50".to_string()),

            UnlockableContent::new(
                "ancient_artifact".to_string(),
                "Ancient Artifact".to_string(),
                "A mysterious artifact of unknown power".to_string(),
                ContentType::Item,
                ContentRarity::Epic,
                UnlockCondition::Milestone("secret_keeper".to_string()),
            ).with_icon("🔮".to_string())
            .with_metadata("item_type".to_string(), "artifact".to_string()),

            // Abilities
            UnlockableContent::new(
                "combat_mastery".to_string(),
                "Combat Mastery".to_string(),
                "Increased damage and critical hit chance".to_string(),
                ContentType::Ability,
                ContentRarity::Rare,
                UnlockCondition::Milestone("warrior_path".to_string()),
            ).with_icon("⚡".to_string())
            .with_metadata("damage_bonus".to_string(), "25".to_string())
            .with_metadata("crit_bonus".to_string(), "10".to_string()),

            UnlockableContent::new(
                "endurance".to_string(),
                "Endurance".to_string(),
                "Increased health and stamina regeneration".to_string(),
                ContentType::Ability,
                ContentRarity::Uncommon,
                UnlockCondition::Milestone("survivor".to_string()),
            ).with_icon("💪".to_string())
            .with_metadata("health_regen".to_string(), "2".to_string())
            .with_metadata("stamina_regen".to_string(), "5".to_string()),

            // Titles
            UnlockableContent::new(
                "warrior_title".to_string(),
                "Warrior Title".to_string(),
                "Display 'Warrior' as your title".to_string(),
                ContentType::Title,
                ContentRarity::Uncommon,
                UnlockCondition::Milestone("warrior_path".to_string()),
            ).with_icon("🏷️".to_string())
            .with_metadata("title_text".to_string(), "Warrior".to_string()),

            UnlockableContent::new(
                "veteran_title".to_string(),
                "Veteran Title".to_string(),
                "Display 'Veteran' as your title".to_string(),
                ContentType::Title,
                ContentRarity::Rare,
                UnlockCondition::Milestone("veteran_adventurer".to_string()),
            ).with_icon("🎖️".to_string())
            .with_metadata("title_text".to_string(), "Veteran".to_string()),

            UnlockableContent::new(
                "dragon_slayer_title".to_string(),
                "Dragon Slayer Title".to_string(),
                "Display 'Dragon Slayer' as your title".to_string(),
                ContentType::Title,
                ContentRarity::Legendary,
                UnlockCondition::Milestone("dragon_slayer".to_string()),
            ).with_icon("🐲".to_string())
            .with_metadata("title_text".to_string(), "Dragon Slayer".to_string()),

            // Special areas
            UnlockableContent::new(
                "collector_vault".to_string(),
                "Collector's Vault".to_string(),
                "Special storage area for rare items".to_string(),
                ContentType::Area,
                ContentRarity::Epic,
                UnlockCondition::Milestone("master_collector".to_string()),
            ).with_icon("🏛️".to_string())
            .with_metadata("storage_slots".to_string(), "100".to_string()),

            UnlockableContent::new(
                "secret_areas".to_string(),
                "Secret Areas".to_string(),
                "Hidden areas throughout the dungeon".to_string(),
                ContentType::Area,
                ContentRarity::Epic,
                UnlockCondition::Milestone("secret_keeper".to_string()),
            ).with_icon("🔍".to_string())
            .with_hint("Find the hidden chamber".to_string()),
        ];

        for content in content_items {
            self.add_content(content);
        }
    }

    /// Add unlockable content to the system
    pub fn add_content(&mut self, content: UnlockableContent) {
        let id = content.id.clone();
        self.content.insert(id, content);
    }

    /// Check if content is unlocked
    pub fn is_content_unlocked(&self, content_id: &str) -> bool {
        self.unlocked_content.contains_key(content_id)
    }

    /// Unlock content
    pub fn unlock_content(&mut self, content_id: &str, unlock_source: &str) -> bool {
        if self.is_content_unlocked(content_id) {
            return false; // Already unlocked
        }

        if let Some(content) = self.content.get(content_id) {
            // Check prerequisites
            for prereq in &content.prerequisites {
                if !self.is_content_unlocked(prereq) {
                    return false; // Prerequisites not met
                }
            }

            // Unlock the content
            let record = UnlockedContentRecord::new(content_id.to_string(), unlock_source.to_string());
            self.unlocked_content.insert(content_id.to_string(), record);

            // Update dependencies
            self.update_content_dependencies(content_id);
            self.update_statistics();

            return true;
        }

        false
    }

    /// Update content dependencies when something is unlocked
    fn update_content_dependencies(&mut self, unlocked_content_id: &str) {
        // Check if this unlock enables other content
        for (content_id, content) in &self.content {
            if self.is_content_unlocked(content_id) {
                continue; // Already unlocked
            }

            // Check if this content's prerequisites are now met
            let prerequisites_met = content.prerequisites.iter()
                .all(|prereq| self.is_content_unlocked(prereq));

            if prerequisites_met {
                // Add to dependencies
                self.content_dependencies
                    .entry(unlocked_content_id.to_string())
                    .or_insert_with(Vec::new)
                    .push(content_id.clone());
            }
        }
    }

    /// Check unlock conditions against current game state
    pub fn check_unlock_conditions(
        &mut self,
        milestone_system: &crate::progression::milestone_system::MilestoneSystem,
        achievement_system: &crate::achievements::AchievementSystem,
        player_level: u32,
    ) -> Vec<String> {
        let mut newly_unlocked = Vec::new();

        for (content_id, content) in &self.content {
            if self.is_content_unlocked(content_id) {
                continue; // Already unlocked
            }

            let condition_met = self.evaluate_unlock_condition(
                &content.unlock_condition,
                milestone_system,
                achievement_system,
                player_level,
            );

            if condition_met {
                if self.unlock_content(content_id, "condition_met") {
                    newly_unlocked.push(content_id.clone());
                }
            }
        }

        newly_unlocked
    }

    /// Evaluate unlock condition
    fn evaluate_unlock_condition(
        &self,
        condition: &UnlockCondition,
        milestone_system: &crate::progression::milestone_system::MilestoneSystem,
        achievement_system: &crate::achievements::AchievementSystem,
        player_level: u32,
    ) -> bool {
        match condition {
            UnlockCondition::Milestone(milestone_id) => {
                milestone_system.get_milestone_status(milestone_id) == 
                crate::progression::milestone_system::MilestoneStatus::Completed
            },
            UnlockCondition::Achievement(achievement_id) => {
                achievement_system.is_unlocked(achievement_id)
            },
            UnlockCondition::Level(required_level) => {
                player_level >= *required_level
            },
            UnlockCondition::Multiple(conditions) => {
                conditions.iter().all(|cond| {
                    self.evaluate_unlock_condition(cond, milestone_system, achievement_system, player_level)
                })
            },
            UnlockCondition::Any(conditions) => {
                conditions.iter().any(|cond| {
                    self.evaluate_unlock_condition(cond, milestone_system, achievement_system, player_level)
                })
            },
            UnlockCondition::Custom(_) => {
                // Custom conditions would be evaluated based on game-specific logic
                false
            },
        }
    }

    /// Access content (for tracking usage)
    pub fn access_content(&mut self, content_id: &str) {
        if let Some(record) = self.unlocked_content.get_mut(content_id) {
            record.access();
            self.update_statistics();
        }
    }

    /// Get all content
    pub fn get_all_content(&self) -> Vec<&UnlockableContent> {
        self.content.values().collect()
    }

    /// Get unlocked content
    pub fn get_unlocked_content(&self) -> Vec<(&UnlockableContent, &UnlockedContentRecord)> {
        self.unlocked_content.iter()
            .filter_map(|(id, record)| {
                self.content.get(id).map(|content| (content, record))
            })
            .collect()
    }

    /// Get locked content
    pub fn get_locked_content(&self) -> Vec<&UnlockableContent> {
        self.content.iter()
            .filter(|(id, _)| !self.is_content_unlocked(id))
            .map(|(_, content)| content)
            .collect()
    }

    /// Get content by type
    pub fn get_content_by_type(&self, content_type: &ContentType) -> Vec<&UnlockableContent> {
        self.content.values()
            .filter(|content| &content.content_type == content_type)
            .collect()
    }

    /// Get content by rarity
    pub fn get_content_by_rarity(&self, rarity: &ContentRarity) -> Vec<&UnlockableContent> {
        self.content.values()
            .filter(|content| &content.rarity == rarity)
            .collect()
    }

    /// Get content with previews available
    pub fn get_previewable_content(&self) -> Vec<&UnlockableContent> {
        self.content.values()
            .filter(|content| content.preview_available)
            .collect()
    }

    /// Get content with hints
    pub fn get_content_with_hints(&self) -> Vec<&UnlockableContent> {
        self.content.values()
            .filter(|content| content.hint.is_some())
            .collect()
    }

    /// Update statistics
    fn update_statistics(&mut self) {
        let total_content = self.content.len();
        let unlocked_content = self.unlocked_content.len();
        let locked_content = total_content - unlocked_content;
        let unlock_percentage = if total_content > 0 {
            (unlocked_content as f32 / total_content as f32) * 100.0
        } else {
            0.0
        };

        // Count by type
        let mut type_unlocks = HashMap::new();
        for content in self.content.values() {
            let entry = type_unlocks.entry(content.content_type.clone()).or_insert((0, 0));
            entry.0 += 1;
            if self.is_content_unlocked(&content.id) {
                entry.1 += 1;
            }
        }

        // Count by rarity
        let mut rarity_unlocks = HashMap::new();
        for content in self.content.values() {
            let entry = rarity_unlocks.entry(content.rarity.clone()).or_insert((0, 0));
            entry.0 += 1;
            if self.is_content_unlocked(&content.id) {
                entry.1 += 1;
            }
        }

        // Recent unlocks (last 10)
        let mut recent_unlocks: Vec<(String, u64)> = self.unlocked_content.iter()
            .map(|(id, record)| (id.clone(), record.unlocked_at))
            .collect();
        recent_unlocks.sort_by(|a, b| b.1.cmp(&a.1));
        let recent_unlocks: Vec<String> = recent_unlocks.into_iter()
            .take(10)
            .map(|(id, _)| id)
            .collect();

        // Most accessed content
        let mut most_accessed: Vec<(String, u32)> = self.unlocked_content.iter()
            .map(|(id, record)| (id.clone(), record.access_count))
            .collect();
        most_accessed.sort_by(|a, b| b.1.cmp(&a.1));
        most_accessed.truncate(10);

        self.statistics = ContentUnlockStatistics {
            total_content,
            unlocked_content,
            locked_content,
            unlock_percentage,
            type_unlocks,
            rarity_unlocks,
            recent_unlocks,
            most_accessed,
        };
    }

    /// Get statistics
    pub fn get_statistics(&self) -> &ContentUnlockStatistics {
        &self.statistics
    }

    /// Export data for persistence
    pub fn export_data(&self) -> UnlockableContentSaveData {
        UnlockableContentSaveData {
            unlocked_content: self.unlocked_content.clone(),
        }
    }

    /// Import data from persistence
    pub fn import_data(&mut self, data: UnlockableContentSaveData) {
        self.unlocked_content = data.unlocked_content;
        self.update_statistics();
    }
}

/// Save data for unlockable content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnlockableContentSaveData {
    pub unlocked_content: HashMap<String, UnlockedContentRecord>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::progression::milestone_system::MilestoneSystem;
    use crate::achievements::AchievementSystem;

    #[test]
    fn test_unlockable_content_system_creation() {
        let system = UnlockableContentSystem::new();
        assert!(system.content.len() > 0);
        assert_eq!(system.unlocked_content.len(), 0);
    }

    #[test]
    fn test_content_unlocking() {
        let mut system = UnlockableContentSystem::new();
        
        // Initially, content should be locked
        assert!(!system.is_content_unlocked("combat_tutorial"));
        
        // Unlock content
        let unlocked = system.unlock_content("combat_tutorial", "test");
        assert!(unlocked);
        assert!(system.is_content_unlocked("combat_tutorial"));
        
        // Can't unlock again
        let unlocked_again = system.unlock_content("combat_tutorial", "test");
        assert!(!unlocked_again);
    }

    #[test]
    fn test_content_access_tracking() {
        let mut system = UnlockableContentSystem::new();
        
        // Unlock and access content
        system.unlock_content("combat_tutorial", "test");
        system.access_content("combat_tutorial");
        system.access_content("combat_tutorial");
        
        if let Some(record) = system.unlocked_content.get("combat_tutorial") {
            assert_eq!(record.access_count, 2);
            assert!(record.first_accessed.is_some());
        }
    }

    #[test]
    fn test_content_filtering() {
        let system = UnlockableContentSystem::new();
        
        // Test filtering by type
        let areas = system.get_content_by_type(&ContentType::Area);
        assert!(areas.len() > 0);
        
        for content in areas {
            assert_eq!(content.content_type, ContentType::Area);
        }
        
        // Test filtering by rarity
        let legendary = system.get_content_by_rarity(&ContentRarity::Legendary);
        assert!(legendary.len() > 0);
        
        for content in legendary {
            assert_eq!(content.rarity, ContentRarity::Legendary);
        }
    }

    #[test]
    fn test_unlock_conditions() {
        let mut content_system = UnlockableContentSystem::new();
        let milestone_system = MilestoneSystem::new();
        let achievement_system = AchievementSystem::new();
        
        // Check unlock conditions
        let newly_unlocked = content_system.check_unlock_conditions(
            &milestone_system,
            &achievement_system,
            1
        );
        
        // Should not unlock anything initially
        assert_eq!(newly_unlocked.len(), 0);
    }

    #[test]
    fn test_content_statistics() {
        let mut system = UnlockableContentSystem::new();
        let initial_stats = system.get_statistics();
        
        assert!(initial_stats.total_content > 0);
        assert_eq!(initial_stats.unlocked_content, 0);
        assert_eq!(initial_stats.unlock_percentage, 0.0);
        
        // Unlock some content
        system.unlock_content("combat_tutorial", "test");
        system.unlock_content("map_system", "test");
        
        let updated_stats = system.get_statistics();
        assert_eq!(updated_stats.unlocked_content, 2);
        assert!(updated_stats.unlock_percentage > 0.0);
    }

    #[test]
    fn test_save_and_load() {
        let mut system = UnlockableContentSystem::new();
        
        // Unlock some content
        system.unlock_content("combat_tutorial", "test");
        system.access_content("combat_tutorial");
        
        // Export data
        let save_data = system.export_data();
        assert_eq!(save_data.unlocked_content.len(), 1);
        
        // Create new system and import data
        let mut new_system = UnlockableContentSystem::new();
        new_system.import_data(save_data);
        
        // Verify data was imported correctly
        assert!(new_system.is_content_unlocked("combat_tutorial"));
        
        if let Some(record) = new_system.unlocked_content.get("combat_tutorial") {
            assert_eq!(record.access_count, 1);
        }
    }
}