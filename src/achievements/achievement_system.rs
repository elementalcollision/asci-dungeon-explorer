use std::collections::{HashMap, HashSet};
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Serialize, Deserialize};
use specs::{World, System, ReadStorage, WriteStorage, Entities, Join};
use crate::components::{Player, Name, Position, Health, Experience};
use crate::resources::GameLog;

/// Achievement types
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AchievementType {
    Combat,
    Exploration,
    Collection,
    Progression,
    Social,
    Special,
    Hidden,
}

/// Achievement rarity levels
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum AchievementRarity {
    Common,
    Uncommon,
    Rare,
    Epic,
    Legendary,
}

/// Achievement difficulty levels
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum AchievementDifficulty {
    Easy,
    Medium,
    Hard,
    Extreme,
}

/// Achievement progress tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AchievementProgress {
    pub current: u32,
    pub target: u32,
    pub started_at: u64,
    pub last_updated: u64,
}

impl AchievementProgress {
    pub fn new(target: u32) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        AchievementProgress {
            current: 0,
            target,
            started_at: now,
            last_updated: now,
        }
    }

    pub fn update(&mut self, value: u32) {
        self.current = value.min(self.target);
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

    pub fn progress_percentage(&self) -> f32 {
        if self.target == 0 {
            100.0
        } else {
            (self.current as f32 / self.target as f32) * 100.0
        }
    }
}

/// Achievement definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Achievement {
    pub id: String,
    pub name: String,
    pub description: String,
    pub achievement_type: AchievementType,
    pub rarity: AchievementRarity,
    pub difficulty: AchievementDifficulty,
    pub points: u32,
    pub hidden: bool,
    pub prerequisites: Vec<String>,
    pub rewards: Vec<AchievementReward>,
    pub progress_target: u32,
    pub icon: String,
    pub unlock_message: String,
}

impl Achievement {
    pub fn new(
        id: String,
        name: String,
        description: String,
        achievement_type: AchievementType,
        rarity: AchievementRarity,
        difficulty: AchievementDifficulty,
        points: u32,
    ) -> Self {
        Achievement {
            id: id.clone(),
            name,
            description,
            achievement_type,
            rarity,
            difficulty,
            points,
            hidden: false,
            prerequisites: Vec::new(),
            rewards: Vec::new(),
            progress_target: 1,
            icon: "🏆".to_string(),
            unlock_message: format!("Achievement unlocked: {}", id),
        }
    }

    pub fn with_hidden(mut self, hidden: bool) -> Self {
        self.hidden = hidden;
        self
    }

    pub fn with_prerequisites(mut self, prerequisites: Vec<String>) -> Self {
        self.prerequisites = prerequisites;
        self
    }

    pub fn with_rewards(mut self, rewards: Vec<AchievementReward>) -> Self {
        self.rewards = rewards;
        self
    }

    pub fn with_progress_target(mut self, target: u32) -> Self {
        self.progress_target = target;
        self
    }

    pub fn with_icon(mut self, icon: String) -> Self {
        self.icon = icon;
        self
    }

    pub fn with_unlock_message(mut self, message: String) -> Self {
        self.unlock_message = message;
        self
    }
}

/// Achievement rewards
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AchievementReward {
    Experience(u32),
    Gold(u32),
    Item(String),
    Title(String),
    Cosmetic(String),
    Unlock(String),
}

/// Unlocked achievement data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnlockedAchievement {
    pub achievement_id: String,
    pub unlocked_at: u64,
    pub progress: AchievementProgress,
    pub rewards_claimed: bool,
}

impl UnlockedAchievement {
    pub fn new(achievement_id: String, progress: AchievementProgress) -> Self {
        UnlockedAchievement {
            achievement_id,
            unlocked_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            progress,
            rewards_claimed: false,
        }
    }
}

/// Achievement statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AchievementStatistics {
    pub total_achievements: usize,
    pub unlocked_achievements: usize,
    pub total_points: u32,
    pub earned_points: u32,
    pub completion_percentage: f32,
    pub rarity_counts: HashMap<AchievementRarity, (usize, usize)>, // (total, unlocked)
    pub type_counts: HashMap<AchievementType, (usize, usize)>, // (total, unlocked)
    pub recent_unlocks: Vec<String>,
}

/// Achievement system for tracking and managing player achievements
pub struct AchievementSystem {
    achievements: HashMap<String, Achievement>,
    unlocked_achievements: HashMap<String, UnlockedAchievement>,
    achievement_progress: HashMap<String, AchievementProgress>,
    pending_notifications: Vec<AchievementNotification>,
    statistics: AchievementStatistics,
}

impl AchievementSystem {
    pub fn new() -> Self {
        let mut system = AchievementSystem {
            achievements: HashMap::new(),
            unlocked_achievements: HashMap::new(),
            achievement_progress: HashMap::new(),
            pending_notifications: Vec::new(),
            statistics: AchievementStatistics {
                total_achievements: 0,
                unlocked_achievements: 0,
                total_points: 0,
                earned_points: 0,
                completion_percentage: 0.0,
                rarity_counts: HashMap::new(),
                type_counts: HashMap::new(),
                recent_unlocks: Vec::new(),
            },
        };

        // Initialize with default achievements
        system.initialize_default_achievements();
        system.update_statistics();

        system
    }

    /// Initialize default achievements
    fn initialize_default_achievements(&mut self) {
        let achievements = vec![
            // Combat achievements
            Achievement::new(
                "first_kill".to_string(),
                "First Blood".to_string(),
                "Defeat your first enemy".to_string(),
                AchievementType::Combat,
                AchievementRarity::Common,
                AchievementDifficulty::Easy,
                10,
            ).with_icon("⚔️".to_string()),

            Achievement::new(
                "kill_100_enemies".to_string(),
                "Slayer".to_string(),
                "Defeat 100 enemies".to_string(),
                AchievementType::Combat,
                AchievementRarity::Uncommon,
                AchievementDifficulty::Medium,
                50,
            ).with_progress_target(100)
            .with_icon("💀".to_string()),

            Achievement::new(
                "boss_slayer".to_string(),
                "Boss Slayer".to_string(),
                "Defeat your first boss".to_string(),
                AchievementType::Combat,
                AchievementRarity::Rare,
                AchievementDifficulty::Hard,
                100,
            ).with_icon("👑".to_string()),

            // Exploration achievements
            Achievement::new(
                "first_steps".to_string(),
                "First Steps".to_string(),
                "Take your first steps in the dungeon".to_string(),
                AchievementType::Exploration,
                AchievementRarity::Common,
                AchievementDifficulty::Easy,
                5,
            ).with_icon("👣".to_string()),

            Achievement::new(
                "explorer".to_string(),
                "Explorer".to_string(),
                "Visit 50 different rooms".to_string(),
                AchievementType::Exploration,
                AchievementRarity::Uncommon,
                AchievementDifficulty::Medium,
                30,
            ).with_progress_target(50)
            .with_icon("🗺️".to_string()),

            Achievement::new(
                "deep_delver".to_string(),
                "Deep Delver".to_string(),
                "Reach dungeon level 10".to_string(),
                AchievementType::Exploration,
                AchievementRarity::Rare,
                AchievementDifficulty::Hard,
                75,
            ).with_progress_target(10)
            .with_icon("⬇️".to_string()),

            // Progression achievements
            Achievement::new(
                "level_up".to_string(),
                "Growing Stronger".to_string(),
                "Reach level 2".to_string(),
                AchievementType::Progression,
                AchievementRarity::Common,
                AchievementDifficulty::Easy,
                15,
            ).with_progress_target(2)
            .with_icon("📈".to_string()),

            Achievement::new(
                "veteran".to_string(),
                "Veteran Adventurer".to_string(),
                "Reach level 20".to_string(),
                AchievementType::Progression,
                AchievementRarity::Epic,
                AchievementDifficulty::Hard,
                200,
            ).with_progress_target(20)
            .with_icon("🎖️".to_string()),

            // Collection achievements
            Achievement::new(
                "treasure_hunter".to_string(),
                "Treasure Hunter".to_string(),
                "Collect 100 gold".to_string(),
                AchievementType::Collection,
                AchievementRarity::Common,
                AchievementDifficulty::Easy,
                20,
            ).with_progress_target(100)
            .with_icon("💰".to_string()),

            Achievement::new(
                "hoarder".to_string(),
                "Hoarder".to_string(),
                "Collect 50 different items".to_string(),
                AchievementType::Collection,
                AchievementRarity::Rare,
                AchievementDifficulty::Medium,
                80,
            ).with_progress_target(50)
            .with_icon("📦".to_string()),

            // Special achievements
            Achievement::new(
                "survivor".to_string(),
                "Survivor".to_string(),
                "Survive for 1 hour of gameplay".to_string(),
                AchievementType::Special,
                AchievementRarity::Uncommon,
                AchievementDifficulty::Medium,
                40,
            ).with_progress_target(3600) // 1 hour in seconds
            .with_icon("⏰".to_string()),

            Achievement::new(
                "perfectionist".to_string(),
                "Perfectionist".to_string(),
                "Complete a dungeon level without taking damage".to_string(),
                AchievementType::Special,
                AchievementRarity::Epic,
                AchievementDifficulty::Extreme,
                150,
            ).with_icon("✨".to_string()),

            // Hidden achievements
            Achievement::new(
                "secret_room".to_string(),
                "Secret Keeper".to_string(),
                "Find a secret room".to_string(),
                AchievementType::Hidden,
                AchievementRarity::Rare,
                AchievementDifficulty::Medium,
                60,
            ).with_hidden(true)
            .with_icon("🔍".to_string()),

            Achievement::new(
                "easter_egg".to_string(),
                "Easter Egg Hunter".to_string(),
                "Find the developer's easter egg".to_string(),
                AchievementType::Hidden,
                AchievementRarity::Legendary,
                AchievementDifficulty::Extreme,
                500,
            ).with_hidden(true)
            .with_icon("🥚".to_string()),
        ];

        for achievement in achievements {
            self.add_achievement(achievement);
        }
    }

    /// Add an achievement to the system
    pub fn add_achievement(&mut self, achievement: Achievement) {
        let id = achievement.id.clone();
        
        // Initialize progress tracking
        if achievement.progress_target > 1 {
            self.achievement_progress.insert(
                id.clone(),
                AchievementProgress::new(achievement.progress_target),
            );
        }

        self.achievements.insert(id, achievement);
        self.update_statistics();
    }

    /// Check if an achievement is unlocked
    pub fn is_unlocked(&self, achievement_id: &str) -> bool {
        self.unlocked_achievements.contains_key(achievement_id)
    }

    /// Get achievement progress
    pub fn get_progress(&self, achievement_id: &str) -> Option<&AchievementProgress> {
        self.achievement_progress.get(achievement_id)
    }

    /// Update achievement progress
    pub fn update_progress(&mut self, achievement_id: &str, value: u32) -> bool {
        if self.is_unlocked(achievement_id) {
            return false;
        }

        if let Some(progress) = self.achievement_progress.get_mut(achievement_id) {
            progress.update(value);
            
            if progress.is_complete() {
                self.unlock_achievement(achievement_id);
                return true;
            }
        } else if let Some(achievement) = self.achievements.get(achievement_id) {
            // Single-step achievement
            if value >= achievement.progress_target {
                self.unlock_achievement(achievement_id);
                return true;
            }
        }

        false
    }

    /// Increment achievement progress
    pub fn increment_progress(&mut self, achievement_id: &str, amount: u32) -> bool {
        if self.is_unlocked(achievement_id) {
            return false;
        }

        if let Some(progress) = self.achievement_progress.get_mut(achievement_id) {
            progress.increment(amount);
            
            if progress.is_complete() {
                self.unlock_achievement(achievement_id);
                return true;
            }
        } else if let Some(_achievement) = self.achievements.get(achievement_id) {
            // Single-step achievement - treat as update
            return self.update_progress(achievement_id, amount);
        }

        false
    }

    /// Unlock an achievement
    fn unlock_achievement(&mut self, achievement_id: &str) {
        if self.is_unlocked(achievement_id) {
            return;
        }

        if let Some(achievement) = self.achievements.get(achievement_id) {
            // Check prerequisites
            for prereq in &achievement.prerequisites {
                if !self.is_unlocked(prereq) {
                    return; // Prerequisites not met
                }
            }

            // Get or create progress
            let progress = self.achievement_progress
                .get(achievement_id)
                .cloned()
                .unwrap_or_else(|| {
                    let mut p = AchievementProgress::new(achievement.progress_target);
                    p.current = achievement.progress_target;
                    p
                });

            // Create unlocked achievement
            let unlocked = UnlockedAchievement::new(achievement_id.to_string(), progress);
            
            // Add notification
            let notification = AchievementNotification {
                achievement_id: achievement_id.to_string(),
                achievement_name: achievement.name.clone(),
                achievement_icon: achievement.icon.clone(),
                message: achievement.unlock_message.clone(),
                points: achievement.points,
                rarity: achievement.rarity.clone(),
                timestamp: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
            };

            self.unlocked_achievements.insert(achievement_id.to_string(), unlocked);
            self.pending_notifications.push(notification);
            
            // Update statistics
            self.update_statistics();
        }
    }

    /// Get pending notifications
    pub fn get_pending_notifications(&mut self) -> Vec<AchievementNotification> {
        std::mem::take(&mut self.pending_notifications)
    }

    /// Get all achievements (visible ones only unless show_hidden is true)
    pub fn get_achievements(&self, show_hidden: bool) -> Vec<&Achievement> {
        self.achievements
            .values()
            .filter(|a| show_hidden || !a.hidden)
            .collect()
    }

    /// Get unlocked achievements
    pub fn get_unlocked_achievements(&self) -> Vec<(&Achievement, &UnlockedAchievement)> {
        self.unlocked_achievements
            .iter()
            .filter_map(|(id, unlocked)| {
                self.achievements.get(id).map(|achievement| (achievement, unlocked))
            })
            .collect()
    }

    /// Get achievements by type
    pub fn get_achievements_by_type(&self, achievement_type: &AchievementType) -> Vec<&Achievement> {
        self.achievements
            .values()
            .filter(|a| &a.achievement_type == achievement_type)
            .collect()
    }

    /// Get achievements by rarity
    pub fn get_achievements_by_rarity(&self, rarity: &AchievementRarity) -> Vec<&Achievement> {
        self.achievements
            .values()
            .filter(|a| &a.rarity == rarity)
            .collect()
    }

    /// Get achievement statistics
    pub fn get_statistics(&self) -> &AchievementStatistics {
        &self.statistics
    }

    /// Update achievement statistics
    fn update_statistics(&mut self) {
        let total_achievements = self.achievements.len();
        let unlocked_achievements = self.unlocked_achievements.len();
        let total_points: u32 = self.achievements.values().map(|a| a.points).sum();
        let earned_points: u32 = self.unlocked_achievements
            .keys()
            .filter_map(|id| self.achievements.get(id))
            .map(|a| a.points)
            .sum();

        let completion_percentage = if total_achievements > 0 {
            (unlocked_achievements as f32 / total_achievements as f32) * 100.0
        } else {
            0.0
        };

        // Count by rarity
        let mut rarity_counts = HashMap::new();
        for achievement in self.achievements.values() {
            let entry = rarity_counts.entry(achievement.rarity.clone()).or_insert((0, 0));
            entry.0 += 1;
            if self.is_unlocked(&achievement.id) {
                entry.1 += 1;
            }
        }

        // Count by type
        let mut type_counts = HashMap::new();
        for achievement in self.achievements.values() {
            let entry = type_counts.entry(achievement.achievement_type.clone()).or_insert((0, 0));
            entry.0 += 1;
            if self.is_unlocked(&achievement.id) {
                entry.1 += 1;
            }
        }

        // Recent unlocks (last 10)
        let mut recent_unlocks: Vec<(String, u64)> = self.unlocked_achievements
            .iter()
            .map(|(id, unlocked)| (id.clone(), unlocked.unlocked_at))
            .collect();
        recent_unlocks.sort_by(|a, b| b.1.cmp(&a.1));
        let recent_unlocks: Vec<String> = recent_unlocks
            .into_iter()
            .take(10)
            .map(|(id, _)| id)
            .collect();

        self.statistics = AchievementStatistics {
            total_achievements,
            unlocked_achievements,
            total_points,
            earned_points,
            completion_percentage,
            rarity_counts,
            type_counts,
            recent_unlocks,
        };
    }

    /// Process game events to trigger achievements
    pub fn process_game_event(&mut self, event: &GameEvent) {
        match event {
            GameEvent::EnemyKilled => {
                self.increment_progress("first_kill", 1);
                self.increment_progress("kill_100_enemies", 1);
            },
            GameEvent::BossDefeated => {
                self.increment_progress("boss_slayer", 1);
            },
            GameEvent::PlayerMoved => {
                self.increment_progress("first_steps", 1);
            },
            GameEvent::RoomVisited => {
                self.increment_progress("explorer", 1);
            },
            GameEvent::LevelChanged(level) => {
                self.update_progress("deep_delver", *level as u32);
                self.update_progress("level_up", *level as u32);
                self.update_progress("veteran", *level as u32);
            },
            GameEvent::GoldCollected(amount) => {
                self.increment_progress("treasure_hunter", *amount);
            },
            GameEvent::ItemCollected => {
                self.increment_progress("hoarder", 1);
            },
            GameEvent::PlaytimeUpdate(seconds) => {
                self.update_progress("survivor", *seconds);
            },
            GameEvent::SecretRoomFound => {
                self.increment_progress("secret_room", 1);
            },
            GameEvent::EasterEggFound => {
                self.increment_progress("easter_egg", 1);
            },
            GameEvent::PerfectLevel => {
                self.increment_progress("perfectionist", 1);
            },
        }
    }

    /// Claim rewards for an achievement
    pub fn claim_rewards(&mut self, achievement_id: &str) -> Vec<AchievementReward> {
        if let Some(unlocked) = self.unlocked_achievements.get_mut(achievement_id) {
            if !unlocked.rewards_claimed {
                unlocked.rewards_claimed = true;
                if let Some(achievement) = self.achievements.get(achievement_id) {
                    return achievement.rewards.clone();
                }
            }
        }
        Vec::new()
    }

    /// Export achievement data for persistence
    pub fn export_data(&self) -> AchievementSaveData {
        AchievementSaveData {
            unlocked_achievements: self.unlocked_achievements.clone(),
            achievement_progress: self.achievement_progress.clone(),
        }
    }

    /// Import achievement data from persistence
    pub fn import_data(&mut self, data: AchievementSaveData) {
        self.unlocked_achievements = data.unlocked_achievements;
        self.achievement_progress = data.achievement_progress;
        self.update_statistics();
    }
}

/// Achievement notification
#[derive(Debug, Clone)]
pub struct AchievementNotification {
    pub achievement_id: String,
    pub achievement_name: String,
    pub achievement_icon: String,
    pub message: String,
    pub points: u32,
    pub rarity: AchievementRarity,
    pub timestamp: u64,
}

/// Game events that can trigger achievements
#[derive(Debug, Clone)]
pub enum GameEvent {
    EnemyKilled,
    BossDefeated,
    PlayerMoved,
    RoomVisited,
    LevelChanged(i32),
    GoldCollected(u32),
    ItemCollected,
    PlaytimeUpdate(u32),
    SecretRoomFound,
    EasterEggFound,
    PerfectLevel,
}

/// Achievement save data for persistence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AchievementSaveData {
    pub unlocked_achievements: HashMap<String, UnlockedAchievement>,
    pub achievement_progress: HashMap<String, AchievementProgress>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_achievement_system_creation() {
        let system = AchievementSystem::new();
        assert!(system.achievements.len() > 0);
        assert_eq!(system.unlocked_achievements.len(), 0);
    }

    #[test]
    fn test_achievement_progress() {
        let mut system = AchievementSystem::new();
        
        // Test single-step achievement
        assert!(!system.is_unlocked("first_kill"));
        let unlocked = system.increment_progress("first_kill", 1);
        assert!(unlocked);
        assert!(system.is_unlocked("first_kill"));
    }

    #[test]
    fn test_multi_step_achievement() {
        let mut system = AchievementSystem::new();
        
        // Test multi-step achievement
        assert!(!system.is_unlocked("kill_100_enemies"));
        
        // Increment progress
        for i in 1..=99 {
            let unlocked = system.increment_progress("kill_100_enemies", 1);
            assert!(!unlocked);
            
            if let Some(progress) = system.get_progress("kill_100_enemies") {
                assert_eq!(progress.current, i);
                assert!(!progress.is_complete());
            }
        }
        
        // Final increment should unlock
        let unlocked = system.increment_progress("kill_100_enemies", 1);
        assert!(unlocked);
        assert!(system.is_unlocked("kill_100_enemies"));
    }

    #[test]
    fn test_achievement_notifications() {
        let mut system = AchievementSystem::new();
        
        // Unlock an achievement
        system.increment_progress("first_kill", 1);
        
        // Check for notifications
        let notifications = system.get_pending_notifications();
        assert_eq!(notifications.len(), 1);
        assert_eq!(notifications[0].achievement_id, "first_kill");
        
        // Notifications should be consumed
        let notifications = system.get_pending_notifications();
        assert_eq!(notifications.len(), 0);
    }

    #[test]
    fn test_achievement_statistics() {
        let mut system = AchievementSystem::new();
        let initial_stats = system.get_statistics();
        
        assert!(initial_stats.total_achievements > 0);
        assert_eq!(initial_stats.unlocked_achievements, 0);
        assert_eq!(initial_stats.completion_percentage, 0.0);
        
        // Unlock an achievement
        system.increment_progress("first_kill", 1);
        
        let updated_stats = system.get_statistics();
        assert_eq!(updated_stats.unlocked_achievements, 1);
        assert!(updated_stats.completion_percentage > 0.0);
        assert!(updated_stats.earned_points > 0);
    }

    #[test]
    fn test_game_event_processing() {
        let mut system = AchievementSystem::new();
        
        // Process enemy kill event
        system.process_game_event(&GameEvent::EnemyKilled);
        assert!(system.is_unlocked("first_kill"));
        
        // Process level change event
        system.process_game_event(&GameEvent::LevelChanged(2));
        assert!(system.is_unlocked("level_up"));
    }

    #[test]
    fn test_achievement_rewards() {
        let mut system = AchievementSystem::new();
        
        // Create achievement with rewards
        let achievement = Achievement::new(
            "test_reward".to_string(),
            "Test Reward".to_string(),
            "Test achievement with rewards".to_string(),
            AchievementType::Special,
            AchievementRarity::Common,
            AchievementDifficulty::Easy,
            10,
        ).with_rewards(vec![
            AchievementReward::Experience(100),
            AchievementReward::Gold(50),
        ]);
        
        system.add_achievement(achievement);
        
        // Unlock achievement
        system.increment_progress("test_reward", 1);
        assert!(system.is_unlocked("test_reward"));
        
        // Claim rewards
        let rewards = system.claim_rewards("test_reward");
        assert_eq!(rewards.len(), 2);
        
        // Can't claim rewards twice
        let rewards = system.claim_rewards("test_reward");
        assert_eq!(rewards.len(), 0);
    }

    #[test]
    fn test_hidden_achievements() {
        let system = AchievementSystem::new();
        
        // Get visible achievements
        let visible = system.get_achievements(false);
        let all = system.get_achievements(true);
        
        assert!(all.len() > visible.len());
        
        // Check that hidden achievements are not in visible list
        let hidden_count = all.iter().filter(|a| a.hidden).count();
        assert!(hidden_count > 0);
        assert_eq!(visible.len() + hidden_count, all.len());
    }

    #[test]
    fn test_achievement_filtering() {
        let system = AchievementSystem::new();
        
        // Test filtering by type
        let combat_achievements = system.get_achievements_by_type(&AchievementType::Combat);
        assert!(combat_achievements.len() > 0);
        
        for achievement in combat_achievements {
            assert_eq!(achievement.achievement_type, AchievementType::Combat);
        }
        
        // Test filtering by rarity
        let common_achievements = system.get_achievements_by_rarity(&AchievementRarity::Common);
        assert!(common_achievements.len() > 0);
        
        for achievement in common_achievements {
            assert_eq!(achievement.rarity, AchievementRarity::Common);
        }
    }

    #[test]
    fn test_save_and_load() {
        let mut system = AchievementSystem::new();
        
        // Unlock some achievements
        system.increment_progress("first_kill", 1);
        system.increment_progress("kill_100_enemies", 50);
        
        // Export data
        let save_data = system.export_data();
        assert_eq!(save_data.unlocked_achievements.len(), 1);
        assert!(save_data.achievement_progress.contains_key("kill_100_enemies"));
        
        // Create new system and import data
        let mut new_system = AchievementSystem::new();
        new_system.import_data(save_data);
        
        // Verify data was imported correctly
        assert!(new_system.is_unlocked("first_kill"));
        assert!(!new_system.is_unlocked("kill_100_enemies"));
        
        if let Some(progress) = new_system.get_progress("kill_100_enemies") {
            assert_eq!(progress.current, 50);
        }
    }
}