use bevy::prelude::*;
use serde::{Serialize, Deserialize};
use std::collections::{HashMap, HashSet};
use rand::{Rng, thread_rng};
use crate::guild::guild_core::GuildResource;
use crate::items::Item;

/// Mission difficulty levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MissionDifficulty {
    Trivial,
    Easy,
    Medium,
    Hard,
    VeryHard,
    Extreme,
}

impl MissionDifficulty {
    /// Get the name of this difficulty level
    pub fn name(&self) -> &'static str {
        match self {
            MissionDifficulty::Trivial => "Trivial",
            MissionDifficulty::Easy => "Easy",
            MissionDifficulty::Medium => "Medium",
            MissionDifficulty::Hard => "Hard",
            MissionDifficulty::VeryHard => "Very Hard",
            MissionDifficulty::Extreme => "Extreme",
        }
    }
    
    /// Get the recommended level for this difficulty
    pub fn recommended_level(&self) -> u32 {
        match self {
            MissionDifficulty::Trivial => 1,
            MissionDifficulty::Easy => 3,
            MissionDifficulty::Medium => 5,
            MissionDifficulty::Hard => 8,
            MissionDifficulty::VeryHard => 12,
            MissionDifficulty::Extreme => 15,
        }
    }
    
    /// Get the base reward multiplier for this difficulty
    pub fn reward_multiplier(&self) -> f32 {
        match self {
            MissionDifficulty::Trivial => 0.5,
            MissionDifficulty::Easy => 1.0,
            MissionDifficulty::Medium => 1.5,
            MissionDifficulty::Hard => 2.0,
            MissionDifficulty::VeryHard => 3.0,
            MissionDifficulty::Extreme => 5.0,
        }
    }
    
    /// Get all difficulty levels
    pub fn all() -> Vec<MissionDifficulty> {
        vec![
            MissionDifficulty::Trivial,
            MissionDifficulty::Easy,
            MissionDifficulty::Medium,
            MissionDifficulty::Hard,
            MissionDifficulty::VeryHard,
            MissionDifficulty::Extreme,
        ]
    }
    
    /// Get a random difficulty level
    pub fn random() -> MissionDifficulty {
        let mut rng = thread_rng();
        let difficulties = MissionDifficulty::all();
        let weights = [20, 30, 25, 15, 7, 3]; // Probability distribution
        
        let total_weight: u32 = weights.iter().sum();
        let mut value = rng.gen_range(0..total_weight);
        
        for (i, &weight) in weights.iter().enumerate() {
            if value < weight {
                return difficulties[i];
            }
            value -= weight;
        }
        
        // Fallback
        MissionDifficulty::Medium
    }
}

/// Mission objective types
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MissionObjectiveType {
    KillEnemies { enemy_type: String, count: u32 },
    CollectItems { item_type: String, count: u32 },
    ExploreArea { area_name: String, percentage: u32 },
    DefeatBoss { boss_name: String },
    EscortNPC { npc_name: String },
    FindArtifact { artifact_name: String },
    SurviveWaves { wave_count: u32 },
    ReachLocation { location_name: String },
    ClearDungeon,
    Custom { description: String },
}

impl MissionObjectiveType {
    /// Get a description of this objective
    pub fn description(&self) -> String {
        match self {
            MissionObjectiveType::KillEnemies { enemy_type, count } => {
                format!("Kill {} {}", count, enemy_type)
            },
            MissionObjectiveType::CollectItems { item_type, count } => {
                format!("Collect {} {}", count, item_type)
            },
            MissionObjectiveType::ExploreArea { area_name, percentage } => {
                format!("Explore {}% of {}", percentage, area_name)
            },
            MissionObjectiveType::DefeatBoss { boss_name } => {
                format!("Defeat {}", boss_name)
            },
            MissionObjectiveType::EscortNPC { npc_name } => {
                format!("Escort {} to safety", npc_name)
            },
            MissionObjectiveType::FindArtifact { artifact_name } => {
                format!("Find the {}", artifact_name)
            },
            MissionObjectiveType::SurviveWaves { wave_count } => {
                format!("Survive {} waves of enemies", wave_count)
            },
            MissionObjectiveType::ReachLocation { location_name } => {
                format!("Reach {}", location_name)
            },
            MissionObjectiveType::ClearDungeon => {
                "Clear the entire dungeon".to_string()
            },
            MissionObjectiveType::Custom { description } => {
                description.clone()
            },
        }
    }
    
    /// Generate a random objective based on difficulty
    pub fn random(difficulty: MissionDifficulty) -> MissionObjectiveType {
        let mut rng = thread_rng();
        
        // Scale counts based on difficulty
        let difficulty_multiplier = match difficulty {
            MissionDifficulty::Trivial => 1,
            MissionDifficulty::Easy => 2,
            MissionDifficulty::Medium => 3,
            MissionDifficulty::Hard => 5,
            MissionDifficulty::VeryHard => 8,
            MissionDifficulty::Extreme => 12,
        };
        
        // Generate random objective
        match rng.gen_range(0..10) {
            0 => MissionObjectiveType::KillEnemies {
                enemy_type: ["Goblins", "Skeletons", "Rats", "Spiders", "Bandits"]
                    [rng.gen_range(0..5)].to_string(),
                count: difficulty_multiplier * rng.gen_range(2..5),
            },
            1 => MissionObjectiveType::CollectItems {
                item_type: ["Gems", "Herbs", "Scrolls", "Artifacts", "Supplies"]
                    [rng.gen_range(0..5)].to_string(),
                count: difficulty_multiplier * rng.gen_range(1..4),
            },
            2 => MissionObjectiveType::ExploreArea {
                area_name: ["Crypt", "Cave", "Forest", "Ruins", "Dungeon"]
                    [rng.gen_range(0..5)].to_string(),
                percentage: 50 + (difficulty_multiplier * 5),
            },
            3 => MissionObjectiveType::DefeatBoss {
                boss_name: ["Goblin King", "Necromancer", "Dragon", "Demon Lord", "Ancient Golem"]
                    [rng.gen_range(0..5)].to_string(),
            },
            4 => MissionObjectiveType::EscortNPC {
                npc_name: ["Merchant", "Scholar", "Noble", "Child", "Prisoner"]
                    [rng.gen_range(0..5)].to_string(),
            },
            5 => MissionObjectiveType::FindArtifact {
                artifact_name: ["Ancient Sword", "Magic Orb", "Lost Crown", "Sacred Relic", "Enchanted Gem"]
                    [rng.gen_range(0..5)].to_string(),
            },
            6 => MissionObjectiveType::SurviveWaves {
                wave_count: difficulty_multiplier * rng.gen_range(1..3),
            },
            7 => MissionObjectiveType::ReachLocation {
                location_name: ["Ancient Altar", "Hidden Chamber", "Mountain Peak", "Underground Lake", "Portal"]
                    [rng.gen_range(0..5)].to_string(),
            },
            8 => MissionObjectiveType::ClearDungeon,
            _ => MissionObjectiveType::Custom {
                description: ["Investigate strange noises", "Recover lost supplies", "Map the area", "Find missing scouts", "Disable traps"]
                    [rng.gen_range(0..5)].to_string(),
            },
        }
    }
}

/// Mission objective status
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MissionObjectiveStatus {
    NotStarted,
    InProgress { current: u32, total: u32 },
    Completed,
    Failed,
}

impl Default for MissionObjectiveStatus {
    fn default() -> Self {
        MissionObjectiveStatus::NotStarted
    }
}

/// Mission objective
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissionObjective {
    pub objective_type: MissionObjectiveType,
    pub status: MissionObjectiveStatus,
    pub location: Option<String>,
    pub required_items: Vec<String>,
    pub target_entities: HashSet<Entity>,
}

impl MissionObjective {
    /// Create a new mission objective
    pub fn new(objective_type: MissionObjectiveType) -> Self {
        let status = match &objective_type {
            MissionObjectiveType::KillEnemies { count, .. } => {
                MissionObjectiveStatus::InProgress { current: 0, total: *count }
            },
            MissionObjectiveType::CollectItems { count, .. } => {
                MissionObjectiveStatus::InProgress { current: 0, total: *count }
            },
            MissionObjectiveType::ExploreArea { percentage, .. } => {
                MissionObjectiveStatus::InProgress { current: 0, total: *percentage }
            },
            MissionObjectiveType::SurviveWaves { wave_count } => {
                MissionObjectiveStatus::InProgress { current: 0, total: *wave_count }
            },
            _ => MissionObjectiveStatus::NotStarted,
        };
        
        MissionObjective {
            objective_type,
            status,
            location: None,
            required_items: Vec::new(),
            target_entities: HashSet::new(),
        }
    }
    
    /// Update objective progress
    pub fn update_progress(&mut self, amount: u32) -> bool {
        match &mut self.status {
            MissionObjectiveStatus::InProgress { current, total } => {
                *current += amount;
                if *current >= *total {
                    self.status = MissionObjectiveStatus::Completed;
                    return true;
                }
            },
            _ => {}
        }
        false
    }
    
    /// Get progress percentage
    pub fn progress_percentage(&self) -> f32 {
        match &self.status {
            MissionObjectiveStatus::InProgress { current, total } => {
                if *total == 0 {
                    return 0.0;
                }
                *current as f32 / *total as f32 * 100.0
            },
            MissionObjectiveStatus::Completed => 100.0,
            _ => 0.0,
        }
    }
    
    /// Check if objective is completed
    pub fn is_completed(&self) -> bool {
        matches!(self.status, MissionObjectiveStatus::Completed)
    }
    
    /// Check if objective is failed
    pub fn is_failed(&self) -> bool {
        matches!(self.status, MissionObjectiveStatus::Failed)
    }
    
    /// Mark objective as completed
    pub fn complete(&mut self) {
        self.status = MissionObjectiveStatus::Completed;
    }
    
    /// Mark objective as failed
    pub fn fail(&mut self) {
        self.status = MissionObjectiveStatus::Failed;
    }
    
    /// Add a target entity
    pub fn add_target(&mut self, entity: Entity) {
        self.target_entities.insert(entity);
    }
    
    /// Remove a target entity
    pub fn remove_target(&mut self, entity: &Entity) {
        self.target_entities.remove(entity);
    }
    
    /// Add a required item
    pub fn add_required_item(&mut self, item_name: &str) {
        self.required_items.push(item_name.to_string());
    }
}

/// Mission status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MissionStatus {
    Available,
    Assigned,
    InProgress,
    Completed,
    Failed,
    Expired,
}

impl Default for MissionStatus {
    fn default() -> Self {
        MissionStatus::Available
    }
}

/// Mission reward types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MissionReward {
    Resources { resource_type: GuildResource, amount: u32 },
    Items { items: Vec<Item> },
    Experience { amount: u32 },
    Reputation { amount: i32 },
    UnlockFacility { facility_name: String },
    UnlockArea { area_name: String },
    Custom { description: String, value: u32 },
}

impl MissionReward {
    /// Get a description of this reward
    pub fn description(&self) -> String {
        match self {
            MissionReward::Resources { resource_type, amount } => {
                format!("{} {}", amount, resource_type.name())
            },
            MissionReward::Items { items } => {
                if items.len() == 1 {
                    format!("Item: {}", items[0].name)
                } else {
                    format!("{} items", items.len())
                }
            },
            MissionReward::Experience { amount } => {
                format!("{} experience", amount)
            },
            MissionReward::Reputation { amount } => {
                format!("{} reputation", amount)
            },
            MissionReward::UnlockFacility { facility_name } => {
                format!("Unlock: {}", facility_name)
            },
            MissionReward::UnlockArea { area_name } => {
                format!("Unlock area: {}", area_name)
            },
            MissionReward::Custom { description, value } => {
                format!("{} (value: {})", description, value)
            },
        }
    }
    
    /// Generate a random reward based on difficulty
    pub fn random(difficulty: MissionDifficulty) -> MissionReward {
        let mut rng = thread_rng();
        
        // Scale rewards based on difficulty
        let reward_multiplier = difficulty.reward_multiplier();
        
        // Generate random reward
        match rng.gen_range(0..7) {
            0 => MissionReward::Resources {
                resource_type: [GuildResource::Gold, GuildResource::Supplies, GuildResource::MagicEssence, 
                               GuildResource::Reputation, GuildResource::RareArtifacts]
                    [rng.gen_range(0..5)],
                amount: (50.0 * reward_multiplier) as u32 + rng.gen_range(10..30),
            },
            1 => {
                // In a real implementation, you would generate actual items here
                // For now, we'll just create a placeholder
                let item_count = (reward_multiplier / 2.0).ceil() as usize;
                let mut items = Vec::with_capacity(item_count);
                for i in 0..item_count {
                    items.push(Item::default()); // Placeholder
                }
                MissionReward::Items { items }
            },
            2 => MissionReward::Experience {
                amount: (100.0 * reward_multiplier) as u32 + rng.gen_range(20..50),
            },
            3 => MissionReward::Reputation {
                amount: (10.0 * reward_multiplier) as i32 + rng.gen_range(5..15),
            },
            4 => MissionReward::UnlockFacility {
                facility_name: ["Training Hall", "Library", "Workshop", "Magic Lab", "Forge"]
                    [rng.gen_range(0..5)].to_string(),
            },
            5 => MissionReward::UnlockArea {
                area_name: ["Ancient Ruins", "Forgotten Caves", "Dark Forest", "Abandoned Mine", "Haunted Castle"]
                    [rng.gen_range(0..5)].to_string(),
            },
            _ => MissionReward::Custom {
                description: ["Special Training", "Rare Knowledge", "Guild Favor", "Magical Enhancement", "Ancient Secret"]
                    [rng.gen_range(0..5)].to_string(),
                value: (30.0 * reward_multiplier) as u32 + rng.gen_range(10..20),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mission_difficulty() {
        let difficulties = MissionDifficulty::all();
        assert_eq!(difficulties.len(), 6);
        
        let easy = MissionDifficulty::Easy;
        assert_eq!(easy.name(), "Easy");
        assert!(easy.reward_multiplier() < MissionDifficulty::Hard.reward_multiplier());
    }

    #[test]
    fn test_mission_objective() {
        let objective_type = MissionObjectiveType::KillEnemies {
            enemy_type: "Goblins".to_string(),
            count: 5,
        };
        
        let mut objective = MissionObjective::new(objective_type);
        assert!(matches!(objective.status, MissionObjectiveStatus::InProgress { current: 0, total: 5 }));
        
        // Update progress
        let completed = objective.update_progress(3);
        assert!(!completed);
        assert!(matches!(objective.status, MissionObjectiveStatus::InProgress { current: 3, total: 5 }));
        
        // Complete objective
        let completed = objective.update_progress(2);
        assert!(completed);
        assert!(matches!(objective.status, MissionObjectiveStatus::Completed));
        assert!(objective.is_completed());
    }
}