use bevy::prelude::*;
use std::collections::HashMap;
use rand::{Rng, thread_rng};
use crate::guild::mission_types::*;
use crate::guild::mission::Mission;

/// Mission board resource
#[derive(Resource, Default)]
pub struct MissionBoard {
    pub missions: HashMap<String, Mission>,
    pub mission_count: u32,
}

impl MissionBoard {
    /// Create a new mission board
    pub fn new() -> Self {
        MissionBoard {
            missions: HashMap::new(),
            mission_count: 0,
        }
    }
    
    /// Add a mission to the board
    pub fn add_mission(&mut self, mission: Mission) {
        self.missions.insert(mission.id.clone(), mission);
    }
    
    /// Remove a mission from the board
    pub fn remove_mission(&mut self, mission_id: &str) -> Option<Mission> {
        self.missions.remove(mission_id)
    }
    
    /// Get a mission by ID
    pub fn get_mission(&self, mission_id: &str) -> Option<&Mission> {
        self.missions.get(mission_id)
    }
    
    /// Get a mutable reference to a mission
    pub fn get_mission_mut(&mut self, mission_id: &str) -> Option<&mut Mission> {
        self.missions.get_mut(mission_id)
    }
    
    /// Get all available missions
    pub fn get_available_missions(&self) -> Vec<&Mission> {
        self.missions.values()
            .filter(|m| m.status == MissionStatus::Available)
            .collect()
    }
    
    /// Get missions by status
    pub fn get_missions_by_status(&self, status: MissionStatus) -> Vec<&Mission> {
        self.missions.values()
            .filter(|m| m.status == status)
            .collect()
    }
    
    /// Get missions by guild
    pub fn get_missions_by_guild(&self, guild_id: &str) -> Vec<&Mission> {
        self.missions.values()
            .filter(|m| m.guild_id == guild_id)
            .collect()
    }
    
    /// Get missions by tag
    pub fn get_missions_by_tag(&self, tag: &str) -> Vec<&Mission> {
        self.missions.values()
            .filter(|m| m.has_tag(tag))
            .collect()
    }
    
    /// Generate a new mission ID
    pub fn generate_mission_id(&mut self) -> String {
        self.mission_count += 1;
        format!("mission_{}", self.mission_count)
    }
    
    /// Generate a random mission
    pub fn generate_random_mission(&mut self, guild_id: &str, current_time: f64) -> Mission {
        let mut rng = thread_rng();
        
        // Generate random difficulty
        let difficulty = MissionDifficulty::random();
        
        // Generate mission name
        let adjectives = ["Dangerous", "Mysterious", "Urgent", "Secret", "Ancient", "Dark", "Lost", "Hidden", "Cursed"];
        let nouns = ["Quest", "Mission", "Task", "Journey", "Expedition", "Adventure", "Assignment", "Operation"];
        let name = format!("{} {}", adjectives[rng.gen_range(0..adjectives.len())], nouns[rng.gen_range(0..nouns.len())]);
        
        // Generate mission description
        let descriptions = [
            "A perilous journey awaits those brave enough to accept this mission.",
            "This mission requires skill and cunning to complete successfully.",
            "Only the most experienced adventurers should attempt this dangerous task.",
            "A simple mission that should pose no significant challenges.",
            "A mysterious client has requested this mission be completed with utmost discretion.",
        ];
        let description = descriptions[rng.gen_range(0..descriptions.len())].to_string();
        
        // Create mission
        let mission_id = self.generate_mission_id();
        let mut mission = Mission::new(mission_id, name, description, difficulty, guild_id.to_string(), current_time);
        
        // Add 1-3 objectives
        let objective_count = rng.gen_range(1..=3);
        for _ in 0..objective_count {
            let objective_type = MissionObjectiveType::random(difficulty);
            let objective = MissionObjective::new(objective_type);
            mission.add_objective(objective);
        }
        
        // Add 1-2 rewards
        let reward_count = rng.gen_range(1..=2);
        for _ in 0..reward_count {
            let reward = MissionReward::random(difficulty);
            mission.add_reward(reward);
        }
        
        // Add some tags
        let possible_tags = ["combat", "exploration", "stealth", "rescue", "collection", "boss", "timed"];
        let tag_count = rng.gen_range(1..=3);
        let mut selected_tags = std::collections::HashSet::new();
        
        while selected_tags.len() < tag_count {
            selected_tags.insert(possible_tags[rng.gen_range(0..possible_tags.len())]);
        }
        
        for tag in selected_tags {
            mission.add_tag(tag);
        }
        
        mission
    }
    
    /// Update mission statuses
    pub fn update_missions(&mut self, current_time: f64) {
        for mission in self.missions.values_mut() {
            mission.check_expiration(current_time);
            
            // Auto-complete missions if all objectives are completed
            if mission.status == MissionStatus::InProgress && mission.all_objectives_completed() {
                mission.complete(current_time);
            }
        }
    }
    
    /// Clean up expired missions
    pub fn clean_up_expired_missions(&mut self, days_to_keep: u32, current_time: f64) {
        let seconds_to_keep = days_to_keep as f64 * 24.0 * 60.0 * 60.0;
        let cutoff_time = current_time - seconds_to_keep;
        
        let expired_missions: Vec<String> = self.missions.iter()
            .filter(|(_, mission)| {
                mission.status == MissionStatus::Expired && 
                mission.creation_time < cutoff_time
            })
            .map(|(id, _)| id.clone())
            .collect();
        
        for id in expired_missions {
            self.missions.remove(&id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mission_board() {
        let mut board = MissionBoard::new();
        
        // Generate mission
        let mission = board.generate_random_mission("test_guild", 100.0);
        let mission_id = mission.id.clone();
        
        // Add to board
        board.add_mission(mission);
        
        // Retrieve mission
        let retrieved = board.get_mission(&mission_id);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, mission_id);
        
        // Get available missions
        let available = board.get_available_missions();
        assert_eq!(available.len(), 1);
        
        // Update missions
        board.update_missions(100.0 + 8.0 * 24.0 * 60.0 * 60.0); // 8 days later
        
        // Check if expired
        let expired = board.get_mission(&mission_id);
        assert!(expired.is_some());
        assert_eq!(expired.unwrap().status, MissionStatus::Expired);
        
        // Clean up expired
        board.clean_up_expired_missions(7, 100.0 + 8.0 * 24.0 * 60.0 * 60.0);
        
        // Should still be there (not old enough)
        assert!(board.get_mission(&mission_id).is_some());
        
        // Clean up with longer threshold
        board.clean_up_expired_missions(7, 100.0 + 40.0 * 24.0 * 60.0 * 60.0); // 40 days later
        
        // Should be gone now
        assert!(board.get_mission(&mission_id).is_none());
    }
}