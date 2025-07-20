use std::collections::{HashMap, HashSet};
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Serialize, Deserialize};
use crate::achievements::GameEvent;

/// Milestone types for different aspects of progression
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MilestoneType {
    Combat,
    Exploration,
    Character,
    Story,
    Collection,
    Social,
    Special,
}

/// Milestone importance levels
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum MilestoneImportance {
    Minor,
    Major,
    Critical,
    Legendary,
}

/// Milestone status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MilestoneStatus {
    Locked,
    Available,
    InProgress,
    Completed,
    Failed,
}

/// Milestone reward types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MilestoneReward {
    Experience(u32),
    Gold(u32),
    Item(String),
    Skill(String),
    Unlock(String),
    Title(String),
    Ability(String),
    Access(String), // Access to new areas/features
}

/// Milestone condition types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MilestoneCondition {
    KillEnemies(u32),
    ReachLevel(u32),
    ExploreRooms(u32),
    CollectGold(u32),
    CollectItems(u32),
    DefeatBoss(String),
    CompleteQuest(String),
    ReachDepth(u32),
    SurviveTime(u32), // seconds
    Custom(String, u32), // custom condition with target value
}

/// Milestone progress tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MilestoneProgress {
    pub current: u32,
    pub target: u32,
    pub started_at: u64,
    pub last_updated: u64,
    pub completion_rate: f32,
}

impl MilestoneProgress {
    pub fn new(target: u32) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        MilestoneProgress {
            current: 0,
            target,
            started_at: now,
            last_updated: now,
            completion_rate: 0.0,
        }
    }

    pub fn update(&mut self, value: u32) {
        self.current = value.min(self.target);
        self.completion_rate = if self.target > 0 {
            (self.current as f32 / self.target as f32) * 100.0
        } else {
            100.0
        };
        self.last_updated = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
    }

    pub fn increment(&mut self, amount: u32) {
        self.update(self.current + amount);
    }

    pub fn is_complete(&self) -> bool {
        self.current >= self.target
    }
}

/// Milestone definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Milestone {
    pub id: String,
    pub name: String,
    pub description: String,
    pub milestone_type: MilestoneType,
    pub importance: MilestoneImportance,
    pub condition: MilestoneCondition,
    pub prerequisites: Vec<String>,
    pub rewards: Vec<MilestoneReward>,
    pub unlocks: Vec<String>, // What this milestone unlocks
    pub hidden: bool,
    pub repeatable: bool,
    pub time_limited: Option<u64>, // Expiration timestamp
    pub icon: String,
}

impl Milestone {
    pub fn new(
        id: String,
        name: String,
        description: String,
        milestone_type: MilestoneType,
        importance: MilestoneImportance,
        condition: MilestoneCondition,
    ) -> Self {
        Milestone {
            id,
            name,
            description,
            milestone_type,
            importance,
            condition,
            prerequisites: Vec::new(),
            rewards: Vec::new(),
            unlocks: Vec::new(),
            hidden: false,
            repeatable: false,
            time_limited: None,
            icon: "🎯".to_string(),
        }
    }

    pub fn with_prerequisites(mut self, prerequisites: Vec<String>) -> Self {
        self.prerequisites = prerequisites;
        self
    }

    pub fn with_rewards(mut self, rewards: Vec<MilestoneReward>) -> Self {
        self.rewards = rewards;
        self
    }

    pub fn with_unlocks(mut self, unlocks: Vec<String>) -> Self {
        self.unlocks = unlocks;
        self
    }

    pub fn with_hidden(mut self, hidden: bool) -> Self {
        self.hidden = hidden;
        self
    }

    pub fn with_repeatable(mut self, repeatable: bool) -> Self {
        self.repeatable = repeatable;
        self
    }

    pub fn with_time_limit(mut self, expiration: u64) -> Self {
        self.time_limited = Some(expiration);
        self
    }

    pub fn with_icon(mut self, icon: String) -> Self {
        self.icon = icon;
        self
    }

    /// Get the target value for this milestone's condition
    pub fn get_target_value(&self) -> u32 {
        match &self.condition {
            MilestoneCondition::KillEnemies(target) => *target,
            MilestoneCondition::ReachLevel(target) => *target,
            MilestoneCondition::ExploreRooms(target) => *target,
            MilestoneCondition::CollectGold(target) => *target,
            MilestoneCondition::CollectItems(target) => *target,
            MilestoneCondition::DefeatBoss(_) => 1,
            MilestoneCondition::CompleteQuest(_) => 1,
            MilestoneCondition::ReachDepth(target) => *target,
            MilestoneCondition::SurviveTime(target) => *target,
            MilestoneCondition::Custom(_, target) => *target,
        }
    }

    /// Check if milestone is expired
    pub fn is_expired(&self) -> bool {
        if let Some(expiration) = self.time_limited {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            now > expiration
        } else {
            false
        }
    }
}

/// Completed milestone record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletedMilestone {
    pub milestone_id: String,
    pub completed_at: u64,
    pub progress: MilestoneProgress,
    pub rewards_claimed: bool,
    pub completion_time: u64, // Time taken to complete in seconds
}

impl CompletedMilestone {
    pub fn new(milestone_id: String, progress: MilestoneProgress) -> Self {
        let completed_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let completion_time = completed_at - progress.started_at;

        CompletedMilestone {
            milestone_id,
            completed_at,
            progress,
            rewards_claimed: false,
            completion_time,
        }
    }
}

/// Milestone statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MilestoneStatistics {
    pub total_milestones: usize,
    pub completed_milestones: usize,
    pub available_milestones: usize,
    pub locked_milestones: usize,
    pub completion_percentage: f32,
    pub type_completion: HashMap<MilestoneType, (usize, usize)>, // (total, completed)
    pub importance_completion: HashMap<MilestoneImportance, (usize, usize)>,
    pub recent_completions: Vec<String>,
    pub total_rewards_claimed: usize,
}

/// Milestone tracking system
pub struct MilestoneSystem {
    milestones: HashMap<String, Milestone>,
    milestone_progress: HashMap<String, MilestoneProgress>,
    completed_milestones: HashMap<String, CompletedMilestone>,
    milestone_status: HashMap<String, MilestoneStatus>,
    unlocked_content: HashSet<String>,
    statistics: MilestoneStatistics,
}

impl MilestoneSystem {
    pub fn new() -> Self {
        let mut system = MilestoneSystem {
            milestones: HashMap::new(),
            milestone_progress: HashMap::new(),
            completed_milestones: HashMap::new(),
            milestone_status: HashMap::new(),
            unlocked_content: HashSet::new(),
            statistics: MilestoneStatistics {
                total_milestones: 0,
                completed_milestones: 0,
                available_milestones: 0,
                locked_milestones: 0,
                completion_percentage: 0.0,
                type_completion: HashMap::new(),
                importance_completion: HashMap::new(),
                recent_completions: Vec::new(),
                total_rewards_claimed: 0,
            },
        };

        // Initialize with default milestones
        system.initialize_default_milestones();
        system.update_milestone_availability();
        system.update_statistics();

        system
    }

    /// Initialize default milestones
    fn initialize_default_milestones(&mut self) {
        let milestones = vec![
            // Combat milestones
            Milestone::new(
                "first_blood".to_string(),
                "First Blood".to_string(),
                "Defeat your first enemy".to_string(),
                MilestoneType::Combat,
                MilestoneImportance::Minor,
                MilestoneCondition::KillEnemies(1),
            ).with_rewards(vec![
                MilestoneReward::Experience(50),
                MilestoneReward::Unlock("combat_tutorial".to_string()),
            ]).with_icon("⚔️".to_string()),

            Milestone::new(
                "warrior_path".to_string(),
                "Path of the Warrior".to_string(),
                "Defeat 100 enemies".to_string(),
                MilestoneType::Combat,
                MilestoneImportance::Major,
                MilestoneCondition::KillEnemies(100),
            ).with_prerequisites(vec!["first_blood".to_string()])
            .with_rewards(vec![
                MilestoneReward::Experience(500),
                MilestoneReward::Title("Warrior".to_string()),
                MilestoneReward::Ability("combat_mastery".to_string()),
            ]).with_icon("🗡️".to_string()),

            // Character progression milestones
            Milestone::new(
                "growing_stronger".to_string(),
                "Growing Stronger".to_string(),
                "Reach level 5".to_string(),
                MilestoneType::Character,
                MilestoneImportance::Minor,
                MilestoneCondition::ReachLevel(5),
            ).with_rewards(vec![
                MilestoneReward::Skill("attribute_point".to_string()),
                MilestoneReward::Unlock("skill_tree".to_string()),
            ]).with_icon("📈".to_string()),

            Milestone::new(
                "veteran_adventurer".to_string(),
                "Veteran Adventurer".to_string(),
                "Reach level 20".to_string(),
                MilestoneType::Character,
                MilestoneImportance::Critical,
                MilestoneCondition::ReachLevel(20),
            ).with_prerequisites(vec!["growing_stronger".to_string()])
            .with_rewards(vec![
                MilestoneReward::Experience(1000),
                MilestoneReward::Title("Veteran".to_string()),
                MilestoneReward::Access("veteran_areas".to_string()),
            ]).with_icon("🎖️".to_string()),

            // Exploration milestones
            Milestone::new(
                "first_steps".to_string(),
                "First Steps".to_string(),
                "Explore 10 rooms".to_string(),
                MilestoneType::Exploration,
                MilestoneImportance::Minor,
                MilestoneCondition::ExploreRooms(10),
            ).with_rewards(vec![
                MilestoneReward::Experience(25),
                MilestoneReward::Unlock("map_system".to_string()),
            ]).with_icon("👣".to_string()),

            Milestone::new(
                "deep_explorer".to_string(),
                "Deep Explorer".to_string(),
                "Reach depth 10".to_string(),
                MilestoneType::Exploration,
                MilestoneImportance::Major,
                MilestoneCondition::ReachDepth(10),
            ).with_prerequisites(vec!["first_steps".to_string()])
            .with_rewards(vec![
                MilestoneReward::Experience(300),
                MilestoneReward::Item("depth_compass".to_string()),
                MilestoneReward::Access("deep_levels".to_string()),
            ]).with_icon("⬇️".to_string()),

            // Collection milestones
            Milestone::new(
                "treasure_seeker".to_string(),
                "Treasure Seeker".to_string(),
                "Collect 1000 gold".to_string(),
                MilestoneType::Collection,
                MilestoneImportance::Minor,
                MilestoneCondition::CollectGold(1000),
            ).with_rewards(vec![
                MilestoneReward::Gold(200),
                MilestoneReward::Unlock("merchant_discounts".to_string()),
            ]).with_icon("💰".to_string()),

            Milestone::new(
                "master_collector".to_string(),
                "Master Collector".to_string(),
                "Collect 100 different items".to_string(),
                MilestoneType::Collection,
                MilestoneImportance::Major,
                MilestoneCondition::CollectItems(100),
            ).with_rewards(vec![
                MilestoneReward::Experience(400),
                MilestoneReward::Title("Collector".to_string()),
                MilestoneReward::Access("collector_vault".to_string()),
            ]).with_icon("📦".to_string()),

            // Story milestones
            Milestone::new(
                "dragon_slayer".to_string(),
                "Dragon Slayer".to_string(),
                "Defeat the Ancient Dragon".to_string(),
                MilestoneType::Story,
                MilestoneImportance::Legendary,
                MilestoneCondition::DefeatBoss("ancient_dragon".to_string()),
            ).with_prerequisites(vec!["veteran_adventurer".to_string(), "deep_explorer".to_string()])
            .with_rewards(vec![
                MilestoneReward::Experience(2000),
                MilestoneReward::Title("Dragon Slayer".to_string()),
                MilestoneReward::Item("dragon_scale_armor".to_string()),
                MilestoneReward::Access("dragon_lair".to_string()),
            ]).with_icon("🐉".to_string()),

            // Special milestones
            Milestone::new(
                "survivor".to_string(),
                "Survivor".to_string(),
                "Survive for 2 hours".to_string(),
                MilestoneType::Special,
                MilestoneImportance::Major,
                MilestoneCondition::SurviveTime(7200), // 2 hours
            ).with_rewards(vec![
                MilestoneReward::Experience(300),
                MilestoneReward::Title("Survivor".to_string()),
                MilestoneReward::Ability("endurance".to_string()),
            ]).with_icon("⏰".to_string()),

            // Hidden milestone
            Milestone::new(
                "secret_keeper".to_string(),
                "Secret Keeper".to_string(),
                "Discover the hidden chamber".to_string(),
                MilestoneType::Special,
                MilestoneImportance::Critical,
                MilestoneCondition::Custom("secret_chamber".to_string(), 1),
            ).with_hidden(true)
            .with_rewards(vec![
                MilestoneReward::Experience(800),
                MilestoneReward::Item("ancient_artifact".to_string()),
                MilestoneReward::Access("secret_areas".to_string()),
            ]).with_icon("🔍".to_string()),
        ];

        for milestone in milestones {
            self.add_milestone(milestone);
        }
    }

    /// Add a milestone to the system
    pub fn add_milestone(&mut self, milestone: Milestone) {
        let id = milestone.id.clone();
        let target = milestone.get_target_value();

        // Initialize progress tracking
        self.milestone_progress.insert(id.clone(), MilestoneProgress::new(target));
        
        // Set initial status
        self.milestone_status.insert(id.clone(), MilestoneStatus::Locked);

        self.milestones.insert(id, milestone);
    }

    /// Update milestone availability based on prerequisites
    fn update_milestone_availability(&mut self) {
        let completed_ids: HashSet<String> = self.completed_milestones.keys().cloned().collect();
        
        for (id, milestone) in &self.milestones {
            if self.completed_milestones.contains_key(id) {
                self.milestone_status.insert(id.clone(), MilestoneStatus::Completed);
                continue;
            }

            if milestone.is_expired() {
                self.milestone_status.insert(id.clone(), MilestoneStatus::Failed);
                continue;
            }

            // Check prerequisites
            let prerequisites_met = milestone.prerequisites.iter()
                .all(|prereq| completed_ids.contains(prereq));

            if prerequisites_met {
                let current_status = self.milestone_status.get(id).unwrap_or(&MilestoneStatus::Locked);
                if *current_status == MilestoneStatus::Locked {
                    self.milestone_status.insert(id.clone(), MilestoneStatus::Available);
                }
            }
        }
    }

    /// Process game events to update milestone progress
    pub fn process_game_event(&mut self, event: &GameEvent) -> Vec<String> {
        let mut completed_milestones = Vec::new();

        for (id, milestone) in &self.milestones {
            if self.completed_milestones.contains_key(id) && !milestone.repeatable {
                continue;
            }

            let status = self.milestone_status.get(id).unwrap_or(&MilestoneStatus::Locked);
            if *status != MilestoneStatus::Available && *status != MilestoneStatus::InProgress {
                continue;
            }

            let should_update = match (&milestone.condition, event) {
                (MilestoneCondition::KillEnemies(_), GameEvent::EnemyKilled) => true,
                (MilestoneCondition::ReachLevel(target), GameEvent::LevelChanged(level)) => *level >= *target as i32,
                (MilestoneCondition::ExploreRooms(_), GameEvent::RoomVisited) => true,
                (MilestoneCondition::CollectGold(_), GameEvent::GoldCollected(_)) => true,
                (MilestoneCondition::CollectItems(_), GameEvent::ItemCollected) => true,
                (MilestoneCondition::DefeatBoss(boss), GameEvent::BossDefeated) => {
                    // In a real implementation, you'd check the specific boss name
                    true
                },
                (MilestoneCondition::SurviveTime(_), GameEvent::PlaytimeUpdate(_)) => true,
                (MilestoneCondition::Custom(event_name, _), _) => {
                    // Custom condition matching would be implemented here
                    false
                },
                _ => false,
            };

            if should_update {
                if let Some(progress) = self.milestone_progress.get_mut(id) {
                    // Update progress based on event
                    match (&milestone.condition, event) {
                        (MilestoneCondition::KillEnemies(_), GameEvent::EnemyKilled) => {
                            progress.increment(1);
                        },
                        (MilestoneCondition::ReachLevel(target), GameEvent::LevelChanged(level)) => {
                            progress.update(*level as u32);
                        },
                        (MilestoneCondition::ExploreRooms(_), GameEvent::RoomVisited) => {
                            progress.increment(1);
                        },
                        (MilestoneCondition::CollectGold(_), GameEvent::GoldCollected(amount)) => {
                            progress.increment(*amount);
                        },
                        (MilestoneCondition::CollectItems(_), GameEvent::ItemCollected) => {
                            progress.increment(1);
                        },
                        (MilestoneCondition::DefeatBoss(_), GameEvent::BossDefeated) => {
                            progress.update(1);
                        },
                        (MilestoneCondition::SurviveTime(_), GameEvent::PlaytimeUpdate(seconds)) => {
                            progress.update(*seconds);
                        },
                        _ => {},
                    }

                    // Update status
                    self.milestone_status.insert(id.clone(), MilestoneStatus::InProgress);

                    // Check if milestone is completed
                    if progress.is_complete() {
                        self.complete_milestone(id);
                        completed_milestones.push(id.clone());
                    }
                }
            }
        }

        if !completed_milestones.is_empty() {
            self.update_milestone_availability();
            self.update_statistics();
        }

        completed_milestones
    }

    /// Complete a milestone
    fn complete_milestone(&mut self, milestone_id: &str) {
        if let Some(progress) = self.milestone_progress.get(milestone_id) {
            let completed = CompletedMilestone::new(milestone_id.to_string(), progress.clone());
            self.completed_milestones.insert(milestone_id.to_string(), completed);
            self.milestone_status.insert(milestone_id.to_string(), MilestoneStatus::Completed);

            // Process unlocks
            if let Some(milestone) = self.milestones.get(milestone_id) {
                for unlock in &milestone.unlocks {
                    self.unlocked_content.insert(unlock.clone());
                }
            }
        }
    }

    /// Check if content is unlocked
    pub fn is_content_unlocked(&self, content_id: &str) -> bool {
        self.unlocked_content.contains(content_id)
    }

    /// Get milestone status
    pub fn get_milestone_status(&self, milestone_id: &str) -> MilestoneStatus {
        self.milestone_status.get(milestone_id)
            .cloned()
            .unwrap_or(MilestoneStatus::Locked)
    }

    /// Get milestone progress
    pub fn get_milestone_progress(&self, milestone_id: &str) -> Option<&MilestoneProgress> {
        self.milestone_progress.get(milestone_id)
    }

    /// Get all milestones
    pub fn get_milestones(&self, include_hidden: bool) -> Vec<&Milestone> {
        self.milestones.values()
            .filter(|m| include_hidden || !m.hidden)
            .collect()
    }

    /// Get milestones by type
    pub fn get_milestones_by_type(&self, milestone_type: &MilestoneType) -> Vec<&Milestone> {
        self.milestones.values()
            .filter(|m| &m.milestone_type == milestone_type)
            .collect()
    }

    /// Get available milestones
    pub fn get_available_milestones(&self) -> Vec<&Milestone> {
        self.milestones.iter()
            .filter(|(id, _)| {
                matches!(self.get_milestone_status(id), MilestoneStatus::Available | MilestoneStatus::InProgress)
            })
            .map(|(_, milestone)| milestone)
            .collect()
    }

    /// Get completed milestones
    pub fn get_completed_milestones(&self) -> Vec<(&Milestone, &CompletedMilestone)> {
        self.completed_milestones.iter()
            .filter_map(|(id, completed)| {
                self.milestones.get(id).map(|milestone| (milestone, completed))
            })
            .collect()
    }

    /// Claim rewards for a milestone
    pub fn claim_milestone_rewards(&mut self, milestone_id: &str) -> Vec<MilestoneReward> {
        if let Some(completed) = self.completed_milestones.get_mut(milestone_id) {
            if !completed.rewards_claimed {
                completed.rewards_claimed = true;
                if let Some(milestone) = self.milestones.get(milestone_id) {
                    self.update_statistics();
                    return milestone.rewards.clone();
                }
            }
        }
        Vec::new()
    }

    /// Update milestone statistics
    fn update_statistics(&mut self) {
        let total_milestones = self.milestones.len();
        let completed_milestones = self.completed_milestones.len();
        let available_milestones = self.milestone_status.values()
            .filter(|status| matches!(status, MilestoneStatus::Available | MilestoneStatus::InProgress))
            .count();
        let locked_milestones = self.milestone_status.values()
            .filter(|status| matches!(status, MilestoneStatus::Locked))
            .count();

        let completion_percentage = if total_milestones > 0 {
            (completed_milestones as f32 / total_milestones as f32) * 100.0
        } else {
            0.0
        };

        // Count by type
        let mut type_completion = HashMap::new();
        for milestone in self.milestones.values() {
            let entry = type_completion.entry(milestone.milestone_type.clone()).or_insert((0, 0));
            entry.0 += 1;
            if self.completed_milestones.contains_key(&milestone.id) {
                entry.1 += 1;
            }
        }

        // Count by importance
        let mut importance_completion = HashMap::new();
        for milestone in self.milestones.values() {
            let entry = importance_completion.entry(milestone.importance.clone()).or_insert((0, 0));
            entry.0 += 1;
            if self.completed_milestones.contains_key(&milestone.id) {
                entry.1 += 1;
            }
        }

        // Recent completions (last 10)
        let mut recent_completions: Vec<(String, u64)> = self.completed_milestones.iter()
            .map(|(id, completed)| (id.clone(), completed.completed_at))
            .collect();
        recent_completions.sort_by(|a, b| b.1.cmp(&a.1));
        let recent_completions: Vec<String> = recent_completions.into_iter()
            .take(10)
            .map(|(id, _)| id)
            .collect();

        // Count claimed rewards
        let total_rewards_claimed = self.completed_milestones.values()
            .filter(|completed| completed.rewards_claimed)
            .count();

        self.statistics = MilestoneStatistics {
            total_milestones,
            completed_milestones,
            available_milestones,
            locked_milestones,
            completion_percentage,
            type_completion,
            importance_completion,
            recent_completions,
            total_rewards_claimed,
        };
    }

    /// Get milestone statistics
    pub fn get_statistics(&self) -> &MilestoneStatistics {
        &self.statistics
    }

    /// Export milestone data for persistence
    pub fn export_data(&self) -> MilestoneSaveData {
        MilestoneSaveData {
            milestone_progress: self.milestone_progress.clone(),
            completed_milestones: self.completed_milestones.clone(),
            milestone_status: self.milestone_status.clone(),
            unlocked_content: self.unlocked_content.clone(),
        }
    }

    /// Import milestone data from persistence
    pub fn import_data(&mut self, data: MilestoneSaveData) {
        self.milestone_progress = data.milestone_progress;
        self.completed_milestones = data.completed_milestones;
        self.milestone_status = data.milestone_status;
        self.unlocked_content = data.unlocked_content;
        
        self.update_milestone_availability();
        self.update_statistics();
    }
}

/// Milestone save data for persistence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MilestoneSaveData {
    pub milestone_progress: HashMap<String, MilestoneProgress>,
    pub completed_milestones: HashMap<String, CompletedMilestone>,
    pub milestone_status: HashMap<String, MilestoneStatus>,
    pub unlocked_content: HashSet<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_milestone_system_creation() {
        let system = MilestoneSystem::new();
        assert!(system.milestones.len() > 0);
        assert_eq!(system.completed_milestones.len(), 0);
    }

    #[test]
    fn test_milestone_progress() {
        let mut system = MilestoneSystem::new();
        
        // Process enemy kill event
        let completed = system.process_game_event(&GameEvent::EnemyKilled);
        
        // Should complete "first_blood" milestone
        assert!(completed.contains(&"first_blood".to_string()));
        assert_eq!(system.get_milestone_status("first_blood"), MilestoneStatus::Completed);
    }

    #[test]
    fn test_milestone_prerequisites() {
        let mut system = MilestoneSystem::new();
        
        // Initially, warrior_path should be locked
        assert_eq!(system.get_milestone_status("warrior_path"), MilestoneStatus::Locked);
        
        // Complete first_blood
        system.process_game_event(&GameEvent::EnemyKilled);
        
        // Now warrior_path should be available
        assert_eq!(system.get_milestone_status("warrior_path"), MilestoneStatus::Available);
    }

    #[test]
    fn test_content_unlocking() {
        let mut system = MilestoneSystem::new();
        
        // Initially, combat_tutorial should not be unlocked
        assert!(!system.is_content_unlocked("combat_tutorial"));
        
        // Complete first_blood milestone
        system.process_game_event(&GameEvent::EnemyKilled);
        
        // Now combat_tutorial should be unlocked
        assert!(system.is_content_unlocked("combat_tutorial"));
    }

    #[test]
    fn test_milestone_rewards() {
        let mut system = MilestoneSystem::new();
        
        // Complete first_blood milestone
        system.process_game_event(&GameEvent::EnemyKilled);
        
        // Claim rewards
        let rewards = system.claim_milestone_rewards("first_blood");
        assert!(rewards.len() > 0);
        
        // Can't claim rewards twice
        let rewards2 = system.claim_milestone_rewards("first_blood");
        assert_eq!(rewards2.len(), 0);
    }

    #[test]
    fn test_milestone_statistics() {
        let mut system = MilestoneSystem::new();
        let initial_stats = system.get_statistics();
        
        assert!(initial_stats.total_milestones > 0);
        assert_eq!(initial_stats.completed_milestones, 0);
        
        // Complete a milestone
        system.process_game_event(&GameEvent::EnemyKilled);
        
        let updated_stats = system.get_statistics();
        assert_eq!(updated_stats.completed_milestones, 1);
        assert!(updated_stats.completion_percentage > 0.0);
    }

    #[test]
    fn test_milestone_filtering() {
        let system = MilestoneSystem::new();
        
        // Test filtering by type
        let combat_milestones = system.get_milestones_by_type(&MilestoneType::Combat);
        assert!(combat_milestones.len() > 0);
        
        for milestone in combat_milestones {
            assert_eq!(milestone.milestone_type, MilestoneType::Combat);
        }
        
        // Test available milestones
        let available = system.get_available_milestones();
        assert!(available.len() > 0);
    }

    #[test]
    fn test_save_and_load() {
        let mut system = MilestoneSystem::new();
        
        // Complete some milestones
        system.process_game_event(&GameEvent::EnemyKilled);
        system.process_game_event(&GameEvent::LevelChanged(5));
        
        // Export data
        let save_data = system.export_data();
        assert_eq!(save_data.completed_milestones.len(), 2);
        
        // Create new system and import data
        let mut new_system = MilestoneSystem::new();
        new_system.import_data(save_data);
        
        // Verify data was imported correctly
        assert_eq!(new_system.get_milestone_status("first_blood"), MilestoneStatus::Completed);
        assert_eq!(new_system.get_milestone_status("growing_stronger"), MilestoneStatus::Completed);
    }
}