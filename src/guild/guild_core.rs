use bevy::prelude::*;
use serde::{Serialize, Deserialize};
use std::collections::{HashMap, HashSet};
use crate::components::{Name, Position};
use crate::items::Item;

/// Guild rank levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GuildRank {
    Recruit,
    Member,
    Veteran,
    Elite,
    Officer,
    Leader,
}

impl GuildRank {
    /// Get the numeric level of this rank
    pub fn level(&self) -> u32 {
        match self {
            GuildRank::Recruit => 0,
            GuildRank::Member => 1,
            GuildRank::Veteran => 2,
            GuildRank::Elite => 3,
            GuildRank::Officer => 4,
            GuildRank::Leader => 5,
        }
    }

    /// Get the name of this rank
    pub fn name(&self) -> &'static str {
        match self {
            GuildRank::Recruit => "Recruit",
            GuildRank::Member => "Member",
            GuildRank::Veteran => "Veteran",
            GuildRank::Elite => "Elite",
            GuildRank::Officer => "Officer",
            GuildRank::Leader => "Leader",
        }
    }

    /// Check if this rank can promote to the given rank
    pub fn can_promote_to(&self, target: GuildRank) -> bool {
        self.level() > target.level()
    }
}

/// Guild resource types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GuildResource {
    Gold,
    Reputation,
    Supplies,
    MagicEssence,
    RareArtifacts,
}

impl GuildResource {
    /// Get the name of this resource
    pub fn name(&self) -> &'static str {
        match self {
            GuildResource::Gold => "Gold",
            GuildResource::Reputation => "Reputation",
            GuildResource::Supplies => "Supplies",
            GuildResource::MagicEssence => "Magic Essence",
            GuildResource::RareArtifacts => "Rare Artifacts",
        }
    }

    /// Get all resource types
    pub fn all() -> Vec<GuildResource> {
        vec![
            GuildResource::Gold,
            GuildResource::Reputation,
            GuildResource::Supplies,
            GuildResource::MagicEssence,
            GuildResource::RareArtifacts,
        ]
    }
}

/// Guild member component
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct GuildMember {
    pub guild_id: String,
    pub rank: GuildRank,
    pub joined_date: f64,
    pub contribution: HashMap<GuildResource, u32>,
    pub missions_completed: u32,
    pub specialization: String,
    pub notes: String,
    pub is_active: bool,
}

impl Default for GuildMember {
    fn default() -> Self {
        GuildMember {
            guild_id: "".to_string(),
            rank: GuildRank::Recruit,
            joined_date: 0.0,
            contribution: HashMap::new(),
            missions_completed: 0,
            specialization: "None".to_string(),
            notes: "".to_string(),
            is_active: true,
        }
    }
}

impl GuildMember {
    /// Create a new guild member
    pub fn new(guild_id: String, joined_date: f64) -> Self {
        let mut member = GuildMember::default();
        member.guild_id = guild_id;
        member.joined_date = joined_date;
        member
    }

    /// Add contribution to the member's record
    pub fn add_contribution(&mut self, resource: GuildResource, amount: u32) {
        *self.contribution.entry(resource).or_insert(0) += amount;
    }

    /// Get total contribution for a resource
    pub fn get_contribution(&self, resource: GuildResource) -> u32 {
        *self.contribution.get(&resource).unwrap_or(&0)
    }

    /// Get total contribution across all resources
    pub fn get_total_contribution(&self) -> u32 {
        self.contribution.values().sum()
    }

    /// Promote member to a new rank
    pub fn promote(&mut self, new_rank: GuildRank) -> bool {
        if new_rank.level() > self.rank.level() {
            self.rank = new_rank;
            true
        } else {
            false
        }
    }

    /// Complete a mission
    pub fn complete_mission(&mut self) {
        self.missions_completed += 1;
    }
}

/// Guild facility types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GuildFacility {
    Headquarters,
    TrainingHall,
    Library,
    Workshop,
    Infirmary,
    MagicLab,
    Vault,
    Garden,
    Forge,
    TavernHall,
}

impl GuildFacility {
    /// Get the name of this facility
    pub fn name(&self) -> &'static str {
        match self {
            GuildFacility::Headquarters => "Headquarters",
            GuildFacility::TrainingHall => "Training Hall",
            GuildFacility::Library => "Library",
            GuildFacility::Workshop => "Workshop",
            GuildFacility::Infirmary => "Infirmary",
            GuildFacility::MagicLab => "Magic Laboratory",
            GuildFacility::Vault => "Vault",
            GuildFacility::Garden => "Garden",
            GuildFacility::Forge => "Forge",
            GuildFacility::TavernHall => "Tavern Hall",
        }
    }

    /// Get the description of this facility
    pub fn description(&self) -> &'static str {
        match self {
            GuildFacility::Headquarters => "Main guild building for administration",
            GuildFacility::TrainingHall => "Facility for training guild members",
            GuildFacility::Library => "Repository of knowledge and research",
            GuildFacility::Workshop => "Crafting and item creation facility",
            GuildFacility::Infirmary => "Medical facility for healing and recovery",
            GuildFacility::MagicLab => "Laboratory for magical research",
            GuildFacility::Vault => "Secure storage for valuable items",
            GuildFacility::Garden => "Garden for growing herbs and ingredients",
            GuildFacility::Forge => "Smithy for creating weapons and armor",
            GuildFacility::TavernHall => "Social gathering place for guild members",
        }
    }

    /// Get the cost to build this facility
    pub fn build_cost(&self) -> HashMap<GuildResource, u32> {
        let mut cost = HashMap::new();
        match self {
            GuildFacility::Headquarters => {
                cost.insert(GuildResource::Gold, 1000);
                cost.insert(GuildResource::Supplies, 500);
            }
            GuildFacility::TrainingHall => {
                cost.insert(GuildResource::Gold, 500);
                cost.insert(GuildResource::Supplies, 300);
            }
            GuildFacility::Library => {
                cost.insert(GuildResource::Gold, 400);
                cost.insert(GuildResource::Reputation, 100);
            }
            GuildFacility::Workshop => {
                cost.insert(GuildResource::Gold, 600);
                cost.insert(GuildResource::Supplies, 400);
            }
            GuildFacility::Infirmary => {
                cost.insert(GuildResource::Gold, 300);
                cost.insert(GuildResource::Supplies, 200);
                cost.insert(GuildResource::MagicEssence, 50);
            }
            GuildFacility::MagicLab => {
                cost.insert(GuildResource::Gold, 800);
                cost.insert(GuildResource::MagicEssence, 200);
            }
            GuildFacility::Vault => {
                cost.insert(GuildResource::Gold, 1200);
                cost.insert(GuildResource::RareArtifacts, 5);
            }
            GuildFacility::Garden => {
                cost.insert(GuildResource::Gold, 200);
                cost.insert(GuildResource::Supplies, 100);
            }
            GuildFacility::Forge => {
                cost.insert(GuildResource::Gold, 700);
                cost.insert(GuildResource::Supplies, 500);
                cost.insert(GuildResource::RareArtifacts, 2);
            }
            GuildFacility::TavernHall => {
                cost.insert(GuildResource::Gold, 400);
                cost.insert(GuildResource::Supplies, 200);
                cost.insert(GuildResource::Reputation, 50);
            }
        }
        cost
    }

    /// Get all facility types
    pub fn all() -> Vec<GuildFacility> {
        vec![
            GuildFacility::Headquarters,
            GuildFacility::TrainingHall,
            GuildFacility::Library,
            GuildFacility::Workshop,
            GuildFacility::Infirmary,
            GuildFacility::MagicLab,
            GuildFacility::Vault,
            GuildFacility::Garden,
            GuildFacility::Forge,
            GuildFacility::TavernHall,
        ]
    }
}

/// Guild facility instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuildFacilityInstance {
    pub facility_type: GuildFacility,
    pub level: u32,
    pub staff: HashSet<Entity>,
    pub upgrades: Vec<String>,
    pub position: Option<Vec2>,
}

impl GuildFacilityInstance {
    /// Create a new facility instance
    pub fn new(facility_type: GuildFacility) -> Self {
        GuildFacilityInstance {
            facility_type,
            level: 1,
            staff: HashSet::new(),
            upgrades: Vec::new(),
            position: None,
        }
    }

    /// Upgrade the facility
    pub fn upgrade(&mut self) -> bool {
        if self.level < 5 {
            self.level += 1;
            true
        } else {
            false
        }
    }

    /// Add staff to the facility
    pub fn add_staff(&mut self, entity: Entity) -> bool {
        self.staff.insert(entity)
    }

    /// Remove staff from the facility
    pub fn remove_staff(&mut self, entity: Entity) -> bool {
        self.staff.remove(&entity)
    }

    /// Add an upgrade to the facility
    pub fn add_upgrade(&mut self, upgrade: String) -> bool {
        if !self.upgrades.contains(&upgrade) {
            self.upgrades.push(upgrade);
            true
        } else {
            false
        }
    }

    /// Get the effectiveness of this facility based on level and upgrades
    pub fn effectiveness(&self) -> f32 {
        let base = self.level as f32;
        let upgrade_bonus = self.upgrades.len() as f32 * 0.2;
        base + upgrade_bonus
    }
}

/// Main guild structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Guild {
    pub id: String,
    pub name: String,
    pub founded_date: f64,
    pub leader: Option<Entity>,
    pub members: HashSet<Entity>,
    pub resources: HashMap<GuildResource, u32>,
    pub facilities: HashMap<GuildFacility, GuildFacilityInstance>,
    pub reputation: u32,
    pub level: u32,
    pub description: String,
    pub headquarters_position: Option<Vec2>,
    pub storage: Vec<Item>,
}

impl Guild {
    /// Create a new guild
    pub fn new(id: String, name: String, founded_date: f64) -> Self {
        let mut guild = Guild {
            id,
            name,
            founded_date,
            leader: None,
            members: HashSet::new(),
            resources: HashMap::new(),
            facilities: HashMap::new(),
            reputation: 0,
            level: 1,
            description: "".to_string(),
            headquarters_position: None,
            storage: Vec::new(),
        };

        // Initialize with headquarters facility
        guild.facilities.insert(
            GuildFacility::Headquarters,
            GuildFacilityInstance::new(GuildFacility::Headquarters),
        );

        // Initialize resources
        for resource in GuildResource::all() {
            guild.resources.insert(resource, 0);
        }

        guild
    }

    /// Add a member to the guild
    pub fn add_member(&mut self, entity: Entity, current_time: f64) -> bool {
        if self.members.insert(entity) {
            true
        } else {
            false
        }
    }

    /// Remove a member from the guild
    pub fn remove_member(&mut self, entity: Entity) -> bool {
        // Can't remove the leader
        if Some(entity) == self.leader {
            return false;
        }

        self.members.remove(&entity)
    }

    /// Set the guild leader
    pub fn set_leader(&mut self, entity: Entity) -> bool {
        if self.members.contains(&entity) {
            self.leader = Some(entity);
            true
        } else {
            false
        }
    }

    /// Add resources to the guild
    pub fn add_resource(&mut self, resource: GuildResource, amount: u32) {
        *self.resources.entry(resource).or_insert(0) += amount;
    }

    /// Remove resources from the guild
    pub fn remove_resource(&mut self, resource: GuildResource, amount: u32) -> bool {
        if let Some(current) = self.resources.get_mut(&resource) {
            if *current >= amount {
                *current -= amount;
                return true;
            }
        }
        false
    }

    /// Check if the guild has enough resources
    pub fn has_resources(&self, requirements: &HashMap<GuildResource, u32>) -> bool {
        for (resource, amount) in requirements {
            if self.resources.get(resource).unwrap_or(&0) < amount {
                return false;
            }
        }
        true
    }

    /// Build a new facility
    pub fn build_facility(&mut self, facility: GuildFacility) -> bool {
        if self.facilities.contains_key(&facility) {
            return false;
        }

        let cost = facility.build_cost();
        if !self.has_resources(&cost) {
            return false;
        }

        // Deduct resources
        for (resource, amount) in cost {
            self.remove_resource(resource, amount);
        }

        // Add facility
        self.facilities.insert(facility, GuildFacilityInstance::new(facility));
        true
    }

    /// Upgrade an existing facility
    pub fn upgrade_facility(&mut self, facility: GuildFacility) -> bool {
        if let Some(instance) = self.facilities.get_mut(&facility) {
            // Calculate upgrade cost (increases with level)
            let mut cost = facility.build_cost();
            let level_multiplier = instance.level as f32 * 0.5 + 1.0;
            for amount in cost.values_mut() {
                *amount = (*amount as f32 * level_multiplier) as u32;
            }

            if !self.has_resources(&cost) {
                return false;
            }

            // Deduct resources
            for (resource, amount) in cost {
                self.remove_resource(resource, amount);
            }

            // Upgrade facility
            instance.upgrade()
        } else {
            false
        }
    }

    /// Add an item to guild storage
    pub fn add_item(&mut self, item: Item) {
        self.storage.push(item);
    }

    /// Remove an item from guild storage by index
    pub fn remove_item(&mut self, index: usize) -> Option<Item> {
        if index < self.storage.len() {
            Some(self.storage.remove(index))
        } else {
            None
        }
    }

    /// Calculate guild level based on facilities and members
    pub fn calculate_level(&mut self) -> u32 {
        let facility_score = self.facilities.values()
            .map(|f| f.level)
            .sum::<u32>();
        
        let member_score = self.members.len() as u32;
        
        let reputation_score = self.reputation / 100;
        
        let level = (facility_score + member_score + reputation_score) / 10 + 1;
        self.level = level;
        level
    }

    /// Get all members with a specific rank
    pub fn get_members_with_rank(&self, rank: GuildRank, member_query: &Query<&GuildMember>) -> Vec<Entity> {
        self.members.iter()
            .filter(|&&entity| {
                if let Ok(member) = member_query.get(entity) {
                    member.rank == rank
                } else {
                    false
                }
            })
            .copied()
            .collect()
    }
}

/// Resource for managing all guilds
#[derive(Resource, Default)]
pub struct GuildManager {
    pub guilds: HashMap<String, Guild>,
    pub player_guild: Option<String>,
}

impl GuildManager {
    /// Create a new guild manager
    pub fn new() -> Self {
        GuildManager {
            guilds: HashMap::new(),
            player_guild: None,
        }
    }

    /// Create a new guild
    pub fn create_guild(&mut self, name: String, leader: Entity, current_time: f64) -> String {
        let id = format!("guild_{}", self.guilds.len());
        let mut guild = Guild::new(id.clone(), name, current_time);
        guild.add_member(leader, current_time);
        guild.set_leader(leader);
        self.guilds.insert(id.clone(), guild);
        id
    }

    /// Get a guild by ID
    pub fn get_guild(&self, id: &str) -> Option<&Guild> {
        self.guilds.get(id)
    }

    /// Get a mutable reference to a guild
    pub fn get_guild_mut(&mut self, id: &str) -> Option<&mut Guild> {
        self.guilds.get_mut(id)
    }

    /// Set the player's guild
    pub fn set_player_guild(&mut self, guild_id: String) {
        self.player_guild = Some(guild_id);
    }

    /// Get the player's guild
    pub fn get_player_guild(&self) -> Option<&Guild> {
        self.player_guild.as_ref().and_then(|id| self.guilds.get(id))
    }

    /// Get a mutable reference to the player's guild
    pub fn get_player_guild_mut(&mut self) -> Option<&mut Guild> {
        if let Some(id) = &self.player_guild {
            self.guilds.get_mut(id)
        } else {
            None
        }
    }
}

/// System for updating guild members
pub fn guild_member_update_system(
    mut member_query: Query<(Entity, &mut GuildMember)>,
    mut guild_manager: ResMut<GuildManager>,
) {
    for (entity, mut member) in member_query.iter_mut() {
        if member.guild_id.is_empty() {
            continue;
        }

        if let Some(guild) = guild_manager.get_guild_mut(&member.guild_id) {
            // Ensure member is in the guild
            if !guild.members.contains(&entity) {
                guild.members.insert(entity);
            }
        }
    }
}

/// System for calculating guild levels
pub fn guild_level_calculation_system(
    mut guild_manager: ResMut<GuildManager>,
) {
    for guild in guild_manager.guilds.values_mut() {
        guild.calculate_level();
    }
}

/// Plugin for guild core systems
pub struct GuildCorePlugin;

impl Plugin for GuildCorePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GuildManager>()
            .add_systems(Update, (
                guild_member_update_system,
                guild_level_calculation_system,
            ).chain());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_guild_rank() {
        assert!(GuildRank::Leader.level() > GuildRank::Member.level());
        assert!(GuildRank::Officer.can_promote_to(GuildRank::Member));
        assert!(!GuildRank::Member.can_promote_to(GuildRank::Officer));
    }

    #[test]
    fn test_guild_member() {
        let mut member = GuildMember::new("test_guild".to_string(), 100.0);
        assert_eq!(member.guild_id, "test_guild");
        assert_eq!(member.rank, GuildRank::Recruit);
        
        // Test contribution
        member.add_contribution(GuildResource::Gold, 100);
        assert_eq!(member.get_contribution(GuildResource::Gold), 100);
        assert_eq!(member.get_contribution(GuildResource::Reputation), 0);
        
        member.add_contribution(GuildResource::Gold, 50);
        assert_eq!(member.get_contribution(GuildResource::Gold), 150);
        
        // Test promotion
        assert!(member.promote(GuildRank::Member));
        assert_eq!(member.rank, GuildRank::Member);
        
        // Can't demote
        assert!(!member.promote(GuildRank::Recruit));
        assert_eq!(member.rank, GuildRank::Member);
        
        // Test mission completion
        assert_eq!(member.missions_completed, 0);
        member.complete_mission();
        assert_eq!(member.missions_completed, 1);
    }

    #[test]
    fn test_guild_facility() {
        let facility = GuildFacility::TrainingHall;
        assert_eq!(facility.name(), "Training Hall");
        
        let cost = facility.build_cost();
        assert!(cost.contains_key(&GuildResource::Gold));
        
        let mut instance = GuildFacilityInstance::new(facility);
        assert_eq!(instance.level, 1);
        
        assert!(instance.upgrade());
        assert_eq!(instance.level, 2);
        
        // Test staff management
        let entity = Entity::from_raw(1);
        assert!(instance.add_staff(entity));
        assert!(instance.staff.contains(&entity));
        
        assert!(instance.remove_staff(entity));
        assert!(!instance.staff.contains(&entity));
        
        // Test upgrades
        assert!(instance.add_upgrade("Enhanced Training".to_string()));
        assert_eq!(instance.upgrades.len(), 1);
        
        // Test effectiveness
        let base_effectiveness = instance.effectiveness();
        instance.add_upgrade("Advanced Equipment".to_string());
        assert!(instance.effectiveness() > base_effectiveness);
    }

    #[test]
    fn test_guild() {
        let mut guild = Guild::new("test".to_string(), "Test Guild".to_string(), 100.0);
        assert_eq!(guild.name, "Test Guild");
        assert_eq!(guild.level, 1);
        
        // Test member management
        let member_entity = Entity::from_raw(1);
        assert!(guild.add_member(member_entity, 100.0));
        assert!(guild.members.contains(&member_entity));
        
        assert!(guild.set_leader(member_entity));
        assert_eq!(guild.leader, Some(member_entity));
        
        // Can't remove leader
        assert!(!guild.remove_member(member_entity));
        
        // Test resource management
        guild.add_resource(GuildResource::Gold, 1000);
        assert_eq!(*guild.resources.get(&GuildResource::Gold).unwrap(), 1000);
        
        assert!(guild.remove_resource(GuildResource::Gold, 500));
        assert_eq!(*guild.resources.get(&GuildResource::Gold).unwrap(), 500);
        
        assert!(!guild.remove_resource(GuildResource::Gold, 1000));
        assert_eq!(*guild.resources.get(&GuildResource::Gold).unwrap(), 500);
        
        // Test facility management
        assert!(guild.facilities.contains_key(&GuildFacility::Headquarters));
        assert!(!guild.facilities.contains_key(&GuildFacility::TrainingHall));
        
        // Add resources for building
        for (resource, amount) in GuildFacility::TrainingHall.build_cost() {
            guild.add_resource(resource, amount);
        }
        
        assert!(guild.build_facility(GuildFacility::TrainingHall));
        assert!(guild.facilities.contains_key(&GuildFacility::TrainingHall));
        
        // Test item storage
        let item = Item::default(); // Assuming Item has a default implementation
        guild.add_item(item);
        assert_eq!(guild.storage.len(), 1);
        
        let removed = guild.remove_item(0);
        assert!(removed.is_some());
        assert_eq!(guild.storage.len(), 0);
    }

    #[test]
    fn test_guild_manager() {
        let mut manager = GuildManager::new();
        let leader = Entity::from_raw(1);
        
        let guild_id = manager.create_guild("Test Guild".to_string(), leader, 100.0);
        assert!(manager.guilds.contains_key(&guild_id));
        
        let guild = manager.get_guild(&guild_id).unwrap();
        assert_eq!(guild.name, "Test Guild");
        assert_eq!(guild.leader, Some(leader));
        
        manager.set_player_guild(guild_id.clone());
        assert_eq!(manager.player_guild, Some(guild_id));
        
        let player_guild = manager.get_player_guild().unwrap();
        assert_eq!(player_guild.name, "Test Guild");
    }
}