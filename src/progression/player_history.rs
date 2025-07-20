use std::collections::{HashMap, VecDeque};
use serde::{Serialize, Deserialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// Types of events to log in player history
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HistoryEventType {
    Combat,
    Exploration,
    Character,
    Items,
    Quests,
    Social,
    System,
    Achievement,
    Milestone,
    Death,
    Special,
}

/// Importance levels for history events
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum EventImportance {
    Trivial,
    Minor,
    Normal,
    Important,
    Major,
    Critical,
    Legendary,
}

/// Player history event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEvent {
    pub id: String,
    pub event_type: HistoryEventType,
    pub importance: EventImportance,
    pub timestamp: u64,
    pub title: String,
    pub description: String,
    pub location: Option<String>, // Where the event occurred
    pub participants: Vec<String>, // NPCs, enemies, etc. involved
    pub items_involved: Vec<String>, // Items gained, lost, or used
    pub stats_before: HashMap<String, i32>, // Player stats before event
    pub stats_after: HashMap<String, i32>, // Player stats after event
    pub metadata: HashMap<String, String>, // Additional event-specific data
    pub tags: Vec<String>, // Tags for categorization and searching
}

impl HistoryEvent {
    pub fn new(
        id: String,
        event_type: HistoryEventType,
        importance: EventImportance,
        title: String,
        description: String,
    ) -> Self {
        HistoryEvent {
            id,
            event_type,
            importance,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            title,
            description,
            location: None,
            participants: Vec::new(),
            items_involved: Vec::new(),
            stats_before: HashMap::new(),
            stats_after: HashMap::new(),
            metadata: HashMap::new(),
            tags: Vec::new(),
        }
    }

    pub fn with_location(mut self, location: String) -> Self {
        self.location = Some(location);
        self
    }

    pub fn with_participants(mut self, participants: Vec<String>) -> Self {
        self.participants = participants;
        self
    }

    pub fn with_items(mut self, items: Vec<String>) -> Self {
        self.items_involved = items;
        self
    }

    pub fn with_stats_before(mut self, stats: HashMap<String, i32>) -> Self {
        self.stats_before = stats;
        self
    }

    pub fn with_stats_after(mut self, stats: HashMap<String, i32>) -> Self {
        self.stats_after = stats;
        self
    }

    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }

    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    /// Get the age of this event in seconds
    pub fn age_seconds(&self) -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
            .saturating_sub(self.timestamp)
    }

    /// Get stat changes from this event
    pub fn get_stat_changes(&self) -> HashMap<String, i32> {
        let mut changes = HashMap::new();
        
        for (stat, after_value) in &self.stats_after {
            let before_value = self.stats_before.get(stat).unwrap_or(&0);
            let change = after_value - before_value;
            if change != 0 {
                changes.insert(stat.clone(), change);
            }
        }
        
        changes
    }
}

/// Player session information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameSession {
    pub session_id: String,
    pub start_time: u64,
    pub end_time: Option<u64>,
    pub duration_seconds: u64,
    pub events_count: usize,
    pub major_events: Vec<String>, // IDs of major events in this session
    pub starting_stats: HashMap<String, i32>,
    pub ending_stats: HashMap<String, i32>,
    pub locations_visited: Vec<String>,
    pub achievements_earned: Vec<String>,
    pub milestones_completed: Vec<String>,
}

impl GameSession {
    pub fn new(session_id: String, starting_stats: HashMap<String, i32>) -> Self {
        GameSession {
            session_id,
            start_time: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            end_time: None,
            duration_seconds: 0,
            events_count: 0,
            major_events: Vec::new(),
            starting_stats,
            ending_stats: HashMap::new(),
            locations_visited: Vec::new(),
            achievements_earned: Vec::new(),
            milestones_completed: Vec::new(),
        }
    }

    pub fn end_session(&mut self, ending_stats: HashMap<String, i32>) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        self.end_time = Some(now);
        self.duration_seconds = now - self.start_time;
        self.ending_stats = ending_stats;
    }

    pub fn is_active(&self) -> bool {
        self.end_time.is_none()
    }
}

/// History statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryStatistics {
    pub total_events: usize,
    pub events_by_type: HashMap<HistoryEventType, usize>,
    pub events_by_importance: HashMap<EventImportance, usize>,
    pub total_sessions: usize,
    pub total_playtime_seconds: u64,
    pub average_session_duration: u64,
    pub most_common_locations: Vec<(String, usize)>,
    pub most_active_days: Vec<(String, usize)>, // Date -> event count
    pub recent_events: Vec<String>, // Recent event IDs
    pub milestone_events: Vec<String>, // Important milestone event IDs
}

/// Player history system
pub struct PlayerHistorySystem {
    events: VecDeque<HistoryEvent>,
    sessions: Vec<GameSession>,
    current_session: Option<GameSession>,
    event_index: HashMap<String, usize>, // Event ID -> index in events
    max_events: usize,
    statistics: HistoryStatistics,
}

impl PlayerHistorySystem {
    pub fn new(max_events: usize) -> Self {
        let mut system = PlayerHistorySystem {
            events: VecDeque::new(),
            sessions: Vec::new(),
            current_session: None,
            event_index: HashMap::new(),
            max_events,
            statistics: HistoryStatistics {
                total_events: 0,
                events_by_type: HashMap::new(),
                events_by_importance: HashMap::new(),
                total_sessions: 0,
                total_playtime_seconds: 0,
                average_session_duration: 0,
                most_common_locations: Vec::new(),
                most_active_days: Vec::new(),
                recent_events: Vec::new(),
                milestone_events: Vec::new(),
            },
        };

        // Create initial example events
        system.initialize_example_events();
        system.update_statistics();

        system
    }

    /// Initialize with example events
    fn initialize_example_events(&mut self) {
        let example_events = vec![
            HistoryEvent::new(
                "game_start".to_string(),
                HistoryEventType::System,
                EventImportance::Important,
                "Adventure Begins".to_string(),
                "Started a new adventure in the dungeon".to_string(),
            ).with_location("Dungeon Entrance".to_string())
            .with_tags(vec!["start".to_string(), "new_game".to_string()]),

            HistoryEvent::new(
                "first_enemy_kill".to_string(),
                HistoryEventType::Combat,
                EventImportance::Normal,
                "First Victory".to_string(),
                "Defeated your first enemy - a goblin".to_string(),
            ).with_location("Room (1,1)".to_string())
            .with_participants(vec!["Goblin".to_string()])
            .with_tags(vec!["combat".to_string(), "first".to_string()]),

            HistoryEvent::new(
                "level_up_2".to_string(),
                HistoryEventType::Character,
                EventImportance::Important,
                "Level Up!".to_string(),
                "Reached level 2 through combat experience".to_string(),
            ).with_metadata("new_level".to_string(), "2".to_string())
            .with_tags(vec!["level_up".to_string(), "progression".to_string()]),
        ];

        for event in example_events {
            self.add_event(event);
        }
    }

    /// Start a new game session
    pub fn start_session(&mut self, starting_stats: HashMap<String, i32>) -> String {
        // End current session if active
        if let Some(mut session) = self.current_session.take() {
            session.end_session(starting_stats.clone());
            self.sessions.push(session);
        }

        // Create new session
        let session_id = format!("session_{}", 
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
        );

        let session = GameSession::new(session_id.clone(), starting_stats);
        self.current_session = Some(session);

        // Log session start event
        let start_event = HistoryEvent::new(
            format!("session_start_{}", session_id),
            HistoryEventType::System,
            EventImportance::Minor,
            "Session Started".to_string(),
            "Started a new game session".to_string(),
        ).with_metadata("session_id".to_string(), session_id.clone())
        .with_tags(vec!["session".to_string(), "start".to_string()]);

        self.add_event(start_event);

        session_id
    }

    /// End the current session
    pub fn end_session(&mut self, ending_stats: HashMap<String, i32>) {
        if let Some(mut session) = self.current_session.take() {
            session.end_session(ending_stats);
            
            // Log session end event
            let end_event = HistoryEvent::new(
                format!("session_end_{}", session.session_id),
                HistoryEventType::System,
                EventImportance::Minor,
                "Session Ended".to_string(),
                format!("Session lasted {} seconds", session.duration_seconds),
            ).with_metadata("session_id".to_string(), session.session_id.clone())
            .with_metadata("duration".to_string(), session.duration_seconds.to_string())
            .with_tags(vec!["session".to_string(), "end".to_string()]);

            self.add_event(end_event);
            self.sessions.push(session);
            self.update_statistics();
        }
    }

    /// Add an event to the history
    pub fn add_event(&mut self, event: HistoryEvent) {
        let event_id = event.id.clone();
        
        // Update current session
        if let Some(session) = &mut self.current_session {
            session.events_count += 1;
            
            if event.importance >= EventImportance::Important {
                session.major_events.push(event_id.clone());
            }
            
            if let Some(location) = &event.location {
                if !session.locations_visited.contains(location) {
                    session.locations_visited.push(location.clone());
                }
            }
        }

        // Add to events deque
        if self.events.len() >= self.max_events {
            if let Some(old_event) = self.events.pop_front() {
                self.event_index.remove(&old_event.id);
            }
        }

        let index = self.events.len();
        self.event_index.insert(event_id, index);
        self.events.push_back(event);

        self.update_statistics();
    }

    /// Get event by ID
    pub fn get_event(&self, event_id: &str) -> Option<&HistoryEvent> {
        if let Some(&index) = self.event_index.get(event_id) {
            self.events.get(index)
        } else {
            None
        }
    }

    /// Get all events
    pub fn get_all_events(&self) -> Vec<&HistoryEvent> {
        self.events.iter().collect()
    }

    /// Get events by type
    pub fn get_events_by_type(&self, event_type: &HistoryEventType) -> Vec<&HistoryEvent> {
        self.events.iter()
            .filter(|event| &event.event_type == event_type)
            .collect()
    }

    /// Get events by importance
    pub fn get_events_by_importance(&self, importance: &EventImportance) -> Vec<&HistoryEvent> {
        self.events.iter()
            .filter(|event| &event.importance == importance)
            .collect()
    }

    /// Get events by location
    pub fn get_events_by_location(&self, location: &str) -> Vec<&HistoryEvent> {
        self.events.iter()
            .filter(|event| {
                event.location.as_ref().map_or(false, |loc| loc == location)
            })
            .collect()
    }

    /// Get events with specific tag
    pub fn get_events_by_tag(&self, tag: &str) -> Vec<&HistoryEvent> {
        self.events.iter()
            .filter(|event| event.tags.contains(&tag.to_string()))
            .collect()
    }

    /// Get recent events
    pub fn get_recent_events(&self, count: usize) -> Vec<&HistoryEvent> {
        self.events.iter()
            .rev()
            .take(count)
            .collect()
    }

    /// Get events from time range
    pub fn get_events_in_time_range(&self, start_time: u64, end_time: u64) -> Vec<&HistoryEvent> {
        self.events.iter()
            .filter(|event| event.timestamp >= start_time && event.timestamp <= end_time)
            .collect()
    }

    /// Search events by text
    pub fn search_events(&self, query: &str) -> Vec<&HistoryEvent> {
        let query_lower = query.to_lowercase();
        self.events.iter()
            .filter(|event| {
                event.title.to_lowercase().contains(&query_lower) ||
                event.description.to_lowercase().contains(&query_lower) ||
                event.tags.iter().any(|tag| tag.to_lowercase().contains(&query_lower))
            })
            .collect()
    }

    /// Get current session
    pub fn get_current_session(&self) -> Option<&GameSession> {
        self.current_session.as_ref()
    }

    /// Get all sessions
    pub fn get_all_sessions(&self) -> &[GameSession] {
        &self.sessions
    }

    /// Get session by ID
    pub fn get_session(&self, session_id: &str) -> Option<&GameSession> {
        self.sessions.iter()
            .find(|session| session.session_id == session_id)
            .or_else(|| {
                self.current_session.as_ref()
                    .filter(|session| session.session_id == session_id)
            })
    }

    /// Log common game events
    pub fn log_combat_victory(&mut self, enemy_name: &str, location: &str, stats_before: HashMap<String, i32>, stats_after: HashMap<String, i32>) {
        let event = HistoryEvent::new(
            format!("combat_victory_{}", self.events.len()),
            HistoryEventType::Combat,
            EventImportance::Normal,
            format!("Defeated {}", enemy_name),
            format!("Successfully defeated {} in combat", enemy_name),
        ).with_location(location.to_string())
        .with_participants(vec![enemy_name.to_string()])
        .with_stats_before(stats_before)
        .with_stats_after(stats_after)
        .with_tags(vec!["combat".to_string(), "victory".to_string()]);

        self.add_event(event);
    }

    pub fn log_level_up(&mut self, new_level: u32, location: &str, stats_before: HashMap<String, i32>, stats_after: HashMap<String, i32>) {
        let event = HistoryEvent::new(
            format!("level_up_{}", new_level),
            HistoryEventType::Character,
            EventImportance::Important,
            format!("Reached Level {}", new_level),
            format!("Advanced to level {} through experience", new_level),
        ).with_location(location.to_string())
        .with_stats_before(stats_before)
        .with_stats_after(stats_after)
        .with_metadata("new_level".to_string(), new_level.to_string())
        .with_tags(vec!["level_up".to_string(), "progression".to_string()]);

        self.add_event(event);
    }

    pub fn log_item_found(&mut self, item_name: &str, location: &str) {
        let event = HistoryEvent::new(
            format!("item_found_{}_{}", item_name, self.events.len()),
            HistoryEventType::Items,
            EventImportance::Minor,
            format!("Found {}", item_name),
            format!("Discovered {} while exploring", item_name),
        ).with_location(location.to_string())
        .with_items(vec![item_name.to_string()])
        .with_tags(vec!["item".to_string(), "found".to_string()]);

        self.add_event(event);
    }

    pub fn log_death(&mut self, cause: &str, location: &str, stats_before: HashMap<String, i32>) {
        let event = HistoryEvent::new(
            format!("death_{}", self.events.len()),
            HistoryEventType::Death,
            EventImportance::Major,
            "Death".to_string(),
            format!("Died from {}", cause),
        ).with_location(location.to_string())
        .with_stats_before(stats_before)
        .with_metadata("cause".to_string(), cause.to_string())
        .with_tags(vec!["death".to_string(), "setback".to_string()]);

        self.add_event(event);
    }

    pub fn log_achievement(&mut self, achievement_name: &str, achievement_id: &str) {
        let event = HistoryEvent::new(
            format!("achievement_{}", achievement_id),
            HistoryEventType::Achievement,
            EventImportance::Important,
            format!("Achievement: {}", achievement_name),
            format!("Unlocked the '{}' achievement", achievement_name),
        ).with_metadata("achievement_id".to_string(), achievement_id.to_string())
        .with_tags(vec!["achievement".to_string(), "unlock".to_string()]);

        self.add_event(event);

        // Update current session
        if let Some(session) = &mut self.current_session {
            session.achievements_earned.push(achievement_id.to_string());
        }
    }

    pub fn log_milestone(&mut self, milestone_name: &str, milestone_id: &str) {
        let event = HistoryEvent::new(
            format!("milestone_{}", milestone_id),
            HistoryEventType::Milestone,
            EventImportance::Major,
            format!("Milestone: {}", milestone_name),
            format!("Completed the '{}' milestone", milestone_name),
        ).with_metadata("milestone_id".to_string(), milestone_id.to_string())
        .with_tags(vec!["milestone".to_string(), "completion".to_string()]);

        self.add_event(event);

        // Update current session
        if let Some(session) = &mut self.current_session {
            session.milestones_completed.push(milestone_id.to_string());
        }
    }

    /// Update statistics
    fn update_statistics(&mut self) {
        let total_events = self.events.len();
        
        // Count by type
        let mut events_by_type = HashMap::new();
        for event in &self.events {
            *events_by_type.entry(event.event_type.clone()).or_insert(0) += 1;
        }

        // Count by importance
        let mut events_by_importance = HashMap::new();
        for event in &self.events {
            *events_by_importance.entry(event.importance.clone()).or_insert(0) += 1;
        }

        // Session statistics
        let total_sessions = self.sessions.len() + if self.current_session.is_some() { 1 } else { 0 };
        let total_playtime_seconds: u64 = self.sessions.iter()
            .map(|s| s.duration_seconds)
            .sum::<u64>() + 
            self.current_session.as_ref()
                .map(|s| {
                    SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs()
                        .saturating_sub(s.start_time)
                })
                .unwrap_or(0);

        let average_session_duration = if total_sessions > 0 {
            total_playtime_seconds / total_sessions as u64
        } else {
            0
        };

        // Most common locations
        let mut location_counts = HashMap::new();
        for event in &self.events {
            if let Some(location) = &event.location {
                *location_counts.entry(location.clone()).or_insert(0) += 1;
            }
        }
        let mut most_common_locations: Vec<(String, usize)> = location_counts.into_iter().collect();
        most_common_locations.sort_by(|a, b| b.1.cmp(&a.1));
        most_common_locations.truncate(10);

        // Recent events (last 10)
        let recent_events: Vec<String> = self.events.iter()
            .rev()
            .take(10)
            .map(|event| event.id.clone())
            .collect();

        // Milestone events
        let milestone_events: Vec<String> = self.events.iter()
            .filter(|event| event.importance >= EventImportance::Major)
            .map(|event| event.id.clone())
            .collect();

        self.statistics = HistoryStatistics {
            total_events,
            events_by_type,
            events_by_importance,
            total_sessions,
            total_playtime_seconds,
            average_session_duration,
            most_common_locations,
            most_active_days: Vec::new(), // Would need date parsing for this
            recent_events,
            milestone_events,
        };
    }

    /// Get statistics
    pub fn get_statistics(&self) -> &HistoryStatistics {
        &self.statistics
    }

    /// Export data for persistence
    pub fn export_data(&self) -> PlayerHistorySaveData {
        PlayerHistorySaveData {
            events: self.events.iter().cloned().collect(),
            sessions: self.sessions.clone(),
            current_session: self.current_session.clone(),
        }
    }

    /// Import data from persistence
    pub fn import_data(&mut self, data: PlayerHistorySaveData) {
        self.events = data.events.into();
        self.sessions = data.sessions;
        self.current_session = data.current_session;

        // Rebuild event index
        self.event_index.clear();
        for (index, event) in self.events.iter().enumerate() {
            self.event_index.insert(event.id.clone(), index);
        }

        self.update_statistics();
    }

    /// Set maximum number of events to keep
    pub fn set_max_events(&mut self, max_events: usize) {
        self.max_events = max_events;
        
        // Trim events if necessary
        while self.events.len() > max_events {
            if let Some(old_event) = self.events.pop_front() {
                self.event_index.remove(&old_event.id);
            }
        }
        
        // Rebuild index
        self.event_index.clear();
        for (index, event) in self.events.iter().enumerate() {
            self.event_index.insert(event.id.clone(), index);
        }
    }
}

/// Save data for player history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerHistorySaveData {
    pub events: Vec<HistoryEvent>,
    pub sessions: Vec<GameSession>,
    pub current_session: Option<GameSession>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_player_history_system_creation() {
        let system = PlayerHistorySystem::new(100);
        assert!(system.events.len() > 0);
        assert!(system.max_events == 100);
    }

    #[test]
    fn test_add_event() {
        let mut system = PlayerHistorySystem::new(100);
        let initial_count = system.events.len();
        
        let event = HistoryEvent::new(
            "test_event".to_string(),
            HistoryEventType::Combat,
            EventImportance::Normal,
            "Test Event".to_string(),
            "A test event".to_string(),
        );

        system.add_event(event);
        assert_eq!(system.events.len(), initial_count + 1);
        assert!(system.event_index.contains_key("test_event"));
    }

    #[test]
    fn test_session_management() {
        let mut system = PlayerHistorySystem::new(100);
        let stats = HashMap::new();
        
        // Start session
        let session_id = system.start_session(stats.clone());
        assert!(system.current_session.is_some());
        assert_eq!(system.current_session.as_ref().unwrap().session_id, session_id);
        
        // End session
        system.end_session(stats);
        assert!(system.current_session.is_none());
        assert_eq!(system.sessions.len(), 1);
    }

    #[test]
    fn test_event_filtering() {
        let mut system = PlayerHistorySystem::new(100);
        
        // Add events of different types
        let combat_event = HistoryEvent::new(
            "combat_test".to_string(),
            HistoryEventType::Combat,
            EventImportance::Normal,
            "Combat Test".to_string(),
            "Combat event".to_string(),
        );
        
        let exploration_event = HistoryEvent::new(
            "exploration_test".to_string(),
            HistoryEventType::Exploration,
            EventImportance::Important,
            "Exploration Test".to_string(),
            "Exploration event".to_string(),
        );

        system.add_event(combat_event);
        system.add_event(exploration_event);

        // Test filtering by type
        let combat_events = system.get_events_by_type(&HistoryEventType::Combat);
        assert!(combat_events.iter().any(|e| e.id == "combat_test"));

        // Test filtering by importance
        let important_events = system.get_events_by_importance(&EventImportance::Important);
        assert!(important_events.iter().any(|e| e.id == "exploration_test"));
    }

    #[test]
    fn test_event_search() {
        let mut system = PlayerHistorySystem::new(100);
        
        let event = HistoryEvent::new(
            "searchable_event".to_string(),
            HistoryEventType::Special,
            EventImportance::Normal,
            "Unique Title".to_string(),
            "This event has unique content".to_string(),
        ).with_tags(vec!["special".to_string(), "unique".to_string()]);

        system.add_event(event);

        // Test text search
        let results = system.search_events("unique");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "searchable_event");

        // Test tag search
        let tag_results = system.get_events_by_tag("special");
        assert_eq!(tag_results.len(), 1);
        assert_eq!(tag_results[0].id, "searchable_event");
    }

    #[test]
    fn test_max_events_limit() {
        let mut system = PlayerHistorySystem::new(3); // Very small limit
        
        // Add more events than the limit
        for i in 0..5 {
            let event = HistoryEvent::new(
                format!("event_{}", i),
                HistoryEventType::System,
                EventImportance::Minor,
                format!("Event {}", i),
                format!("Event number {}", i),
            );
            system.add_event(event);
        }

        // Should only keep the last 3 events
        assert_eq!(system.events.len(), 3);
        assert!(!system.event_index.contains_key("event_0"));
        assert!(!system.event_index.contains_key("event_1"));
        assert!(system.event_index.contains_key("event_4"));
    }

    #[test]
    fn test_logging_methods() {
        let mut system = PlayerHistorySystem::new(100);
        let stats_before = HashMap::new();
        let stats_after = HashMap::new();
        
        // Test combat logging
        system.log_combat_victory("Goblin", "Room (1,1)", stats_before.clone(), stats_after.clone());
        let combat_events = system.get_events_by_type(&HistoryEventType::Combat);
        assert!(combat_events.len() > 0);

        // Test level up logging
        system.log_level_up(5, "Room (2,2)", stats_before, stats_after);
        let character_events = system.get_events_by_type(&HistoryEventType::Character);
        assert!(character_events.len() > 0);

        // Test achievement logging
        system.log_achievement("First Kill", "first_kill");
        let achievement_events = system.get_events_by_type(&HistoryEventType::Achievement);
        assert!(achievement_events.len() > 0);
    }

    #[test]
    fn test_save_and_load() {
        let mut system = PlayerHistorySystem::new(100);
        
        // Add some events and start a session
        let stats = HashMap::new();
        system.start_session(stats.clone());
        system.log_combat_victory("Test Enemy", "Test Location", stats.clone(), stats.clone());
        
        // Export data
        let save_data = system.export_data();
        assert!(save_data.events.len() > 0);
        assert!(save_data.current_session.is_some());
        
        // Create new system and import data
        let mut new_system = PlayerHistorySystem::new(100);
        new_system.import_data(save_data);
        
        // Verify data was imported correctly
        assert!(new_system.events.len() > 0);
        assert!(new_system.current_session.is_some());
        assert!(!new_system.event_index.is_empty());
    }

    #[test]
    fn test_statistics() {
        let system = PlayerHistorySystem::new(100);
        let stats = system.get_statistics();
        
        assert!(stats.total_events > 0);
        assert!(!stats.events_by_type.is_empty());
        assert!(!stats.recent_events.is_empty());
    }
}