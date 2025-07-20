use std::collections::{HashMap, HashSet};
use serde::{Serialize, Deserialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// Types of world changes
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WorldChangeType {
    StructuralChange,  // Walls, doors, passages
    ItemPlacement,     // Items added/removed
    NPCChange,         // NPCs added/removed/modified
    EnvironmentalChange, // Lighting, atmosphere, etc.
    QuestChange,       // Quest-related changes
    PlayerAction,      // Changes caused by player actions
    SystemChange,      // System-generated changes
}

/// Persistence level of world changes
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PersistenceLevel {
    Temporary,    // Lasts until area reload
    Session,      // Lasts for current game session
    Permanent,    // Persists across game sessions
    Conditional,  // Persists based on conditions
}

/// World change scope
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ChangeScope {
    Room(i32, i32),           // Specific room coordinates
    Floor(i32),               // Entire floor/level
    Area(String),             // Named area
    Global,                   // Affects entire world
}

/// World change data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldChange {
    pub id: String,
    pub change_type: WorldChangeType,
    pub scope: ChangeScope,
    pub persistence: PersistenceLevel,
    pub description: String,
    pub created_at: u64,
    pub expires_at: Option<u64>,
    pub conditions: Vec<String>, // Conditions for the change to remain active
    pub metadata: HashMap<String, String>,
    pub reversible: bool,
    pub priority: i32, // Higher priority changes override lower ones
}

impl WorldChange {
    pub fn new(
        id: String,
        change_type: WorldChangeType,
        scope: ChangeScope,
        persistence: PersistenceLevel,
        description: String,
    ) -> Self {
        WorldChange {
            id,
            change_type,
            scope,
            persistence,
            description,
            created_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            expires_at: None,
            conditions: Vec::new(),
            metadata: HashMap::new(),
            reversible: false,
            priority: 0,
        }
    }

    pub fn with_expiration(mut self, expires_at: u64) -> Self {
        self.expires_at = Some(expires_at);
        self
    }

    pub fn with_conditions(mut self, conditions: Vec<String>) -> Self {
        self.conditions = conditions;
        self
    }

    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }

    pub fn with_reversible(mut self, reversible: bool) -> Self {
        self.reversible = reversible;
        self
    }

    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    /// Check if the change is expired
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            now > expires_at
        } else {
            false
        }
    }

    /// Check if conditions are met for this change to remain active
    pub fn conditions_met(&self, active_conditions: &HashSet<String>) -> bool {
        if self.conditions.is_empty() {
            return true;
        }
        
        self.conditions.iter().all(|condition| active_conditions.contains(condition))
    }
}

/// World change event for tracking what happened
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldChangeEvent {
    pub change_id: String,
    pub event_type: WorldChangeEventType,
    pub timestamp: u64,
    pub cause: String,
    pub details: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorldChangeEventType {
    Applied,
    Removed,
    Expired,
    Overridden,
    ConditionFailed,
}

impl WorldChangeEvent {
    pub fn new(
        change_id: String,
        event_type: WorldChangeEventType,
        cause: String,
    ) -> Self {
        WorldChangeEvent {
            change_id,
            event_type,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            cause,
            details: HashMap::new(),
        }
    }

    pub fn with_details(mut self, details: HashMap<String, String>) -> Self {
        self.details = details;
        self
    }
}

/// Statistics about world changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldChangeStatistics {
    pub total_changes: usize,
    pub active_changes: usize,
    pub expired_changes: usize,
    pub changes_by_type: HashMap<WorldChangeType, usize>,
    pub changes_by_scope: HashMap<ChangeScope, usize>,
    pub changes_by_persistence: HashMap<PersistenceLevel, usize>,
    pub recent_changes: Vec<String>,
    pub most_modified_areas: Vec<(ChangeScope, usize)>,
}

/// Persistent world changes system
pub struct WorldChangesSystem {
    changes: HashMap<String, WorldChange>,
    change_history: Vec<WorldChangeEvent>,
    active_conditions: HashSet<String>,
    scope_changes: HashMap<ChangeScope, Vec<String>>, // Changes by scope for efficient lookup
    statistics: WorldChangeStatistics,
    max_history_size: usize,
}

impl WorldChangesSystem {
    pub fn new() -> Self {
        let mut system = WorldChangesSystem {
            changes: HashMap::new(),
            change_history: Vec::new(),
            active_conditions: HashSet::new(),
            scope_changes: HashMap::new(),
            statistics: WorldChangeStatistics {
                total_changes: 0,
                active_changes: 0,
                expired_changes: 0,
                changes_by_type: HashMap::new(),
                changes_by_scope: HashMap::new(),
                changes_by_persistence: HashMap::new(),
                recent_changes: Vec::new(),
                most_modified_areas: Vec::new(),
            },
            max_history_size: 1000,
        };

        // Initialize with some example world changes
        system.initialize_example_changes();
        system.update_statistics();

        system
    }

    /// Initialize with example world changes
    fn initialize_example_changes(&mut self) {
        // Example: Door opened by player action
        let door_change = WorldChange::new(
            "door_room_5_3_opened".to_string(),
            WorldChangeType::StructuralChange,
            ChangeScope::Room(5, 3),
            PersistenceLevel::Permanent,
            "Door opened in room (5,3)".to_string(),
        ).with_metadata("door_id".to_string(), "main_door".to_string())
        .with_metadata("state".to_string(), "open".to_string())
        .with_reversible(true)
        .with_priority(1);

        self.apply_change(door_change, "player_action".to_string());

        // Example: Temporary lighting change
        let lighting_change = WorldChange::new(
            "torch_lit_room_2_1".to_string(),
            WorldChangeType::EnvironmentalChange,
            ChangeScope::Room(2, 1),
            PersistenceLevel::Session,
            "Torch lit in room (2,1)".to_string(),
        ).with_metadata("light_level".to_string(), "bright".to_string())
        .with_expiration(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() + 3600 // Expires in 1 hour
        );

        self.apply_change(lighting_change, "player_action".to_string());
    }

    /// Apply a world change
    pub fn apply_change(&mut self, change: WorldChange, cause: String) -> bool {
        let change_id = change.id.clone();
        let scope = change.scope.clone();

        // Check if change already exists
        if self.changes.contains_key(&change_id) {
            return false;
        }

        // Add to scope tracking
        self.scope_changes
            .entry(scope)
            .or_insert_with(Vec::new)
            .push(change_id.clone());

        // Store the change
        self.changes.insert(change_id.clone(), change);

        // Record the event
        let event = WorldChangeEvent::new(
            change_id.clone(),
            WorldChangeEventType::Applied,
            cause,
        );
        self.add_event(event);

        self.update_statistics();
        true
    }

    /// Remove a world change
    pub fn remove_change(&mut self, change_id: &str, cause: String) -> bool {
        if let Some(change) = self.changes.remove(change_id) {
            // Remove from scope tracking
            if let Some(scope_changes) = self.scope_changes.get_mut(&change.scope) {
                scope_changes.retain(|id| id != change_id);
                if scope_changes.is_empty() {
                    self.scope_changes.remove(&change.scope);
                }
            }

            // Record the event
            let event = WorldChangeEvent::new(
                change_id.to_string(),
                WorldChangeEventType::Removed,
                cause,
            );
            self.add_event(event);

            self.update_statistics();
            return true;
        }
        false
    }

    /// Update active conditions
    pub fn update_conditions(&mut self, conditions: HashSet<String>) {
        self.active_conditions = conditions;
        self.cleanup_conditional_changes();
    }

    /// Add a condition
    pub fn add_condition(&mut self, condition: String) {
        self.active_conditions.insert(condition);
        self.cleanup_conditional_changes();
    }

    /// Remove a condition
    pub fn remove_condition(&mut self, condition: &str) {
        self.active_conditions.remove(condition);
        self.cleanup_conditional_changes();
    }

    /// Clean up expired and conditional changes
    pub fn cleanup_changes(&mut self) {
        let mut to_remove = Vec::new();

        for (id, change) in &self.changes {
            if change.is_expired() {
                to_remove.push((id.clone(), "expired".to_string()));
            }
        }

        for (id, cause) in to_remove {
            self.remove_change(&id, cause);
            
            // Record expiration event
            let event = WorldChangeEvent::new(
                id,
                WorldChangeEventType::Expired,
                "automatic_cleanup".to_string(),
            );
            self.add_event(event);
        }

        self.cleanup_conditional_changes();
    }

    /// Clean up changes that no longer meet their conditions
    fn cleanup_conditional_changes(&mut self) {
        let mut to_remove = Vec::new();

        for (id, change) in &self.changes {
            if change.persistence == PersistenceLevel::Conditional {
                if !change.conditions_met(&self.active_conditions) {
                    to_remove.push(id.clone());
                }
            }
        }

        for id in to_remove {
            self.remove_change(&id, "condition_failed".to_string());
            
            // Record condition failure event
            let event = WorldChangeEvent::new(
                id,
                WorldChangeEventType::ConditionFailed,
                "condition_check".to_string(),
            );
            self.add_event(event);
        }
    }

    /// Get changes for a specific scope
    pub fn get_changes_for_scope(&self, scope: &ChangeScope) -> Vec<&WorldChange> {
        if let Some(change_ids) = self.scope_changes.get(scope) {
            change_ids.iter()
                .filter_map(|id| self.changes.get(id))
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Get changes by type
    pub fn get_changes_by_type(&self, change_type: &WorldChangeType) -> Vec<&WorldChange> {
        self.changes.values()
            .filter(|change| &change.change_type == change_type)
            .collect()
    }

    /// Get changes by persistence level
    pub fn get_changes_by_persistence(&self, persistence: &PersistenceLevel) -> Vec<&WorldChange> {
        self.changes.values()
            .filter(|change| &change.persistence == persistence)
            .collect()
    }

    /// Get all active changes
    pub fn get_active_changes(&self) -> Vec<&WorldChange> {
        self.changes.values()
            .filter(|change| !change.is_expired() && change.conditions_met(&self.active_conditions))
            .collect()
    }

    /// Check if a specific change is active
    pub fn is_change_active(&self, change_id: &str) -> bool {
        if let Some(change) = self.changes.get(change_id) {
            !change.is_expired() && change.conditions_met(&self.active_conditions)
        } else {
            false
        }
    }

    /// Get change by ID
    pub fn get_change(&self, change_id: &str) -> Option<&WorldChange> {
        self.changes.get(change_id)
    }

    /// Get all changes
    pub fn get_all_changes(&self) -> Vec<&WorldChange> {
        self.changes.values().collect()
    }

    /// Get change history
    pub fn get_change_history(&self) -> &[WorldChangeEvent] {
        &self.change_history
    }

    /// Get recent change history
    pub fn get_recent_history(&self, count: usize) -> Vec<&WorldChangeEvent> {
        self.change_history.iter()
            .rev()
            .take(count)
            .collect()
    }

    /// Add event to history
    fn add_event(&mut self, event: WorldChangeEvent) {
        self.change_history.push(event);
        
        // Limit history size
        if self.change_history.len() > self.max_history_size {
            self.change_history.remove(0);
        }
    }

    /// Update statistics
    fn update_statistics(&mut self) {
        let total_changes = self.changes.len();
        let active_changes = self.get_active_changes().len();
        let expired_changes = self.changes.values()
            .filter(|change| change.is_expired())
            .count();

        // Count by type
        let mut changes_by_type = HashMap::new();
        for change in self.changes.values() {
            *changes_by_type.entry(change.change_type.clone()).or_insert(0) += 1;
        }

        // Count by scope
        let mut changes_by_scope = HashMap::new();
        for change in self.changes.values() {
            *changes_by_scope.entry(change.scope.clone()).or_insert(0) += 1;
        }

        // Count by persistence
        let mut changes_by_persistence = HashMap::new();
        for change in self.changes.values() {
            *changes_by_persistence.entry(change.persistence.clone()).or_insert(0) += 1;
        }

        // Recent changes (last 10)
        let mut recent_changes: Vec<(String, u64)> = self.changes.iter()
            .map(|(id, change)| (id.clone(), change.created_at))
            .collect();
        recent_changes.sort_by(|a, b| b.1.cmp(&a.1));
        let recent_changes: Vec<String> = recent_changes.into_iter()
            .take(10)
            .map(|(id, _)| id)
            .collect();

        // Most modified areas
        let mut area_counts: HashMap<ChangeScope, usize> = HashMap::new();
        for change in self.changes.values() {
            *area_counts.entry(change.scope.clone()).or_insert(0) += 1;
        }
        let mut most_modified_areas: Vec<(ChangeScope, usize)> = area_counts.into_iter().collect();
        most_modified_areas.sort_by(|a, b| b.1.cmp(&a.1));
        most_modified_areas.truncate(10);

        self.statistics = WorldChangeStatistics {
            total_changes,
            active_changes,
            expired_changes,
            changes_by_type,
            changes_by_scope,
            changes_by_persistence,
            recent_changes,
            most_modified_areas,
        };
    }

    /// Get statistics
    pub fn get_statistics(&self) -> &WorldChangeStatistics {
        &self.statistics
    }

    /// Clear session changes
    pub fn clear_session_changes(&mut self) {
        let session_changes: Vec<String> = self.changes.iter()
            .filter(|(_, change)| change.persistence == PersistenceLevel::Session)
            .map(|(id, _)| id.clone())
            .collect();

        for change_id in session_changes {
            self.remove_change(&change_id, "session_end".to_string());
        }
    }

    /// Clear temporary changes
    pub fn clear_temporary_changes(&mut self) {
        let temp_changes: Vec<String> = self.changes.iter()
            .filter(|(_, change)| change.persistence == PersistenceLevel::Temporary)
            .map(|(id, _)| id.clone())
            .collect();

        for change_id in temp_changes {
            self.remove_change(&change_id, "area_reload".to_string());
        }
    }

    /// Export data for persistence
    pub fn export_data(&self) -> WorldChangesSaveData {
        // Only export permanent and conditional changes
        let persistent_changes: HashMap<String, WorldChange> = self.changes.iter()
            .filter(|(_, change)| {
                matches!(change.persistence, PersistenceLevel::Permanent | PersistenceLevel::Conditional)
            })
            .map(|(id, change)| (id.clone(), change.clone()))
            .collect();

        WorldChangesSaveData {
            changes: persistent_changes,
            active_conditions: self.active_conditions.clone(),
        }
    }

    /// Import data from persistence
    pub fn import_data(&mut self, data: WorldChangesSaveData) {
        // Clear existing permanent changes
        let permanent_changes: Vec<String> = self.changes.iter()
            .filter(|(_, change)| {
                matches!(change.persistence, PersistenceLevel::Permanent | PersistenceLevel::Conditional)
            })
            .map(|(id, _)| id.clone())
            .collect();

        for change_id in permanent_changes {
            self.remove_change(&change_id, "data_import".to_string());
        }

        // Import new changes
        for (id, change) in data.changes {
            let scope = change.scope.clone();
            self.scope_changes
                .entry(scope)
                .or_insert_with(Vec::new)
                .push(id.clone());
            self.changes.insert(id, change);
        }

        self.active_conditions = data.active_conditions;
        self.update_statistics();
    }

    /// Set maximum history size
    pub fn set_max_history_size(&mut self, size: usize) {
        self.max_history_size = size;
        
        // Trim current history if needed
        if self.change_history.len() > size {
            let excess = self.change_history.len() - size;
            self.change_history.drain(0..excess);
        }
    }
}

/// Save data for world changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldChangesSaveData {
    pub changes: HashMap<String, WorldChange>,
    pub active_conditions: HashSet<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_world_changes_system_creation() {
        let system = WorldChangesSystem::new();
        assert!(system.changes.len() > 0);
        assert!(system.change_history.len() > 0);
    }

    #[test]
    fn test_apply_and_remove_change() {
        let mut system = WorldChangesSystem::new();
        
        let change = WorldChange::new(
            "test_change".to_string(),
            WorldChangeType::ItemPlacement,
            ChangeScope::Room(1, 1),
            PersistenceLevel::Permanent,
            "Test change".to_string(),
        );

        // Apply change
        let applied = system.apply_change(change, "test".to_string());
        assert!(applied);
        assert!(system.changes.contains_key("test_change"));

        // Remove change
        let removed = system.remove_change("test_change", "test".to_string());
        assert!(removed);
        assert!(!system.changes.contains_key("test_change"));
    }

    #[test]
    fn test_change_expiration() {
        let mut system = WorldChangesSystem::new();
        
        let expired_change = WorldChange::new(
            "expired_change".to_string(),
            WorldChangeType::EnvironmentalChange,
            ChangeScope::Room(1, 1),
            PersistenceLevel::Temporary,
            "Expired change".to_string(),
        ).with_expiration(1); // Already expired

        system.apply_change(expired_change, "test".to_string());
        assert!(system.changes.contains_key("expired_change"));

        // Cleanup should remove expired changes
        system.cleanup_changes();
        assert!(!system.changes.contains_key("expired_change"));
    }

    #[test]
    fn test_conditional_changes() {
        let mut system = WorldChangesSystem::new();
        
        let conditional_change = WorldChange::new(
            "conditional_change".to_string(),
            WorldChangeType::NPCChange,
            ChangeScope::Room(1, 1),
            PersistenceLevel::Conditional,
            "Conditional change".to_string(),
        ).with_conditions(vec!["quest_active".to_string()]);

        system.apply_change(conditional_change, "test".to_string());
        assert!(system.changes.contains_key("conditional_change"));

        // Without the condition, change should be removed
        system.cleanup_changes();
        assert!(!system.changes.contains_key("conditional_change"));

        // Add the change again with condition
        let conditional_change2 = WorldChange::new(
            "conditional_change2".to_string(),
            WorldChangeType::NPCChange,
            ChangeScope::Room(1, 1),
            PersistenceLevel::Conditional,
            "Conditional change 2".to_string(),
        ).with_conditions(vec!["quest_active".to_string()]);

        system.apply_change(conditional_change2, "test".to_string());
        system.add_condition("quest_active".to_string());
        
        // Now it should remain
        system.cleanup_changes();
        assert!(system.changes.contains_key("conditional_change2"));
    }

    #[test]
    fn test_scope_filtering() {
        let mut system = WorldChangesSystem::new();
        
        let room_change = WorldChange::new(
            "room_change".to_string(),
            WorldChangeType::StructuralChange,
            ChangeScope::Room(5, 5),
            PersistenceLevel::Permanent,
            "Room change".to_string(),
        );

        system.apply_change(room_change, "test".to_string());
        
        let room_changes = system.get_changes_for_scope(&ChangeScope::Room(5, 5));
        assert_eq!(room_changes.len(), 1);
        assert_eq!(room_changes[0].id, "room_change");
    }

    #[test]
    fn test_change_statistics() {
        let system = WorldChangesSystem::new();
        let stats = system.get_statistics();
        
        assert!(stats.total_changes > 0);
        assert!(stats.active_changes > 0);
        assert!(!stats.changes_by_type.is_empty());
    }

    #[test]
    fn test_save_and_load() {
        let mut system = WorldChangesSystem::new();
        
        let permanent_change = WorldChange::new(
            "permanent_test".to_string(),
            WorldChangeType::StructuralChange,
            ChangeScope::Room(10, 10),
            PersistenceLevel::Permanent,
            "Permanent test change".to_string(),
        );

        system.apply_change(permanent_change, "test".to_string());
        system.add_condition("test_condition".to_string());
        
        // Export data
        let save_data = system.export_data();
        assert!(save_data.changes.contains_key("permanent_test"));
        assert!(save_data.active_conditions.contains("test_condition"));
        
        // Create new system and import data
        let mut new_system = WorldChangesSystem::new();
        new_system.import_data(save_data);
        
        // Verify data was imported correctly
        assert!(new_system.changes.contains_key("permanent_test"));
        assert!(new_system.active_conditions.contains("test_condition"));
    }

    #[test]
    fn test_session_cleanup() {
        let mut system = WorldChangesSystem::new();
        
        let session_change = WorldChange::new(
            "session_change".to_string(),
            WorldChangeType::EnvironmentalChange,
            ChangeScope::Room(1, 1),
            PersistenceLevel::Session,
            "Session change".to_string(),
        );

        system.apply_change(session_change, "test".to_string());
        assert!(system.changes.contains_key("session_change"));

        // Clear session changes
        system.clear_session_changes();
        assert!(!system.changes.contains_key("session_change"));
    }
}