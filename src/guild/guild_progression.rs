use bevy::prelude::*;
use serde::{Serialize, Deserialize};
use std::collections::{HashMap, HashSet};
use crate::guild::guild_core::{Guild, GuildMember, GuildResource, GuildFacility, GuildFacilityInstance};

/// Guild progression component
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct GuildProgression {
    pub level: u32,
    pub experience: u32,
    pub experience_to_next_level: u32,
    pub reputation_level: u32,
    pub milestones: Vec<GuildMilestone>,
    pub achievements: Vec<GuildAchievement>,
    pub unlocked_facilities: HashSet<GuildFacility>,
    pub facility_upgrade_paths: HashMap<GuildFacility, Vec<FacilityUpgrade>>,
    pub available_upgrades: Vec<GuildUpgrade>,
    pub applied_upgrades: Vec<GuildUpgrade>,
    pub specialization: GuildSpecialization,
    pub perks: HashSet<GuildPerk>,
}

impl Default for GuildProgression {
    fn default() -> Self {
        GuildProgression {
            level: 1,
            experience: 0,
            experience_to_next_level: 1000,
            reputation_level: 1,
            milestones: Vec::new(),
            achievements: Vec::new(),
            unlocked_facilities: HashSet::new(),
            facility_upgrade_paths: HashMap::new(),
            available_upgrades: Vec::new(),
            applied_upgrades: Vec::new(),
            specialization: GuildSpecialization::Balanced,
            perks: HashSet::new(),
        }
    }
}
//
/ Guild milestone
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuildMilestone {
    pub id: String,
    pub name: String,
    pub description: String,
    pub requirements: Vec<MilestoneRequirement>,
    pub rewards: Vec<MilestoneReward>,
    pub is_completed: bool,
    pub completion_date: Option<f64>,
}

/// Milestone requirement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MilestoneRequirement {
    GuildLevel(u32),
    MembersCount(u32),
    CompletedMissions(u32),
    ReputationLevel(u32),
    FacilityLevel(GuildFacility, u32),
    ResourceAmount(GuildResource, u32),
    SpecificAchievement(String),
}

/// Milestone reward
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MilestoneReward {
    Experience(u32),
    Reputation(u32),
    Resources(GuildResource, u32),
    UnlockFacility(GuildFacility),
    UnlockUpgrade(GuildUpgrade),
    UnlockPerk(GuildPerk),
}

/// Guild achievement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuildAchievement {
    pub id: String,
    pub name: String,
    pub description: String,
    pub is_secret: bool,
    pub is_completed: bool,
    pub completion_date: Option<f64>,
    pub progress: Option<(u32, u32)>, // current, total
    pub rewards: Vec<MilestoneReward>,
}

/// Facility upgrade
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FacilityUpgrade {
    pub name: String,
    pub description: String,
    pub level_requirement: u32,
    pub cost: HashMap<GuildResource, u32>,
    pub effects: Vec<UpgradeEffect>,
    pub is_applied: bool,
}

/// Guild upgrade
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct GuildUpgrade {
    pub id: String,
    pub name: String,
    pub description: String,
    pub level_requirement: u32,
    pub reputation_requirement: u32,
    pub cost: HashMap<GuildResource, u32>,
    pub effects: Vec<UpgradeEffect>,
    pub is_applied: bool,
}

/// Upgrade effect
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum UpgradeEffect {
    IncreaseStorage(GuildResource, u32),
    IncreaseProduction(GuildResource, f32),
    ReduceCost(GuildResource, f32),
    IncreaseEffectiveness(GuildFacility, f32),
    UnlockRecipe(String),
    UnlockMission(String),
    UnlockArea(String),
    IncreaseAgentStats(String, f32),
    ReduceMissionTime(f32),
    IncreaseMissionRewards(f32),
    CustomEffect(String),
}

/// Guild specialization
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GuildSpecialization {
    Balanced,
    Combat,
    Exploration,
    Crafting,
    Trading,
    Research,
}

/// Guild perk
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GuildPerk {
    IncreasedStorage,
    EnhancedTraining,
    ImprovedCrafting,
    BetterMissions,
    ResourceEfficiency,
    ReputationBonus,
    ExperienceBonus,
    RecruitmentBonus,
    FastTravel,
    MarketAccess,
    RareItemChance,
    CustomPerk(String),
}
impl
 GuildProgression {
    /// Create a new guild progression
    pub fn new() -> Self {
        let mut progression = GuildProgression::default();
        
        // Initialize with basic unlocked facilities
        progression.unlocked_facilities.insert(GuildFacility::Headquarters);
        progression.unlocked_facilities.insert(GuildFacility::TrainingHall);
        
        // Initialize facility upgrade paths
        progression.init_facility_upgrade_paths();
        
        // Initialize available upgrades
        progression.init_available_upgrades();
        
        // Initialize milestones
        progression.init_milestones();
        
        progression
    }
    
    /// Initialize facility upgrade paths
    fn init_facility_upgrade_paths(&mut self) {
        // Headquarters upgrades
        let hq_upgrades = vec![
            FacilityUpgrade {
                name: "Expanded Quarters".to_string(),
                description: "Increases member capacity by 5".to_string(),
                level_requirement: 2,
                cost: [(GuildResource::Gold, 800), (GuildResource::Supplies, 400)].iter().cloned().collect(),
                effects: vec![UpgradeEffect::CustomEffect("Increase max members by 5".to_string())],
                is_applied: false,
            },
            FacilityUpgrade {
                name: "Guild Banner".to_string(),
                description: "Increases reputation gain by 10%".to_string(),
                level_requirement: 3,
                cost: [(GuildResource::Gold, 1200), (GuildResource::Supplies, 300)].iter().cloned().collect(),
                effects: vec![UpgradeEffect::CustomEffect("Increase reputation gain by 10%".to_string())],
                is_applied: false,
            },
        ];
        
        // Training Hall upgrades
        let training_upgrades = vec![
            FacilityUpgrade {
                name: "Improved Training Dummies".to_string(),
                description: "Enhances training effectiveness by 10%".to_string(),
                level_requirement: 2,
                cost: [(GuildResource::Gold, 500), (GuildResource::Supplies, 200)].iter().cloned().collect(),
                effects: vec![UpgradeEffect::IncreaseEffectiveness(GuildFacility::TrainingHall, 0.1)],
                is_applied: false,
            },
            FacilityUpgrade {
                name: "Advanced Combat Training".to_string(),
                description: "Increases combat stats for all members by 5%".to_string(),
                level_requirement: 3,
                cost: [(GuildResource::Gold, 1000), (GuildResource::Supplies, 400)].iter().cloned().collect(),
                effects: vec![UpgradeEffect::IncreaseAgentStats("Combat".to_string(), 0.05)],
                is_applied: false,
            },
        ];
        
        // Add upgrade paths
        self.facility_upgrade_paths.insert(GuildFacility::Headquarters, hq_upgrades);
        self.facility_upgrade_paths.insert(GuildFacility::TrainingHall, training_upgrades);
    }
    
    /// Initialize available upgrades
    fn init_available_upgrades(&mut self) {
        self.available_upgrades = vec![
            GuildUpgrade {
                id: "guild_hall_expansion".to_string(),
                name: "Guild Hall Expansion".to_string(),
                description: "Expands the guild hall, allowing more members".to_string(),
                level_requirement: 2,
                reputation_requirement: 1,
                cost: [(GuildResource::Gold, 1000), (GuildResource::Supplies, 500)].iter().cloned().collect(),
                effects: vec![UpgradeEffect::CustomEffect("Increase max members by 5".to_string())],
                is_applied: false,
            },
            GuildUpgrade {
                id: "mission_board_upgrade".to_string(),
                name: "Mission Board Upgrade".to_string(),
                description: "Improves mission rewards by 10%".to_string(),
                level_requirement: 3,
                reputation_requirement: 2,
                cost: [(GuildResource::Gold, 1500), (GuildResource::Supplies, 300)].iter().cloned().collect(),
                effects: vec![UpgradeEffect::IncreaseMissionRewards(0.1)],
                is_applied: false,
            },
        ];
    }
    
    /// Initialize milestones
    fn init_milestones(&mut self) {
        self.milestones = vec![
            GuildMilestone {
                id: "first_steps".to_string(),
                name: "First Steps".to_string(),
                description: "Establish your guild and recruit your first members".to_string(),
                requirements: vec![
                    MilestoneRequirement::MembersCount(3),
                ],
                rewards: vec![
                    MilestoneReward::Experience(500),
                    MilestoneReward::Resources(GuildResource::Gold, 300),
                ],
                is_completed: false,
                completion_date: None,
            },
            GuildMilestone {
                id: "growing_reputation".to_string(),
                name: "Growing Reputation".to_string(),
                description: "Complete missions and gain recognition".to_string(),
                requirements: vec![
                    MilestoneRequirement::CompletedMissions(5),
                    MilestoneRequirement::ReputationLevel(2),
                ],
                rewards: vec![
                    MilestoneReward::Experience(1000),
                    MilestoneReward::Reputation(100),
                    MilestoneReward::UnlockFacility(GuildFacility::Library),
                ],
                is_completed: false,
                completion_date: None,
            },
        ];
    }    

    /// Add experience to the guild
    pub fn add_experience(&mut self, amount: u32) -> bool {
        self.experience += amount;
        
        // Check for level up
        if self.experience >= self.experience_to_next_level {
            self.level_up();
            return true;
        }
        
        false
    }
    
    /// Level up the guild
    pub fn level_up(&mut self) {
        self.level += 1;
        
        // Calculate new experience threshold (exponential growth)
        self.experience_to_next_level = 1000 * self.level * self.level;
        
        // Unlock new facilities based on level
        self.unlock_facilities_for_level();
        
        // Update available upgrades
        self.update_available_upgrades();
    }
    
    /// Unlock facilities for current level
    fn unlock_facilities_for_level(&mut self) {
        match self.level {
            2 => {
                self.unlocked_facilities.insert(GuildFacility::Infirmary);
            },
            3 => {
                self.unlocked_facilities.insert(GuildFacility::Garden);
            },
            4 => {
                self.unlocked_facilities.insert(GuildFacility::Forge);
            },
            5 => {
                self.unlocked_facilities.insert(GuildFacility::Vault);
            },
            _ => {}
        }
    }
    
    /// Update available upgrades based on current level
    fn update_available_upgrades(&mut self) {
        // Add level-specific upgrades
        match self.level {
            3 => {
                self.available_upgrades.push(GuildUpgrade {
                    id: "advanced_training".to_string(),
                    name: "Advanced Training Program".to_string(),
                    description: "Increases experience gain for all members by 15%".to_string(),
                    level_requirement: 3,
                    reputation_requirement: 2,
                    cost: [(GuildResource::Gold, 2000), (GuildResource::Supplies, 500)].iter().cloned().collect(),
                    effects: vec![UpgradeEffect::IncreaseAgentStats("Experience".to_string(), 0.15)],
                    is_applied: false,
                });
            },
            5 => {
                self.available_upgrades.push(GuildUpgrade {
                    id: "guild_specialization".to_string(),
                    name: "Guild Specialization".to_string(),
                    description: "Choose a specialization for your guild".to_string(),
                    level_requirement: 5,
                    reputation_requirement: 3,
                    cost: [(GuildResource::Gold, 5000), (GuildResource::Supplies, 1000), (GuildResource::RareArtifacts, 5)].iter().cloned().collect(),
                    effects: vec![UpgradeEffect::CustomEffect("Unlock guild specialization".to_string())],
                    is_applied: false,
                });
            },
            _ => {}
        }
    }
    
    /// Apply an upgrade
    pub fn apply_upgrade(&mut self, upgrade_id: &str, guild: &mut Guild) -> Result<(), String> {
        // Find the upgrade
        let upgrade_index = self.available_upgrades.iter().position(|u| u.id == upgrade_id)
            .ok_or_else(|| format!("Upgrade {} not found", upgrade_id))?;
        
        let upgrade = &self.available_upgrades[upgrade_index];
        
        // Check requirements
        if self.level < upgrade.level_requirement {
            return Err(format!("Guild level {} required (current: {})", upgrade.level_requirement, self.level));
        }
        
        if self.reputation_level < upgrade.reputation_requirement {
            return Err(format!("Guild reputation level {} required (current: {})", upgrade.reputation_requirement, self.reputation_level));
        }
        
        // Check resources
        for (resource, amount) in &upgrade.cost {
            let current = guild.resources.get(resource).copied().unwrap_or(0);
            if current < *amount {
                return Err(format!("Not enough {}: {} required (current: {})", resource.name(), amount, current));
            }
        }
        
        // Deduct resources
        for (resource, amount) in &upgrade.cost {
            guild.remove_resource(*resource, *amount);
        }
        
        // Apply effects
        for effect in &upgrade.effects {
            apply_upgrade_effect(effect, guild);
        }
        
        // Mark as applied and move to applied upgrades
        let mut upgrade = self.available_upgrades.remove(upgrade_index);
        upgrade.is_applied = true;
        self.applied_upgrades.push(upgrade);
        
        Ok(())
    }    

    /// Apply a facility upgrade
    pub fn apply_facility_upgrade(&mut self, facility: GuildFacility, upgrade_index: usize, guild: &mut Guild) -> Result<(), String> {
        // Find the facility upgrade path
        let upgrade_path = self.facility_upgrade_paths.get_mut(&facility)
            .ok_or_else(|| format!("No upgrade path for facility {:?}", facility))?;
        
        if upgrade_index >= upgrade_path.len() {
            return Err(format!("Upgrade index {} out of bounds for facility {:?}", upgrade_index, facility));
        }
        
        let upgrade = &upgrade_path[upgrade_index];
        
        // Check requirements
        if self.level < upgrade.level_requirement {
            return Err(format!("Guild level {} required (current: {})", upgrade.level_requirement, self.level));
        }
        
        // Check if facility exists
        let facility_instance = guild.facilities.get(&facility)
            .ok_or_else(|| format!("Facility {:?} not built", facility))?;
        
        // Check resources
        for (resource, amount) in &upgrade.cost {
            let current = guild.resources.get(resource).copied().unwrap_or(0);
            if current < *amount {
                return Err(format!("Not enough {}: {} required (current: {})", resource.name(), amount, current));
            }
        }
        
        // Deduct resources
        for (resource, amount) in &upgrade.cost {
            guild.remove_resource(*resource, *amount);
        }
        
        // Apply effects
        for effect in &upgrade.effects {
            apply_upgrade_effect(effect, guild);
        }
        
        // Mark as applied
        upgrade_path[upgrade_index].is_applied = true;
        
        Ok(())
    }
    
    /// Set guild specialization
    pub fn set_specialization(&mut self, specialization: GuildSpecialization) {
        self.specialization = specialization;
        
        // Add specialization-specific perks
        match specialization {
            GuildSpecialization::Combat => {
                self.perks.insert(GuildPerk::EnhancedTraining);
            },
            GuildSpecialization::Exploration => {
                self.perks.insert(GuildPerk::FastTravel);
            },
            GuildSpecialization::Crafting => {
                self.perks.insert(GuildPerk::ImprovedCrafting);
            },
            GuildSpecialization::Trading => {
                self.perks.insert(GuildPerk::MarketAccess);
            },
            GuildSpecialization::Research => {
                self.perks.insert(GuildPerk::ExperienceBonus);
            },
            GuildSpecialization::Balanced => {
                self.perks.insert(GuildPerk::ResourceEfficiency);
            },
        }
    }
    
    /// Add reputation
    pub fn add_reputation(&mut self, amount: u32, guild: &mut Guild) {
        guild.reputation += amount;
        
        // Check for reputation level up
        let new_level = calculate_reputation_level(guild.reputation);
        if new_level > self.reputation_level {
            self.reputation_level = new_level;
            self.on_reputation_level_up(guild);
        }
    }
    
    /// Handle reputation level up
    fn on_reputation_level_up(&mut self, guild: &mut Guild) {
        // Add perks based on reputation level
        match self.reputation_level {
            2 => {
                self.perks.insert(GuildPerk::ReputationBonus);
            },
            3 => {
                self.perks.insert(GuildPerk::RecruitmentBonus);
            },
            4 => {
                self.perks.insert(GuildPerk::BetterMissions);
            },
            5 => {
                self.perks.insert(GuildPerk::RareItemChance);
            },
            _ => {}
        }
        
        // Update available upgrades
        self.update_available_upgrades();
    }    

    /// Check milestone completion
    pub fn check_milestones(&mut self, guild: &Guild, current_time: f64) -> Vec<&GuildMilestone> {
        let mut completed_milestones = Vec::new();
        
        for milestone in &mut self.milestones {
            if milestone.is_completed {
                continue;
            }
            
            let mut all_requirements_met = true;
            
            for requirement in &milestone.requirements {
                if !check_milestone_requirement(requirement, guild, self) {
                    all_requirements_met = false;
                    break;
                }
            }
            
            if all_requirements_met {
                milestone.is_completed = true;
                milestone.completion_date = Some(current_time);
                completed_milestones.push(milestone);
            }
        }
        
        completed_milestones
    }
    
    /// Apply milestone rewards
    pub fn apply_milestone_rewards(&mut self, milestone: &GuildMilestone, guild: &mut Guild) {
        for reward in &milestone.rewards {
            match reward {
                MilestoneReward::Experience(amount) => {
                    self.add_experience(*amount);
                },
                MilestoneReward::Reputation(amount) => {
                    self.add_reputation(*amount, guild);
                },
                MilestoneReward::Resources(resource, amount) => {
                    guild.add_resource(*resource, *amount);
                },
                MilestoneReward::UnlockFacility(facility) => {
                    self.unlocked_facilities.insert(*facility);
                },
                MilestoneReward::UnlockUpgrade(upgrade) => {
                    if !self.available_upgrades.iter().any(|u| u.id == upgrade.id) {
                        self.available_upgrades.push(upgrade.clone());
                    }
                },
                MilestoneReward::UnlockPerk(perk) => {
                    self.perks.insert(perk.clone());
                },
            }
        }
    }
    
    /// Check achievement completion
    pub fn check_achievements(&mut self, guild: &Guild, current_time: f64) -> Vec<&GuildAchievement> {
        let mut completed_achievements = Vec::new();
        
        // In a real implementation, you would check specific achievement conditions
        // For now, we'll just return an empty vector
        
        completed_achievements
    }
    
    /// Apply achievement rewards
    pub fn apply_achievement_rewards(&mut self, achievement: &GuildAchievement, guild: &mut Guild) {
        for reward in &achievement.rewards {
            match reward {
                MilestoneReward::Experience(amount) => {
                    self.add_experience(*amount);
                },
                MilestoneReward::Reputation(amount) => {
                    self.add_reputation(*amount, guild);
                },
                MilestoneReward::Resources(resource, amount) => {
                    guild.add_resource(*resource, *amount);
                },
                MilestoneReward::UnlockFacility(facility) => {
                    self.unlocked_facilities.insert(*facility);
                },
                MilestoneReward::UnlockUpgrade(upgrade) => {
                    if !self.available_upgrades.iter().any(|u| u.id == upgrade.id) {
                        self.available_upgrades.push(upgrade.clone());
                    }
                },
                MilestoneReward::UnlockPerk(perk) => {
                    self.perks.insert(perk.clone());
                },
            }
        }
    }    

    /// Get available facility upgrades
    pub fn get_available_facility_upgrades(&self, facility: GuildFacility) -> Vec<&FacilityUpgrade> {
        if let Some(upgrades) = self.facility_upgrade_paths.get(&facility) {
            upgrades.iter().filter(|u| !u.is_applied && u.level_requirement <= self.level).collect()
        } else {
            Vec::new()
        }
    }
    
    /// Get available guild upgrades
    pub fn get_available_guild_upgrades(&self) -> Vec<&GuildUpgrade> {
        self.available_upgrades.iter()
            .filter(|u| !u.is_applied && u.level_requirement <= self.level && u.reputation_requirement <= self.reputation_level)
            .collect()
    }
    
    /// Check if a facility is unlocked
    pub fn is_facility_unlocked(&self, facility: GuildFacility) -> bool {
        self.unlocked_facilities.contains(&facility)
    }
    
    /// Check if a perk is active
    pub fn has_perk(&self, perk: &GuildPerk) -> bool {
        self.perks.contains(perk)
    }
    
    /// Get perk effect value
    pub fn get_perk_effect_value(&self, perk: &GuildPerk) -> f32 {
        if !self.has_perk(perk) {
            return 0.0;
        }
        
        match perk {
            GuildPerk::IncreasedStorage => 0.2,
            GuildPerk::EnhancedTraining => 0.15,
            GuildPerk::ImprovedCrafting => 0.25,
            GuildPerk::BetterMissions => 0.2,
            GuildPerk::ResourceEfficiency => 0.1,
            GuildPerk::ReputationBonus => 0.15,
            GuildPerk::ExperienceBonus => 0.2,
            GuildPerk::RecruitmentBonus => 0.25,
            GuildPerk::FastTravel => 0.3,
            GuildPerk::MarketAccess => 0.15,
            GuildPerk::RareItemChance => 0.1,
            GuildPerk::CustomPerk(_) => 0.1,
        }
    }
}

/// Apply upgrade effect to guild
fn apply_upgrade_effect(effect: &UpgradeEffect, guild: &mut Guild) {
    match effect {
        UpgradeEffect::IncreaseStorage(resource, amount) => {
            // In a real implementation, you would increase storage capacity
            // For now, we'll just add the resources
            guild.add_resource(*resource, *amount);
        },
        UpgradeEffect::IncreaseProduction(resource, factor) => {
            // Would be handled by resource production system
        },
        UpgradeEffect::ReduceCost(resource, factor) => {
            // Would be handled by cost calculation system
        },
        UpgradeEffect::IncreaseEffectiveness(facility, factor) => {
            if let Some(instance) = guild.facilities.get_mut(facility) {
                // In a real implementation, you would have an effectiveness field
                // For now, we'll just add an upgrade to the facility
                instance.add_upgrade(format!("Effectiveness +{:.0}%", factor * 100.0));
            }
        },
        UpgradeEffect::UnlockRecipe(recipe) => {
            // Would be handled by crafting system
        },
        UpgradeEffect::UnlockMission(mission) => {
            // Would be handled by mission system
        },
        UpgradeEffect::UnlockArea(area) => {
            // Would be handled by world system
        },
        UpgradeEffect::IncreaseAgentStats(stat, factor) => {
            // Would be handled by agent stats system
        },
        UpgradeEffect::ReduceMissionTime(factor) => {
            // Would be handled by mission system
        },
        UpgradeEffect::IncreaseMissionRewards(factor) => {
            // Would be handled by mission reward system
        },
        UpgradeEffect::CustomEffect(_) => {
            // Custom effects would be handled separately
        },
    }
}
/// 
Check if a milestone requirement is met
fn check_milestone_requirement(requirement: &MilestoneRequirement, guild: &Guild, progression: &GuildProgression) -> bool {
    match requirement {
        MilestoneRequirement::GuildLevel(level) => {
            progression.level >= *level
        },
        MilestoneRequirement::MembersCount(count) => {
            guild.members.len() as u32 >= *count
        },
        MilestoneRequirement::CompletedMissions(count) => {
            // In a real implementation, you would track completed missions
            // For now, we'll just use the member count as a proxy
            guild.members.len() as u32 >= *count
        },
        MilestoneRequirement::ReputationLevel(level) => {
            progression.reputation_level >= *level
        },
        MilestoneRequirement::FacilityLevel(facility, level) => {
            if let Some(instance) = guild.facilities.get(facility) {
                instance.level >= *level
            } else {
                false
            }
        },
        MilestoneRequirement::ResourceAmount(resource, amount) => {
            guild.resources.get(resource).copied().unwrap_or(0) >= *amount
        },
        MilestoneRequirement::SpecificAchievement(achievement_id) => {
            progression.achievements.iter().any(|a| a.id == *achievement_id && a.is_completed)
        },
    }
}

/// Calculate reputation level from reputation points
fn calculate_reputation_level(reputation: u32) -> u32 {
    if reputation < 100 {
        1
    } else if reputation < 500 {
        2
    } else if reputation < 1500 {
        3
    } else if reputation < 5000 {
        4
    } else {
        5
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_guild_progression_creation() {
        let progression = GuildProgression::new();
        assert_eq!(progression.level, 1);
        assert_eq!(progression.experience, 0);
        assert!(progression.unlocked_facilities.contains(&GuildFacility::Headquarters));
        assert!(progression.unlocked_facilities.contains(&GuildFacility::TrainingHall));
    }

    #[test]
    fn test_guild_level_up() {
        let mut progression = GuildProgression::new();
        let initial_exp_to_next = progression.experience_to_next_level;
        
        // Add enough experience to level up
        let leveled_up = progression.add_experience(initial_exp_to_next);
        
        assert!(leveled_up);
        assert_eq!(progression.level, 2);
        assert!(progression.experience_to_next_level > initial_exp_to_next);
        assert!(progression.unlocked_facilities.contains(&GuildFacility::Infirmary));
    }

    #[test]
    fn test_reputation_level() {
        assert_eq!(calculate_reputation_level(50), 1);
        assert_eq!(calculate_reputation_level(100), 2);
        assert_eq!(calculate_reputation_level(1000), 3);
        assert_eq!(calculate_reputation_level(2000), 4);
        assert_eq!(calculate_reputation_level(5000), 5);
    }

    #[test]
    fn test_milestone_requirements() {
        let mut guild = Guild::new("test".to_string(), "Test Guild".to_string(), 0.0);
        guild.resources.insert(GuildResource::Gold, 1000);
        
        let mut progression = GuildProgression::new();
        progression.level = 3;
        progression.reputation_level = 2;
        
        assert!(check_milestone_requirement(&MilestoneRequirement::GuildLevel(3), &guild, &progression));
        assert!(check_milestone_requirement(&MilestoneRequirement::ResourceAmount(GuildResource::Gold, 1000), &guild, &progression));
        assert!(!check_milestone_requirement(&MilestoneRequirement::MembersCount(10), &guild, &progression));
    }
}