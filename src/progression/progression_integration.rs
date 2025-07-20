use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use serde::{Serialize, Deserialize};
use crate::{
    game_state::GameState,
    achievements::{AchievementSystem, GameEvent},
    progression::{
        MilestoneSystem, UnlockableContentSystem, WorldChangesSystem, PlayerHistorySystem,
        MilestoneType, ContentType, WorldChangeType, HistoryEventType, EventImportance,
        WorldChange, PersistenceLevel, ChangeScope,
    },
};

/// Comprehensive progression tracking integration
pub struct ProgressionIntegration {
    milestone_system: MilestoneSystem,
    unlockable_content_system: UnlockableContentSystem,
    world_changes_system: WorldChangesSystem,
    player_history_system: PlayerHistorySystem,
    current_player_stats: HashMap<String, i32>,
    current_session_id: Option<String>,
    enabled: bool,
}

impl ProgressionIntegration {
    /// Create new progression integration
    pub fn new() -> Self {
        ProgressionIntegration {
            milestone_system: MilestoneSystem::new(),
            unlockable_content_system: UnlockableContentSystem::new(),
            world_changes_system: WorldChangesSystem::new(),
            player_history_system: PlayerHistorySystem::new(1000),
            current_player_stats: HashMap::new(),
            current_session_id: None,
            enabled: true,
        }
    }

    /// Start a new game session
    pub fn start_session(&mut self, starting_stats: HashMap<String, i32>) -> String {
        self.current_player_stats = starting_stats.clone();
        let session_id = self.player_history_system.start_session(starting_stats);
        self.current_session_id = Some(session_id.clone());
        
        // Clear temporary world changes
        self.world_changes_system.clear_temporary_changes();
        
        session_id
    }

    /// End the current session
    pub fn end_session(&mut self) {
        if self.current_session_id.is_some() {
            self.player_history_system.end_session(self.current_player_stats.clone());
            self.world_changes_system.clear_session_changes();
            self.current_session_id = None;
        }
    }

    /// Update progression systems
    pub fn update(&mut self, game_state: &GameState, achievement_system: &AchievementSystem) {
        if !self.enabled {
            return;
        }

        // Update player level for unlock conditions
        let player_level = self.current_player_stats.get("level").unwrap_or(&1);

        // Check unlock conditions
        let newly_unlocked = self.unlockable_content_system.check_unlock_conditions(
            &self.milestone_system,
            achievement_system,
            *player_level as u32,
        );

        // Log newly unlocked content
        for content_id in newly_unlocked {
            if let Some(content) = self.unlockable_content_system.get_all_content()
                .iter()
                .find(|c| c.id == content_id) {
                
                self.player_history_system.add_event(
                    crate::progression::HistoryEvent::new(
                        format!("content_unlocked_{}", content_id),
                        HistoryEventType::System,
                        EventImportance::Important,
                        format!("Unlocked: {}", content.name),
                        format!("Unlocked new content: {}", content.description),
                    ).with_metadata("content_id".to_string(), content_id.clone())
                    .with_metadata("content_type".to_string(), format!("{:?}", content.content_type))
                    .with_tags(vec!["unlock".to_string(), "content".to_string()])
                );
            }
        }

        // Clean up expired world changes
        self.world_changes_system.cleanup_changes();
    }

    /// Process game events for progression tracking
    pub fn process_game_event(&mut self, event: &GameEvent, location: Option<String>) {
        if !self.enabled {
            return;
        }

        let stats_before = self.current_player_stats.clone();

        // Process milestone system
        let completed_milestones = self.milestone_system.process_game_event(event);
        
        // Log completed milestones
        for milestone_id in completed_milestones {
            if let Some(milestone) = self.milestone_system.get_milestones(true)
                .iter()
                .find(|m| m.id == milestone_id) {
                
                self.player_history_system.log_milestone(&milestone.name, &milestone_id);
            }
        }

        // Log specific events in history
        match event {
            GameEvent::EnemyKilled => {
                if let Some(loc) = &location {
                    self.player_history_system.log_combat_victory(
                        "Enemy", // In real implementation, would have enemy name
                        loc,
                        stats_before,
                        self.current_player_stats.clone(),
                    );
                }
            },
            GameEvent::LevelChanged(new_level) => {
                self.current_player_stats.insert("level".to_string(), *new_level);
                if let Some(loc) = &location {
                    self.player_history_system.log_level_up(
                        *new_level as u32,
                        loc,
                        stats_before,
                        self.current_player_stats.clone(),
                    );
                }
            },
            GameEvent::ItemCollected => {
                if let Some(loc) = &location {
                    self.player_history_system.log_item_found("Item", loc); // In real implementation, would have item name
                }
            },
            GameEvent::GoldCollected(amount) => {
                let current_gold = self.current_player_stats.get("gold").unwrap_or(&0);
                self.current_player_stats.insert("gold".to_string(), current_gold + *amount as i32);
            },
            _ => {},
        }
    }

    /// Apply a world change
    pub fn apply_world_change(
        &mut self,
        change_id: String,
        change_type: WorldChangeType,
        scope: ChangeScope,
        persistence: PersistenceLevel,
        description: String,
        cause: String,
    ) -> bool {
        let change = WorldChange::new(change_id, change_type, scope, persistence, description);
        let applied = self.world_changes_system.apply_change(change, cause.clone());
        
        if applied {
            // Log the world change
            self.player_history_system.add_event(
                crate::progression::HistoryEvent::new(
                    format!("world_change_{}", std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs()),
                    HistoryEventType::System,
                    EventImportance::Minor,
                    "World Changed".to_string(),
                    format!("Applied world change: {}", description),
                ).with_metadata("cause".to_string(), cause)
                .with_tags(vec!["world_change".to_string()])
            );
        }
        
        applied
    }

    /// Check if content is unlocked
    pub fn is_content_unlocked(&self, content_id: &str) -> bool {
        self.unlockable_content_system.is_content_unlocked(content_id) ||
        self.milestone_system.is_content_unlocked(content_id)
    }

    /// Access unlocked content
    pub fn access_content(&mut self, content_id: &str) {
        self.unlockable_content_system.access_content(content_id);
        
        // Log content access
        self.player_history_system.add_event(
            crate::progression::HistoryEvent::new(
                format!("content_accessed_{}_{}", content_id, std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs()),
                HistoryEventType::System,
                EventImportance::Trivial,
                "Content Accessed".to_string(),
                format!("Accessed content: {}", content_id),
            ).with_metadata("content_id".to_string(), content_id.to_string())
            .with_tags(vec!["access".to_string(), "content".to_string()])
        );
    }

    /// Get world changes for a specific scope
    pub fn get_world_changes_for_scope(&self, scope: &ChangeScope) -> Vec<&WorldChange> {
        self.world_changes_system.get_changes_for_scope(scope)
    }

    /// Check if a world change is active
    pub fn is_world_change_active(&self, change_id: &str) -> bool {
        self.world_changes_system.is_change_active(change_id)
    }

    /// Update world conditions
    pub fn update_world_conditions(&mut self, conditions: HashSet<String>) {
        self.world_changes_system.update_conditions(conditions);
    }

    /// Add world condition
    pub fn add_world_condition(&mut self, condition: String) {
        self.world_changes_system.add_condition(condition);
    }

    /// Remove world condition
    pub fn remove_world_condition(&mut self, condition: &str) {
        self.world_changes_system.remove_condition(condition);
    }

    /// Get milestone progress
    pub fn get_milestone_progress(&self, milestone_id: &str) -> Option<&crate::progression::MilestoneProgress> {
        self.milestone_system.get_milestone_progress(milestone_id)
    }

    /// Get milestone status
    pub fn get_milestone_status(&self, milestone_id: &str) -> crate::progression::MilestoneStatus {
        self.milestone_system.get_milestone_status(milestone_id)
    }

    /// Claim milestone rewards
    pub fn claim_milestone_rewards(&mut self, milestone_id: &str) -> Vec<crate::progression::MilestoneReward> {
        let rewards = self.milestone_system.claim_milestone_rewards(milestone_id);
        
        if !rewards.is_empty() {
            // Log reward claiming
            self.player_history_system.add_event(
                crate::progression::HistoryEvent::new(
                    format!("rewards_claimed_{}", milestone_id),
                    HistoryEventType::System,
                    EventImportance::Normal,
                    "Rewards Claimed".to_string(),
                    format!("Claimed rewards for milestone: {}", milestone_id),
                ).with_metadata("milestone_id".to_string(), milestone_id.to_string())
                .with_metadata("reward_count".to_string(), rewards.len().to_string())
                .with_tags(vec!["rewards".to_string(), "milestone".to_string()])
            );
        }
        
        rewards
    }

    /// Get available milestones
    pub fn get_available_milestones(&self) -> Vec<&crate::progression::Milestone> {
        self.milestone_system.get_available_milestones()
    }

    /// Get completed milestones
    pub fn get_completed_milestones(&self) -> Vec<(&crate::progression::Milestone, &crate::progression::CompletedMilestone)> {
        self.milestone_system.get_completed_milestones()
    }

    /// Get unlocked content
    pub fn get_unlocked_content(&self) -> Vec<(&crate::progression::UnlockableContent, &crate::progression::UnlockedContentRecord)> {
        self.unlockable_content_system.get_unlocked_content()
    }

    /// Get locked content
    pub fn get_locked_content(&self) -> Vec<&crate::progression::UnlockableContent> {
        self.unlockable_content_system.get_locked_content()
    }

    /// Get recent history events
    pub fn get_recent_history(&self, count: usize) -> Vec<&crate::progression::HistoryEvent> {
        self.player_history_system.get_recent_events(count)
    }

    /// Search history events
    pub fn search_history(&self, query: &str) -> Vec<&crate::progression::HistoryEvent> {
        self.player_history_system.search_events(query)
    }

    /// Get current session
    pub fn get_current_session(&self) -> Option<&crate::progression::GameSession> {
        self.player_history_system.get_current_session()
    }

    /// Get all sessions
    pub fn get_all_sessions(&self) -> &[crate::progression::GameSession] {
        self.player_history_system.get_all_sessions()
    }

    /// Update player stats
    pub fn update_player_stats(&mut self, stats: HashMap<String, i32>) {
        self.current_player_stats = stats;
    }

    /// Get current player stats
    pub fn get_current_player_stats(&self) -> &HashMap<String, i32> {
        &self.current_player_stats
    }

    /// Enable or disable progression tracking
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Check if progression tracking is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Get comprehensive progression statistics
    pub fn get_progression_statistics(&self) -> ProgressionStatistics {
        ProgressionStatistics {
            milestone_stats: self.milestone_system.get_statistics().clone(),
            content_stats: self.unlockable_content_system.get_statistics().clone(),
            world_changes_stats: self.world_changes_system.get_statistics().clone(),
            history_stats: self.player_history_system.get_statistics().clone(),
            current_session_id: self.current_session_id.clone(),
            enabled: self.enabled,
        }
    }

    /// Export all progression data for persistence
    pub fn export_data(&self) -> ProgressionSaveData {
        ProgressionSaveData {
            milestone_data: self.milestone_system.export_data(),
            content_data: self.unlockable_content_system.export_data(),
            world_changes_data: self.world_changes_system.export_data(),
            history_data: self.player_history_system.export_data(),
            current_player_stats: self.current_player_stats.clone(),
        }
    }

    /// Import progression data from persistence
    pub fn import_data(&mut self, data: ProgressionSaveData) {
        self.milestone_system.import_data(data.milestone_data);
        self.unlockable_content_system.import_data(data.content_data);
        self.world_changes_system.import_data(data.world_changes_data);
        self.player_history_system.import_data(data.history_data);
        self.current_player_stats = data.current_player_stats;
    }

    /// Reset progression data (for new game)
    pub fn reset_progression(&mut self) {
        self.milestone_system = MilestoneSystem::new();
        self.unlockable_content_system = UnlockableContentSystem::new();
        self.world_changes_system = WorldChangesSystem::new();
        self.player_history_system = PlayerHistorySystem::new(1000);
        self.current_player_stats.clear();
        self.current_session_id = None;
    }

    /// Get content by type
    pub fn get_content_by_type(&self, content_type: &ContentType) -> Vec<&crate::progression::UnlockableContent> {
        self.unlockable_content_system.get_content_by_type(content_type)
    }

    /// Get milestones by type
    pub fn get_milestones_by_type(&self, milestone_type: &MilestoneType) -> Vec<&crate::progression::Milestone> {
        self.milestone_system.get_milestones_by_type(milestone_type)
    }

    /// Get history events by type
    pub fn get_history_by_type(&self, event_type: &HistoryEventType) -> Vec<&crate::progression::HistoryEvent> {
        self.player_history_system.get_events_by_type(event_type)
    }

    /// Log custom event
    pub fn log_custom_event(
        &mut self,
        event_id: String,
        event_type: HistoryEventType,
        importance: EventImportance,
        title: String,
        description: String,
        location: Option<String>,
        tags: Vec<String>,
    ) {
        let mut event = crate::progression::HistoryEvent::new(
            event_id,
            event_type,
            importance,
            title,
            description,
        ).with_tags(tags);

        if let Some(loc) = location {
            event = event.with_location(loc);
        }

        self.player_history_system.add_event(event);
    }
}

/// Comprehensive progression statistics
#[derive(Debug, Clone)]
pub struct ProgressionStatistics {
    pub milestone_stats: crate::progression::MilestoneStatistics,
    pub content_stats: crate::progression::ContentUnlockStatistics,
    pub world_changes_stats: crate::progression::WorldChangeStatistics,
    pub history_stats: crate::progression::HistoryStatistics,
    pub current_session_id: Option<String>,
    pub enabled: bool,
}

/// Save data for all progression systems
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressionSaveData {
    pub milestone_data: crate::progression::MilestoneSaveData,
    pub content_data: crate::progression::UnlockableContentSaveData,
    pub world_changes_data: crate::progression::WorldChangesSaveData,
    pub history_data: crate::progression::PlayerHistorySaveData,
    pub current_player_stats: HashMap<String, i32>,
}

/// Convenience methods for common progression events
impl ProgressionIntegration {
    /// Player killed an enemy
    pub fn on_enemy_killed(&mut self, enemy_name: &str, location: &str) {
        self.process_game_event(&GameEvent::EnemyKilled, Some(location.to_string()));
        
        // Apply world change for enemy death
        self.apply_world_change(
            format!("enemy_killed_{}_{}", enemy_name, std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()),
            WorldChangeType::NPCChange,
            ChangeScope::Room(0, 0), // In real implementation, would use actual coordinates
            PersistenceLevel::Permanent,
            format!("{} was defeated", enemy_name),
            "combat".to_string(),
        );
    }

    /// Player leveled up
    pub fn on_level_up(&mut self, new_level: i32, location: &str) {
        self.process_game_event(&GameEvent::LevelChanged(new_level), Some(location.to_string()));
    }

    /// Player found an item
    pub fn on_item_found(&mut self, item_name: &str, location: &str) {
        self.process_game_event(&GameEvent::ItemCollected, Some(location.to_string()));
        
        // Apply world change for item removal
        self.apply_world_change(
            format!("item_taken_{}_{}", item_name, std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()),
            WorldChangeType::ItemPlacement,
            ChangeScope::Room(0, 0), // In real implementation, would use actual coordinates
            PersistenceLevel::Permanent,
            format!("{} was taken", item_name),
            "player_action".to_string(),
        );
    }

    /// Player collected gold
    pub fn on_gold_collected(&mut self, amount: u32) {
        self.process_game_event(&GameEvent::GoldCollected(amount), None);
    }

    /// Player entered a new room
    pub fn on_room_entered(&mut self, room_coords: (i32, i32)) {
        self.process_game_event(&GameEvent::RoomVisited, Some(format!("Room ({},{})", room_coords.0, room_coords.1)));
    }

    /// Player died
    pub fn on_player_death(&mut self, cause: &str, location: &str) {
        self.player_history_system.log_death(cause, location, self.current_player_stats.clone());
    }

    /// Door was opened
    pub fn on_door_opened(&mut self, door_id: &str, room_coords: (i32, i32)) {
        self.apply_world_change(
            format!("door_opened_{}_{}", door_id, std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()),
            WorldChangeType::StructuralChange,
            ChangeScope::Room(room_coords.0, room_coords.1),
            PersistenceLevel::Permanent,
            format!("Door {} was opened", door_id),
            "player_action".to_string(),
        );
    }

    /// Secret area discovered
    pub fn on_secret_discovered(&mut self, secret_name: &str, location: &str) {
        self.process_game_event(&GameEvent::SecretRoomFound, Some(location.to_string()));
        
        self.log_custom_event(
            format!("secret_discovered_{}", secret_name),
            HistoryEventType::Exploration,
            EventImportance::Major,
            "Secret Discovered!".to_string(),
            format!("Discovered the secret: {}", secret_name),
            Some(location.to_string()),
            vec!["secret".to_string(), "discovery".to_string()],
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progression_integration_creation() {
        let integration = ProgressionIntegration::new();
        assert!(integration.is_enabled());
    }

    #[test]
    fn test_session_management() {
        let mut integration = ProgressionIntegration::new();
        let stats = HashMap::new();
        
        let session_id = integration.start_session(stats);
        assert!(integration.current_session_id.is_some());
        assert_eq!(integration.current_session_id.as_ref().unwrap(), &session_id);
        
        integration.end_session();
        assert!(integration.current_session_id.is_none());
    }

    #[test]
    fn test_game_event_processing() {
        let mut integration = ProgressionIntegration::new();
        let stats = HashMap::new();
        integration.start_session(stats);
        
        // Process enemy kill event
        integration.on_enemy_killed("Goblin", "Room (1,1)");
        
        // Check if milestone was completed
        assert_eq!(
            integration.get_milestone_status("first_blood"),
            crate::progression::MilestoneStatus::Completed
        );
    }

    #[test]
    fn test_world_changes() {
        let mut integration = ProgressionIntegration::new();
        
        let applied = integration.apply_world_change(
            "test_change".to_string(),
            WorldChangeType::StructuralChange,
            ChangeScope::Room(1, 1),
            PersistenceLevel::Permanent,
            "Test change".to_string(),
            "test".to_string(),
        );
        
        assert!(applied);
        assert!(integration.is_world_change_active("test_change"));
    }

    #[test]
    fn test_content_unlocking() {
        let mut integration = ProgressionIntegration::new();
        let achievement_system = crate::achievements::AchievementSystem::new();
        let game_state = crate::game_state::GameState::new();
        
        // Initially, combat tutorial should not be unlocked
        assert!(!integration.is_content_unlocked("combat_tutorial"));
        
        // Process enemy kill to complete milestone
        integration.on_enemy_killed("Goblin", "Room (1,1)");
        
        // Update to check unlock conditions
        integration.update(&game_state, &achievement_system);
        
        // Now combat tutorial should be unlocked
        assert!(integration.is_content_unlocked("combat_tutorial"));
    }

    #[test]
    fn test_progression_statistics() {
        let integration = ProgressionIntegration::new();
        let stats = integration.get_progression_statistics();
        
        assert!(stats.enabled);
        assert!(stats.milestone_stats.total_milestones > 0);
        assert!(stats.content_stats.total_content > 0);
    }

    #[test]
    fn test_save_and_load() {
        let mut integration = ProgressionIntegration::new();
        let stats = HashMap::new();
        integration.start_session(stats);
        
        // Make some progress
        integration.on_enemy_killed("Goblin", "Room (1,1)");
        integration.on_level_up(2, "Room (1,1)");
        
        // Export data
        let save_data = integration.export_data();
        
        // Create new integration and import data
        let mut new_integration = ProgressionIntegration::new();
        new_integration.import_data(save_data);
        
        // Verify data was imported correctly
        assert_eq!(
            new_integration.get_milestone_status("first_blood"),
            crate::progression::MilestoneStatus::Completed
        );
    }

    #[test]
    fn test_custom_event_logging() {
        let mut integration = ProgressionIntegration::new();
        
        integration.log_custom_event(
            "test_event".to_string(),
            HistoryEventType::Special,
            EventImportance::Important,
            "Test Event".to_string(),
            "A custom test event".to_string(),
            Some("Test Location".to_string()),
            vec!["test".to_string(), "custom".to_string()],
        );
        
        let recent_events = integration.get_recent_history(10);
        assert!(recent_events.iter().any(|e| e.id == "test_event"));
    }
}