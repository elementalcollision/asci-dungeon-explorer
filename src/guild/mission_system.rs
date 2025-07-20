use bevy::prelude::*;
use crate::guild::mission_board::MissionBoard;
use crate::guild::mission::*;
use crate::guild::guild_core::{Guild, GuildMember};

/// System for generating missions
pub fn mission_generation_system(
    mut mission_board: ResMut<MissionBoard>,
    time: Res<Time>,
    guild_query: Query<&Guild>,
) {
    let current_time = time.elapsed_seconds_f64();
    
    // Generate missions for each guild
    for guild in guild_query.iter() {
        // In a real implementation, you would control mission generation rate
        // For now, we'll just generate a few missions for testing
        
        // Generate 1-3 random missions
        let mut rng = rand::thread_rng();
        let mission_count = rng.gen_range(1..=3);
        
        for _ in 0..mission_count {
            let mission = mission_board.generate_random_mission(&guild.id, current_time);
            mission_board.add_mission(mission);
        }
    }
}

/// System for updating mission objectives
pub fn mission_objective_update_system(
    mut mission_board: ResMut<MissionBoard>,
    mut guild_member_query: Query<(&GuildMember, &mut MissionTracker)>,
    time: Res<Time>,
) {
    let current_time = time.elapsed_seconds_f64();
    
    // Update mission statuses
    mission_board.update_missions(current_time);
    
    // Clean up old expired missions (keep for 30 days)
    mission_board.clean_up_expired_missions(30, current_time);
    
    // Update agent mission trackers
    for (guild_member, mut mission_tracker) in guild_member_query.iter_mut() {
        if let Some(mission_id) = &mission_tracker.active_mission {
            if let Some(mission) = mission_board.get_mission(mission_id) {
                // Sync mission progress with tracker
                let mut objective_statuses = Vec::new();
                for objective in &mission.objectives {
                    objective_statuses.push(objective.status.clone());
                }
                
                mission_tracker.mission_progress.insert(mission_id.clone(), objective_statuses);
                
                // Check for mission completion or failure
                match mission.status {
                    MissionStatus::Completed => {
                        mission_tracker.complete_mission(current_time);
                    },
                    MissionStatus::Failed | MissionStatus::Expired => {
                        mission_tracker.fail_mission(current_time);
                    },
                    _ => {}
                }
            }
        }
    }
}

/// System for handling mission rewards
pub fn mission_reward_system(
    mut mission_board: ResMut<MissionBoard>,
    mut guild_query: Query<&mut Guild>,
    mut guild_member_query: Query<(Entity, &GuildMember, &mut MissionTracker)>,
) {
    // Process completed missions that haven't been rewarded yet
    for (entity, guild_member, mut mission_tracker) in guild_member_query.iter_mut() {
        if let Some(mission_id) = mission_tracker.complete_mission(0.0) { // 0.0 is a placeholder, we're just checking
            if let Some(mission) = mission_board.get_mission(&mission_id) {
                // Find the guild
                for mut guild in guild_query.iter_mut() {
                    if guild.id == mission.guild_id {
                        // Apply rewards
                        for reward in &mission.rewards {
                            match reward {
                                MissionReward::Resources { resource_type, amount } => {
                                    guild.add_resource(*resource_type, *amount);
                                },
                                MissionReward::Items { items } => {
                                    for item in items {
                                        guild.add_item(item.clone());
                                    }
                                },
                                MissionReward::Experience { amount } => {
                                    // In a real implementation, you would add experience to the agent
                                },
                                MissionReward::Reputation { amount } => {
                                    guild.reputation = (guild.reputation as i32 + amount).max(0) as u32;
                                },
                                MissionReward::UnlockFacility { facility_name: _ } => {
                                    // In a real implementation, you would unlock the facility
                                },
                                MissionReward::UnlockArea { area_name: _ } => {
                                    // In a real implementation, you would unlock the area
                                },
                                MissionReward::Custom { description: _, value: _ } => {
                                    // Handle custom rewards
                                },
                            }
                        }
                        
                        // Update guild member's completed missions count
                        if let Some(guild_member_entity) = guild.members.get(&entity) {
                            if let Ok((_, guild_member, _)) = guild_member_query.get(*guild_member_entity) {
                                // In a real implementation, you would update the guild member's stats
                            }
                        }
                        
                        break;
                    }
                }
            }
        }
    }
}

/// System for assigning missions to agents
pub fn mission_assignment_system(
    mut commands: Commands,
    mut mission_board: ResMut<MissionBoard>,
    guild_query: Query<&Guild>,
    guild_member_query: Query<(Entity, &GuildMember, Option<&MissionTracker>)>,
    time: Res<Time>,
) {
    let current_time = time.elapsed_seconds_f64();
    
    // Process each guild
    for guild in guild_query.iter() {
        // Get available missions for this guild
        let available_missions: Vec<&Mission> = mission_board.missions.values()
            .filter(|m| m.guild_id == guild.id && m.status == MissionStatus::Available)
            .collect();
        
        // Get guild members without active missions
        let available_members: Vec<(Entity, &GuildMember)> = guild_member_query.iter()
            .filter(|(entity, member, tracker)| {
                member.guild_id == guild.id && 
                guild.members.contains(entity) && 
                (tracker.is_none() || tracker.unwrap().active_mission.is_none())
            })
            .map(|(entity, member, _)| (entity, member))
            .collect();
        
        // Assign missions to available members
        for (entity, member) in available_members {
            if available_missions.is_empty() {
                break;
            }
            
            // Find a suitable mission based on member's level/experience
            // In a real implementation, you would match missions to member capabilities
            if let Some(mission) = available_missions.first() {
                if let Some(mut mission) = mission_board.get_mission_mut(&mission.id) {
                    // Assign mission to member
                    mission.assign_agent(entity);
                    mission.start(current_time);
                    
                    // Create or update mission tracker for the member
                    if let Ok((_, _, Some(tracker))) = guild_member_query.get(entity) {
                        let mut new_tracker = tracker.clone();
                        new_tracker.start_mission(&mission.id, current_time);
                        commands.entity(entity).insert(new_tracker);
                    } else {
                        let mut new_tracker = MissionTracker::default();
                        new_tracker.start_mission(&mission.id, current_time);
                        commands.entity(entity).insert(new_tracker);
                    }
                }
            }
        }
    }
}

/// Plugin for mission systems
pub struct MissionSystemPlugin;

impl Plugin for MissionSystemPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MissionBoard>()
           .add_systems(Update, (
               mission_generation_system,
               mission_objective_update_system,
               mission_reward_system,
               mission_assignment_system,
           ).chain());
    }
}