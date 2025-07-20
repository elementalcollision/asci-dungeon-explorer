use bevy::prelude::*;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use crate::guild::guild_core::GuildMember;
use crate::components::{CombatStats, Health, Name};

/// Agent stats component
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct AgentStats {
    pub level: u32,
    pub experience: u32,
    pub experience_to_next_level: u32,
    pub available_stat_points: u32,
    pub attributes: HashMap<String, u32>,
    pub skills: HashMap<String, u32>,
    pub specializations: Vec<String>,
    pub trait_bonuses: HashMap<String, i32>,
    pub missions_completed: u32,
    pub kills: u32,
    pub items_found: u32,
    pub damage_dealt: u32,
    pub damage_taken: u32,
    pub healing_done: u32,
    pub deaths: u32,
}

impl Default for AgentStats {
    fn default() -> Self {
        let mut attributes = HashMap::new();
        attributes.insert("Strength".to_string(), 10);
        attributes.insert("Dexterity".to_string(), 10);
        attributes.insert("Constitution".to_string(), 10);
        attributes.insert("Intelligence".to_string(), 10);
        attributes.insert("Wisdom".to_string(), 10);
        attributes.insert("Charisma".to_string(), 10);
        
        let mut skills = HashMap::new();
        skills.insert("Melee".to_string(), 1);
        skills.insert("Ranged".to_string(), 1);
        skills.insert("Magic".to_string(), 1);
        skills.insert("Stealth".to_string(), 1);
        skills.insert("Perception".to_string(), 1);
        skills.insert("Survival".to_string(), 1);
        skills.insert("Lockpicking".to_string(), 1);
        skills.insert("Alchemy".to_string(), 1);
        
        AgentStats {
            level: 1,
            experience: 0,
            experience_to_next_level: 100,
            available_stat_points: 0,
            attributes,
            skills,
            specializations: Vec::new(),
            trait_bonuses: HashMap::new(),
            missions_completed: 0,
            kills: 0,
            items_found: 0,
            damage_dealt: 0,
            damage_taken: 0,
            healing_done: 0,
            deaths: 0,
        }
    }
}

impl AgentStats {
    /// Create new agent stats with a specific role
    pub fn new(role: &str) -> Self {
        let mut stats = AgentStats::default();
        
        // Adjust attributes based on role
        match role {
            "Fighter" => {
                stats.attributes.insert("Strength".to_string(), 14);
                stats.attributes.insert("Constitution".to_string(), 12);
                stats.skills.insert("Melee".to_string(), 3);
                stats.specializations.push("Combat".to_string());
            },
            "Rogue" => {
                stats.attributes.insert("Dexterity".to_string(), 14);
                stats.attributes.insert("Charisma".to_string(), 12);
                stats.skills.insert("Stealth".to_string(), 3);
                stats.skills.insert("Lockpicking".to_string(), 3);
                stats.specializations.push("Infiltration".to_string());
            },
            "Mage" => {
                stats.attributes.insert("Intelligence".to_string(), 14);
                stats.attributes.insert("Wisdom".to_string(), 12);
                stats.skills.insert("Magic".to_string(), 3);
                stats.specializations.push("Spellcasting".to_string());
            },
            "Ranger" => {
                stats.attributes.insert("Dexterity".to_string(), 14);
                stats.attributes.insert("Wisdom".to_string(), 12);
                stats.skills.insert("Ranged".to_string(), 3);
                stats.skills.insert("Survival".to_string(), 3);
                stats.specializations.push("Exploration".to_string());
            },
            "Cleric" => {
                stats.attributes.insert("Wisdom".to_string(), 14);
                stats.attributes.insert("Constitution".to_string(), 12);
                stats.skills.insert("Magic".to_string(), 2);
                stats.specializations.push("Healing".to_string());
            },
            _ => {
                // Default balanced stats
            }
        }
        
        stats
    }
    
    /// Add experience and handle level ups
    pub fn add_experience(&mut self, amount: u32) -> bool {
        self.experience += amount;
        
        // Check for level up
        if self.experience >= self.experience_to_next_level {
            self.level_up();
            return true;
        }
        
        false
    }
    
    /// Level up the agent
    pub fn level_up(&mut self) {
        self.level += 1;
        self.available_stat_points += 2;
        
        // Calculate new experience threshold (exponential growth)
        self.experience_to_next_level = 100 * (self.level as u32).pow(2);
        
        // Every few levels, gain a skill point in primary skills
        if self.level % 3 == 0 {
            for specialization in &self.specializations {
                match specialization.as_str() {
                    "Combat" => {
                        *self.skills.entry("Melee".to_string()).or_insert(1) += 1;
                    },
                    "Infiltration" => {
                        *self.skills.entry("Stealth".to_string()).or_insert(1) += 1;
                    },
                    "Spellcasting" => {
                        *self.skills.entry("Magic".to_string()).or_insert(1) += 1;
                    },
                    "Exploration" => {
                        *self.skills.entry("Survival".to_string()).or_insert(1) += 1;
                    },
                    "Healing" => {
                        *self.skills.entry("Alchemy".to_string()).or_insert(1) += 1;
                    },
                    _ => {}
                }
            }
        }
    }
    
    /// Spend stat points to increase an attribute
    pub fn increase_attribute(&mut self, attribute: &str, amount: u32) -> bool {
        if self.available_stat_points < amount {
            return false;
        }
        
        if let Some(value) = self.attributes.get_mut(attribute) {
            *value += amount;
            self.available_stat_points -= amount;
            return true;
        }
        
        false
    }
    
    /// Increase a skill
    pub fn increase_skill(&mut self, skill: &str, amount: u32) -> bool {
        if self.available_stat_points < amount {
            return false;
        }
        
        if let Some(value) = self.skills.get_mut(skill) {
            *value += amount;
            self.available_stat_points -= amount;
            return true;
        }
        
        false
    }
    
    /// Add a specialization
    pub fn add_specialization(&mut self, specialization: &str) -> bool {
        if !self.specializations.contains(&specialization.to_string()) {
            self.specializations.push(specialization.to_string());
            return true;
        }
        
        false
    }
    
    /// Get attribute value with bonuses
    pub fn get_attribute(&self, attribute: &str) -> u32 {
        let base = self.attributes.get(attribute).copied().unwrap_or(10);
        let bonus = self.trait_bonuses.get(attribute).copied().unwrap_or(0);
        
        (base as i32 + bonus) as u32
    }
    
    /// Get skill value with bonuses
    pub fn get_skill(&self, skill: &str) -> u32 {
        let base = self.skills.get(skill).copied().unwrap_or(0);
        let bonus = self.trait_bonuses.get(skill).copied().unwrap_or(0);
        
        (base as i32 + bonus) as u32
    }
    
    /// Record a completed mission
    pub fn complete_mission(&mut self) {
        self.missions_completed += 1;
        
        // Award experience based on mission completion
        self.add_experience(50 * self.level);
    }
    
    /// Record a kill
    pub fn record_kill(&mut self) {
        self.kills += 1;
        self.add_experience(10);
    }
    
    /// Record found item
    pub fn record_item_found(&mut self) {
        self.items_found += 1;
        self.add_experience(5);
    }
    
    /// Record damage dealt
    pub fn record_damage_dealt(&mut self, amount: u32) {
        self.damage_dealt += amount;
        self.add_experience(amount / 10);
    }
    
    /// Record damage taken
    pub fn record_damage_taken(&mut self, amount: u32) {
        self.damage_taken += amount;
    }
    
    /// Record healing done
    pub fn record_healing_done(&mut self, amount: u32) {
        self.healing_done += amount;
        self.add_experience(amount / 5);
    }
    
    /// Record a death
    pub fn record_death(&mut self) {
        self.deaths += 1;
    }
}

/// Agent progression component
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct AgentProgression {
    pub role: String,
    pub career_path: Vec<String>,
    pub milestones: Vec<String>,
    pub training_history: Vec<String>,
    pub current_training: Option<String>,
    pub training_progress: f32,
    pub training_completion_time: f64,
    pub mentor: Option<Entity>,
    pub apprentice: Option<Entity>,
    pub reputation: i32,
    pub notable_achievements: Vec<String>,
    pub promotion_eligibility: bool,
}

impl Default for AgentProgression {
    fn default() -> Self {
        AgentProgression {
            role: "Recruit".to_string(),
            career_path: Vec::new(),
            milestones: Vec::new(),
            training_history: Vec::new(),
            current_training: None,
            training_progress: 0.0,
            training_completion_time: 0.0,
            mentor: None,
            apprentice: None,
            reputation: 0,
            notable_achievements: Vec::new(),
            promotion_eligibility: false,
        }
    }
}

impl AgentProgression {
    /// Create new progression for a role
    pub fn new(role: &str) -> Self {
        let mut progression = AgentProgression::default();
        progression.role = role.to_string();
        progression.career_path.push(role.to_string());
        progression
    }
    
    /// Start training in a skill
    pub fn start_training(&mut self, skill: &str, completion_time: f64) -> bool {
        if self.current_training.is_some() {
            return false;
        }
        
        self.current_training = Some(skill.to_string());
        self.training_progress = 0.0;
        self.training_completion_time = completion_time;
        true
    }
    
    /// Update training progress
    pub fn update_training(&mut self, current_time: f64) -> bool {
        if let Some(skill) = &self.current_training {
            if current_time >= self.training_completion_time {
                // Training complete
                self.training_history.push(skill.clone());
                self.current_training = None;
                self.training_progress = 0.0;
                return true;
            } else {
                // Update progress
                let total_time = self.training_completion_time - (current_time - self.training_progress);
                let elapsed = current_time - (current_time - self.training_progress);
                self.training_progress = (elapsed / total_time) as f32;
            }
        }
        
        false
    }
    
    /// Add a milestone
    pub fn add_milestone(&mut self, milestone: &str) {
        if !self.milestones.contains(&milestone.to_string()) {
            self.milestones.push(milestone.to_string());
            
            // Check for promotion eligibility
            self.check_promotion_eligibility();
        }
    }
    
    /// Add a notable achievement
    pub fn add_achievement(&mut self, achievement: &str) {
        if !self.notable_achievements.contains(&achievement.to_string()) {
            self.notable_achievements.push(achievement.to_string());
            self.reputation += 10;
            
            // Check for promotion eligibility
            self.check_promotion_eligibility();
        }
    }
    
    /// Set mentor
    pub fn set_mentor(&mut self, mentor: Entity) {
        self.mentor = Some(mentor);
    }
    
    /// Set apprentice
    pub fn set_apprentice(&mut self, apprentice: Entity) {
        self.apprentice = Some(apprentice);
    }
    
    /// Promote to a new role
    pub fn promote(&mut self, new_role: &str) {
        self.role = new_role.to_string();
        self.career_path.push(new_role.to_string());
        self.promotion_eligibility = false;
        
        // Add promotion milestone
        self.add_milestone(&format!("Promoted to {}", new_role));
    }
    
    /// Check if eligible for promotion
    fn check_promotion_eligibility(&mut self) {
        // Simple promotion logic based on milestones and achievements
        let milestone_count = self.milestones.len();
        let achievement_count = self.notable_achievements.len();
        
        match self.role.as_str() {
            "Recruit" => {
                self.promotion_eligibility = milestone_count >= 3;
            },
            "Member" => {
                self.promotion_eligibility = milestone_count >= 5 && achievement_count >= 2;
            },
            "Veteran" => {
                self.promotion_eligibility = milestone_count >= 10 && achievement_count >= 5 && self.reputation >= 50;
            },
            "Elite" => {
                self.promotion_eligibility = milestone_count >= 15 && achievement_count >= 10 && self.reputation >= 100;
            },
            _ => {
                self.promotion_eligibility = false;
            }
        }
    }
}

/// System for updating agent stats based on combat stats
pub fn agent_stats_update_system(
    mut agent_query: Query<(&mut AgentStats, &CombatStats, &Health)>,
) {
    for (mut agent_stats, combat_stats, health) in agent_query.iter_mut() {
        // Update health percentage for decision making
        let health_percentage = health.current as f32 / health.max as f32;
        
        // Record damage taken if health decreased
        if health.current < health.max {
            let damage_taken = health.max - health.current;
            agent_stats.record_damage_taken(damage_taken as u32);
        }
        
        // In a real implementation, you would track damage dealt through combat events
    }
}

/// System for updating agent progression
pub fn agent_progression_update_system(
    time: Res<Time>,
    mut agent_query: Query<(&mut AgentProgression, &AgentStats, &GuildMember)>,
) {
    let current_time = time.elapsed_seconds_f64();
    
    for (mut progression, stats, guild_member) in agent_query.iter_mut() {
        // Update training progress
        progression.update_training(current_time);
        
        // Check for milestones based on stats
        if stats.level >= 5 && !progression.milestones.contains(&"Reached Level 5".to_string()) {
            progression.add_milestone("Reached Level 5");
        }
        
        if stats.level >= 10 && !progression.milestones.contains(&"Reached Level 10".to_string()) {
            progression.add_milestone("Reached Level 10");
        }
        
        if stats.missions_completed >= 10 && !progression.milestones.contains(&"Completed 10 Missions".to_string()) {
            progression.add_milestone("Completed 10 Missions");
        }
        
        // Check for achievements
        if stats.kills >= 100 && !progression.notable_achievements.contains(&"Slayer".to_string()) {
            progression.add_achievement("Slayer");
        }
        
        if stats.items_found >= 50 && !progression.notable_achievements.contains(&"Treasure Hunter".to_string()) {
            progression.add_achievement("Treasure Hunter");
        }
        
        // Sync with guild member data
        if guild_member.missions_completed > stats.missions_completed {
            for _ in 0..(guild_member.missions_completed - stats.missions_completed) {
                stats.complete_mission();
            }
        }
    }
}

/// System for applying agent stats to combat stats
pub fn agent_stats_to_combat_stats_system(
    mut query: Query<(&AgentStats, &mut CombatStats)>,
) {
    for (agent_stats, mut combat_stats) in query.iter_mut() {
        // Calculate base stats from attributes
        let strength = agent_stats.get_attribute("Strength");
        let constitution = agent_stats.get_attribute("Constitution");
        let dexterity = agent_stats.get_attribute("Dexterity");
        
        // Apply to combat stats
        combat_stats.max_hp = 50 + (constitution * 5) as i32;
        combat_stats.hp = combat_stats.max_hp; // Only set on initialization
        combat_stats.power = 5 + (strength / 2) as i32;
        combat_stats.defense = 2 + (constitution / 3) as i32;
        
        // Apply skill bonuses
        let melee_skill = agent_stats.get_skill("Melee");
        combat_stats.power += melee_skill as i32;
        
        // Level-based scaling
        let level_bonus = (agent_stats.level - 1) as i32;
        combat_stats.max_hp += level_bonus * 3;
        combat_stats.power += level_bonus / 2;
        combat_stats.defense += level_bonus / 3;
    }
}

/// Plugin for agent progression systems
pub struct AgentProgressionPlugin;

impl Plugin for AgentProgressionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            agent_stats_update_system,
            agent_progression_update_system,
            agent_stats_to_combat_stats_system,
        ).chain());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_stats_creation() {
        let fighter_stats = AgentStats::new("Fighter");
        assert_eq!(fighter_stats.get_attribute("Strength"), 14);
        assert_eq!(fighter_stats.get_skill("Melee"), 3);
        assert!(fighter_stats.specializations.contains(&"Combat".to_string()));
        
        let mage_stats = AgentStats::new("Mage");
        assert_eq!(mage_stats.get_attribute("Intelligence"), 14);
        assert_eq!(mage_stats.get_skill("Magic"), 3);
    }

    #[test]
    fn test_experience_and_leveling() {
        let mut stats = AgentStats::default();
        assert_eq!(stats.level, 1);
        
        // Add experience but not enough to level
        let leveled_up = stats.add_experience(50);
        assert!(!leveled_up);
        assert_eq!(stats.level, 1);
        assert_eq!(stats.experience, 50);
        
        // Add enough experience to level up
        let leveled_up = stats.add_experience(50);
        assert!(leveled_up);
        assert_eq!(stats.level, 2);
        assert_eq!(stats.available_stat_points, 2);
        
        // Test spending stat points
        let success = stats.increase_attribute("Strength", 1);
        assert!(success);
        assert_eq!(stats.get_attribute("Strength"), 11);
        assert_eq!(stats.available_stat_points, 1);
    }

    #[test]
    fn test_agent_progression() {
        let mut progression = AgentProgression::new("Fighter");
        assert_eq!(progression.role, "Fighter");
        
        // Test milestones
        progression.add_milestone("First Mission Complete");
        progression.add_milestone("Defeated Boss");
        progression.add_milestone("Explored Dungeon");
        
        // Should be eligible for promotion from Recruit
        assert!(progression.promotion_eligibility);
        
        // Test promotion
        progression.promote("Member");
        assert_eq!(progression.role, "Member");
        assert!(!progression.promotion_eligibility); // Reset after promotion
        assert_eq!(progression.career_path.len(), 2);
        
        // Test training
        let start_time = 100.0;
        let completion_time = 200.0;
        let success = progression.start_training("Advanced Combat", completion_time);
        assert!(success);
        assert_eq!(progression.current_training, Some("Advanced Combat".to_string()));
        
        // Training not complete yet
        let completed = progression.update_training(150.0);
        assert!(!completed);
        
        // Training complete
        let completed = progression.update_training(200.0);
        assert!(completed);
        assert_eq!(progression.current_training, None);
        assert!(progression.training_history.contains(&"Advanced Combat".to_string()));
    }
}