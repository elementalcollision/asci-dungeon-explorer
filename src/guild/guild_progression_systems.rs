use bevy::prelude::*;
use crate::guild::guild_core::{Guild, GuildMember, GuildManager, GuildFacility};
use crate::guild::guild_progression::{GuildProgression, GuildPerk};
use crate::guild::mission::{Mission, MissionTracker};
use crate::guild::mission_board::MissionBoard;

/// System for initializing guild progression
pub fn guild_progression_init_system(
    mut commands: Commands,
    mut guild_manager: ResMut<GuildManager>,
    guild_query: Query<(Entity, &Guild), Without<GuildProgression>>,
) {
    for (entity, guild) in guild_query.iter() {
        // Create progression component for guild
        let progression = GuildProgression::new();
        commands.entity(entity).insert(progression);
    }
}

/// System for updating guild progression
pub fn guild_progression_update_system(
    mut guild_query: Query<(Entity, &mut Guild, &mut GuildProgression)>,
    time: Res<Time>,
) {
    let current_time = time.elapsed_seconds_f64();
    
    for (entity, mut guild, mut progression) in guild_query.iter_mut() {
        // Check milestones
        let completed_milestones = progression.check_milestones(&guild, current_time);
        
        // Apply milestone rewards
        for milestone in completed_milestones {
            progression.apply_milestone_rewards(milestone, &mut guild);
            info!("Guild completed milestone: {}", milestone.name);
        }
        
        // Check achievements
        let completed_achievements = progression.check_achievements(&guild, current_time);
        
        // Apply achievement rewards
        for achievement in completed_achievements {
            progression.apply_achievement_rewards(achievement, &mut guild);
            info!("Guild earned achievement: {}", achievement.name);
        }
    }
}

/// System for handling mission completion rewards
pub fn mission_completion_reward_system(
    mut guild_query: Query<(Entity, &mut Guild, &mut GuildProgression)>,
    mission_board: Res<MissionBoard>,
    mut mission_tracker_query: Query<(Entity, &mut MissionTracker, &GuildMember)>,
    time: Res<Time>,
) {
    let current_time = time.elapsed_seconds_f64();
    
    // Process completed missions
    for (entity, mut tracker, guild_member) in mission_tracker_query.iter_mut() {
        // Check for newly completed missions
        if let Some(mission_id) = tracker.complete_mission(current_time) {
            if let Some(mission) = mission_board.get_mission(&mission_id) {
                // Find the guild
                for (guild_entity, mut guild, mut progression) in guild_query.iter_mut() {
                    if guild.id == guild_member.guild_id {
                        // Award guild experience based on mission difficulty
                        let exp_reward = match mission.difficulty {
                            crate::guild::mission_types::MissionDifficulty::Trivial => 50,
                            crate::guild::mission_types::MissionDifficulty::Easy => 100,
                            crate::guild::mission_types::MissionDifficulty::Medium => 200,
                            crate::guild::mission_types::MissionDifficulty::Hard => 400,
                            crate::guild::mission_types::MissionDifficulty::VeryHard => 800,
                            crate::guild::mission_types::MissionDifficulty::Extreme => 1500,
                        };
                        
                        // Apply experience bonus from perks
                        let exp_bonus = if progression.has_perk(&GuildPerk::ExperienceBonus) {
                            progression.get_perk_effect_value(&GuildPerk::ExperienceBonus)
                        } else {
                            0.0
                        };
                        
                        let final_exp = (exp_reward as f32 * (1.0 + exp_bonus)) as u32;
                        
                        // Add experience and check for level up
                        if progression.add_experience(final_exp) {
                            info!("Guild {} leveled up to {}", guild.name, progression.level);
                        }
                        
                        // Add reputation
                        let rep_reward = match mission.difficulty {
                            crate::guild::mission_types::MissionDifficulty::Trivial => 5,
                            crate::guild::mission_types::MissionDifficulty::Easy => 10,
                            crate::guild::mission_types::MissionDifficulty::Medium => 20,
                            crate::guild::mission_types::MissionDifficulty::Hard => 40,
                            crate::guild::mission_types::MissionDifficulty::VeryHard => 80,
                            crate::guild::mission_types::MissionDifficulty::Extreme => 150,
                        };
                        
                        // Apply reputation bonus from perks
                        let rep_bonus = if progression.has_perk(&GuildPerk::ReputationBonus) {
                            progression.get_perk_effect_value(&GuildPerk::ReputationBonus)
                        } else {
                            0.0
                        };
                        
                        let final_rep = (rep_reward as f32 * (1.0 + rep_bonus)) as u32;
                        progression.add_reputation(final_rep, &mut guild);
                        
                        break;
                    }
                }
            }
        }
    }
}/// 
System for applying facility effects
pub fn facility_effects_system(
    mut guild_query: Query<(&mut Guild, &GuildProgression)>,
) {
    for (mut guild, progression) in guild_query.iter_mut() {
        // Apply effects from facilities
        for (facility, instance) in &guild.facilities {
            match facility {
                GuildFacility::Headquarters => {
                    // Headquarters increases member capacity
                    let base_capacity = 10;
                    let level_bonus = instance.level as u32 * 2;
                    
                    // Apply perks if available
                    let perk_bonus = if progression.has_perk(&GuildPerk::RecruitmentBonus) {
                        (base_capacity as f32 * progression.get_perk_effect_value(&GuildPerk::RecruitmentBonus)) as u32
                    } else {
                        0
                    };
                    
                    // Set max members
                    guild.max_members = base_capacity + level_bonus + perk_bonus;
                },
                GuildFacility::TrainingHall => {
                    // Training hall would affect agent training speed
                    // This would be handled in agent training systems
                },
                GuildFacility::Library => {
                    // Library increases experience gain
                    // This would be handled in experience gain systems
                },
                GuildFacility::Workshop => {
                    // Workshop affects crafting
                    // This would be handled in crafting systems
                },
                GuildFacility::Infirmary => {
                    // Infirmary affects healing rate
                    // This would be handled in healing systems
                },
                GuildFacility::Vault => {
                    // Vault increases resource capacity
                    let base_capacity = 1000;
                    let level_bonus = instance.level as u32 * 500;
                    
                    // Apply increased storage perk if available
                    let perk_bonus = if progression.has_perk(&GuildPerk::IncreasedStorage) {
                        (base_capacity as f32 * progression.get_perk_effect_value(&GuildPerk::IncreasedStorage)) as u32
                    } else {
                        0
                    };
                    
                    guild.resource_capacity = base_capacity + level_bonus + perk_bonus;
                },
                _ => {}
            }
        }
    }
}

/// System for applying guild perks
pub fn guild_perks_system(
    guild_query: Query<(&Guild, &GuildProgression)>,
    mut member_query: Query<(Entity, &mut GuildMember)>,
) {
    for (guild, progression) in guild_query.iter() {
        // Apply perks to guild members
        for (entity, mut member) in member_query.iter_mut() {
            if member.guild_id == guild.id {
                // Apply perks based on guild specialization
                match progression.specialization {
                    crate::guild::guild_progression::GuildSpecialization::Combat => {
                        // Combat specialization would boost combat stats
                        // This would be handled in combat systems
                    },
                    crate::guild::guild_progression::GuildSpecialization::Exploration => {
                        // Exploration specialization would boost exploration abilities
                        // This would be handled in exploration systems
                    },
                    crate::guild::guild_progression::GuildSpecialization::Crafting => {
                        // Crafting specialization would boost crafting abilities
                        // This would be handled in crafting systems
                    },
                    crate::guild::guild_progression::GuildSpecialization::Trading => {
                        // Trading specialization would boost trading abilities
                        // This would be handled in trading systems
                    },
                    crate::guild::guild_progression::GuildSpecialization::Research => {
                        // Research specialization would boost research abilities
                        // This would be handled in research systems
                    },
                    crate::guild::guild_progression::GuildSpecialization::Balanced => {
                        // Balanced specialization provides minor boosts to all areas
                        // This would be handled in various systems
                    },
                }
            }
        }
    }
}

/// System for resource production from facilities
pub fn resource_production_system(
    mut guild_query: Query<(&mut Guild, &GuildProgression)>,
    time: Res<Time>,
) {
    let delta_time = time.delta_seconds();
    
    for (mut guild, progression) in guild_query.iter_mut() {
        // Calculate resource production from facilities
        let mut gold_production = 0.0;
        let mut supplies_production = 0.0;
        let mut magic_essence_production = 0.0;
        
        for (facility, instance) in &guild.facilities {
            match facility {
                GuildFacility::Headquarters => {
                    // Headquarters produces a small amount of gold
                    gold_production += 0.1 * instance.level as f32;
                },
                GuildFacility::TrainingHall => {
                    // Training hall consumes supplies
                    supplies_production -= 0.2 * instance.level as f32;
                },
                GuildFacility::Library => {
                    // Library consumes gold but produces magic essence
                    gold_production -= 0.1 * instance.level as f32;
                    magic_essence_production += 0.05 * instance.level as f32;
                },
                GuildFacility::Workshop => {
                    // Workshop produces supplies
                    supplies_production += 0.3 * instance.level as f32;
                },
                GuildFacility::Infirmary => {
                    // Infirmary consumes supplies
                    supplies_production -= 0.1 * instance.level as f32;
                },
                GuildFacility::Vault => {
                    // Vault has no resource production/consumption
                },
                GuildFacility::Garden => {
                    // Garden produces supplies
                    supplies_production += 0.2 * instance.level as f32;
                },
                GuildFacility::Forge => {
                    // Forge consumes supplies but produces gold
                    supplies_production -= 0.3 * instance.level as f32;
                    gold_production += 0.2 * instance.level as f32;
                },
                _ => {}
            }
        }
        
        // Apply resource efficiency perk if available
        if progression.has_perk(&GuildPerk::ResourceEfficiency) {
            let efficiency = progression.get_perk_effect_value(&GuildPerk::ResourceEfficiency);
            
            // Reduce consumption (negative production)
            if gold_production < 0.0 {
                gold_production *= (1.0 - efficiency);
            }
            
            if supplies_production < 0.0 {
                supplies_production *= (1.0 - efficiency);
            }
            
            // Increase production (positive production)
            if gold_production > 0.0 {
                gold_production *= (1.0 + efficiency);
            }
            
            if supplies_production > 0.0 {
                supplies_production *= (1.0 + efficiency);
            }
            
            if magic_essence_production > 0.0 {
                magic_essence_production *= (1.0 + efficiency);
            }
        }
        
        // Apply production based on time
        let gold_delta = gold_production * delta_time;
        let supplies_delta = supplies_production * delta_time;
        let magic_essence_delta = magic_essence_production * delta_time;
        
        // Update resources
        if gold_delta != 0.0 {
            let current_gold = guild.resources.get(&crate::guild::guild_core::GuildResource::Gold).copied().unwrap_or(0);
            let new_gold = (current_gold as f32 + gold_delta).max(0.0) as u32;
            guild.resources.insert(crate::guild::guild_core::GuildResource::Gold, new_gold);
        }
        
        if supplies_delta != 0.0 {
            let current_supplies = guild.resources.get(&crate::guild::guild_core::GuildResource::Supplies).copied().unwrap_or(0);
            let new_supplies = (current_supplies as f32 + supplies_delta).max(0.0) as u32;
            guild.resources.insert(crate::guild::guild_core::GuildResource::Supplies, new_supplies);
        }
        
        if magic_essence_delta != 0.0 {
            let current_essence = guild.resources.get(&crate::guild::guild_core::GuildResource::MagicEssence).copied().unwrap_or(0);
            let new_essence = (current_essence as f32 + magic_essence_delta).max(0.0) as u32;
            guild.resources.insert(crate::guild::guild_core::GuildResource::MagicEssence, new_essence);
        }
    }
}/
// Plugin for guild progression systems
pub struct GuildProgressionPlugin;

impl Plugin for GuildProgressionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, guild_progression_init_system)
           .add_systems(Update, (
               guild_progression_update_system,
               mission_completion_reward_system,
               facility_effects_system,
               guild_perks_system,
               resource_production_system,
           ).chain());
    }
}