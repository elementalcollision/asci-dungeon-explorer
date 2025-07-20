use bevy::prelude::*;
use crate::guild::asynchronous_exploration::{
    AsyncExplorationManager, AsyncExplorationState, AsyncExpedition, AsyncEvent, 
    AsyncEventType, ExpeditionRewardType
};
use crate::guild::guild_core::{Guild, GuildMember, GuildManager, GuildResource};
use crate::guild::mission::{Mission, MissionTracker};
use crate::guild::mission_board::MissionBoard;
use crate::guild::agent_progression::AgentStats;

/// System for updating asynchronous exploration
pub fn async_exploration_update_system(
    mut async_manager: ResMut<AsyncExplorationManager>,
    time: Res<Time>,
) {
    let current_time = time.elapsed_seconds_f64();
    let delta_time = time.delta_seconds_f64();
    
    async_manager.update(current_time, delta_time);
}

/// System for auto-assigning missions to available agents
pub fn auto_mission_assignment_system(
    mut async_manager: ResMut<AsyncExplorationManager>,
    mission_board: Res<MissionBoard>,
    guild_manager: Res<GuildManager>,
    agent_query: Query<(Entity, &GuildMember, &AgentStats, Option<&MissionTracker>)>,
    time: Res<Time>,
) {
    if !async_manager.auto_assign_missions || async_manager.state != AsyncExplorationState::Active {
        return;
    }
    
    let current_time = time.elapsed_seconds_f64();
    
    // Find available missions and agents
    for guild in guild_manager.guilds.values() {
        let available_missions = mission_board.get_missions_by_guild(&guild.id)
            .into_iter()
            .filter(|m| m.status == crate::guild::mission_types::MissionStatus::Available)
            .collect::<Vec<_>>();
        
        if available_missions.is_empty() {
            continue;
        }
        
        // Find available agents in this guild
        let mut available_agents: Vec<(Entity, &GuildMember, &AgentStats)> = Vec::new();
        
        for (entity, member, stats, tracker) in agent_query.iter() {
            if member.guild_id == guild.id && 
               guild.members.contains(&entity) &&
               (tracker.is_none() || tracker.unwrap().active_mission.is_none()) {
                
                // Check if agent is not already on an expedition
                let on_expedition = async_manager.active_expeditions.values()
                    .any(|exp| exp.assigned_agents.contains(&entity));
                
                if !on_expedition {
                    available_agents.push((entity, member, stats));
                }
            }
        }
        
        // Try to assign missions
        for mission in available_missions {
            if available_agents.is_empty() {
                break;
            }
            
            // Determine how many agents to assign (1-3 based on mission difficulty)
            let agent_count = match mission.difficulty {
                crate::guild::mission_types::MissionDifficulty::Trivial => 1,
                crate::guild::mission_types::MissionDifficulty::Easy => 1,
                crate::guild::mission_types::MissionDifficulty::Medium => 2,
                crate::guild::mission_types::MissionDifficulty::Hard => 2,
                crate::guild::mission_types::MissionDifficulty::VeryHard => 3,
                crate::guild::mission_types::MissionDifficulty::Extreme => 3,
            }.min(available_agents.len());
            
            if agent_count == 0 {
                continue;
            }
            
            // Select best agents for this mission
            let mut selected_agents = Vec::new();
            
            // Sort agents by level (higher level first)
            available_agents.sort_by(|a, b| b.2.level.cmp(&a.2.level));
            
            for i in 0..agent_count {
                selected_agents.push(available_agents[i].0);
            }
            
            // Remove selected agents from available list
            available_agents.retain(|(entity, _, _)| !selected_agents.contains(entity));
            
            // Create expedition
            if let Ok(expedition_id) = async_manager.create_expedition(mission, selected_agents, current_time) {
                info!("Auto-assigned mission '{}' to expedition '{}'", mission.name, expedition_id);
            }
        }
    }
}

/// System for processing asynchronous events
pub fn async_event_processing_system(
    mut async_manager: ResMut<AsyncExplorationManager>,
    mut guild_manager: ResMut<GuildManager>,
    mut agent_query: Query<(Entity, &mut AgentStats, &mut MissionTracker)>,
    mut commands: Commands,
) {
    // Process pending events
    while let Some(event) = async_manager.pop_event() {
        match event.event_type {
            AsyncEventType::ExpeditionStarted => {
                info!("Expedition started: {:?}", event.expedition_id);
            },
            AsyncEventType::ExpeditionCompleted => {
                if let Some(expedition_id) = &event.expedition_id {
                    process_expedition_completion(&mut async_manager, &mut guild_manager, &mut agent_query, expedition_id);
                }
            },
            AsyncEventType::ExpeditionFailed => {
                if let Some(expedition_id) = &event.expedition_id {
                    process_expedition_failure(&mut async_manager, &mut agent_query, expedition_id);
                }
            },
            AsyncEventType::AgentLevelUp => {
                // Handle agent level up
                info!("Agent leveled up during expedition");
            },
            AsyncEventType::ResourceDiscovered => {
                // Handle resource discovery
                info!("Resources discovered during expedition");
            },
            _ => {
                // Handle other event types
                info!("Async event: {:?}", event.event_type);
            }
        }
    }
}

/// Process expedition completion
fn process_expedition_completion(
    async_manager: &mut AsyncExplorationManager,
    guild_manager: &mut GuildManager,
    agent_query: &mut Query<(Entity, &mut AgentStats, &mut MissionTracker)>,
    expedition_id: &str,
) {
    // Find the completed expedition
    let expedition = async_manager.completed_expeditions.iter()
        .find(|exp| exp.id == expedition_id);
    
    if let Some(expedition) = expedition {
        // Apply rewards
        for reward in &expedition.rewards {
            match &reward.reward_type {
                ExpeditionRewardType::Experience => {
                    if let Some(recipient) = reward.recipient {
                        if let Ok((_, mut stats, _)) = agent_query.get_mut(recipient) {
                            stats.add_experience(reward.amount);
                        }
                    }
                },
                ExpeditionRewardType::Gold => {
                    // Add gold to guild (find guild by checking agent membership)
                    if let Some(&first_agent) = expedition.assigned_agents.first() {
                        if let Ok((_, _, _)) = agent_query.get(first_agent) {
                            // In a real implementation, you would find the guild through the agent
                            // For now, we'll add to the first guild
                            if let Some(guild) = guild_manager.guilds.values_mut().next() {
                                guild.add_resource(GuildResource::Gold, reward.amount);
                            }
                        }
                    }
                },
                ExpeditionRewardType::GuildResource(resource) => {
                    // Add resource to guild
                    if let Some(&first_agent) = expedition.assigned_agents.first() {
                        if let Ok((_, _, _)) = agent_query.get(first_agent) {
                            if let Some(guild) = guild_manager.guilds.values_mut().next() {
                                guild.add_resource(*resource, reward.amount);
                            }
                        }
                    }
                },
                ExpeditionRewardType::Items => {
                    // Add items to recipient or guild storage
                    if let Some(recipient) = reward.recipient {
                        // In a real implementation, you would add items to agent inventory
                        info!("Agent received {} items", reward.amount);
                    }
                },
                ExpeditionRewardType::Reputation => {
                    // Add reputation to guild
                    if let Some(&first_agent) = expedition.assigned_agents.first() {
                        if let Ok((_, _, _)) = agent_query.get(first_agent) {
                            if let Some(guild) = guild_manager.guilds.values_mut().next() {
                                guild.reputation += reward.amount;
                            }
                        }
                    }
                },
                ExpeditionRewardType::SkillPoints => {
                    if let Some(recipient) = reward.recipient {
                        if let Ok((_, mut stats, _)) = agent_query.get_mut(recipient) {
                            stats.available_stat_points += reward.amount;
                        }
                    }
                },
            }
        }
        
        // Update mission trackers
        for &agent in &expedition.assigned_agents {
            if let Ok((_, _, mut tracker)) = agent_query.get_mut(agent) {
                tracker.complete_mission(0.0); // Current time would be passed in real implementation
            }
        }
        
        info!("Expedition '{}' completed successfully with {} rewards", expedition_id, expedition.rewards.len());
    }
}

/// Process expedition failure
fn process_expedition_failure(
    async_manager: &mut AsyncExplorationManager,
    agent_query: &mut Query<(Entity, &mut AgentStats, &mut MissionTracker)>,
    expedition_id: &str,
) {
    // Find the failed expedition
    let expedition = async_manager.completed_expeditions.iter()
        .find(|exp| exp.id == expedition_id);
    
    if let Some(expedition) = expedition {
        // Update mission trackers to reflect failure
        for &agent in &expedition.assigned_agents {
            if let Ok((_, _, mut tracker)) = agent_query.get_mut(agent) {
                tracker.fail_mission(0.0); // Current time would be passed in real implementation
            }
        }
        
        // Apply any casualties or negative effects
        for &casualty in &expedition.casualties {
            if let Ok((_, mut stats, _)) = agent_query.get_mut(casualty) {
                // In a real implementation, you might apply injury effects
                info!("Agent was injured during failed expedition");
            }
        }
        
        info!("Expedition '{}' failed", expedition_id);
    }
}

/// System for calculating offline progress
pub fn offline_progress_system(
    mut async_manager: ResMut<AsyncExplorationManager>,
    time: Res<Time>,
) {
    let current_time = time.elapsed_seconds_f64();
    
    // Check if we need to calculate offline progress
    if let Some(last_offline_time) = async_manager.last_offline_time {
        let offline_duration = current_time - last_offline_time;
        
        // Only calculate if offline for more than 5 minutes
        if offline_duration > 300.0 {
            let offline_events = async_manager.calculate_offline_progress(offline_duration, current_time);
            
            if !offline_events.is_empty() {
                info!("Calculated offline progress: {} events generated", offline_events.len());
                
                // Add events back to queue for processing
                for event in offline_events {
                    async_manager.event_queue.push_back(event);
                }
            }
        }
    }
    
    async_manager.last_offline_time = Some(current_time);
}

/// System for expedition timeout handling
pub fn expedition_timeout_system(
    mut async_manager: ResMut<AsyncExplorationManager>,
    time: Res<Time>,
) {
    let current_time = time.elapsed_seconds_f64();
    
    // Check for expeditions that have taken too long
    let mut timed_out_expeditions = Vec::new();
    
    for (expedition_id, expedition) in &async_manager.active_expeditions {
        let elapsed_time = current_time - expedition.start_time;
        let timeout_threshold = expedition.estimated_duration * 2.0; // 2x estimated duration
        
        if elapsed_time > timeout_threshold {
            timed_out_expeditions.push(expedition_id.clone());
        }
    }
    
    // Handle timed out expeditions
    for expedition_id in timed_out_expeditions {
        if let Some(mut expedition) = async_manager.active_expeditions.remove(&expedition_id) {
            expedition.state = crate::guild::asynchronous_exploration::ExpeditionState::Failed;
            expedition.actual_duration = Some(current_time - expedition.start_time);
            
            // Add failure event
            async_manager.event_queue.push_back(AsyncEvent {
                event_type: AsyncEventType::ExpeditionFailed,
                timestamp: current_time,
                expedition_id: Some(expedition_id.clone()),
                data: std::collections::HashMap::new(),
            });
            
            async_manager.completed_expeditions.push(expedition);
            warn!("Expedition '{}' timed out and was marked as failed", expedition_id);
        }
    }
}

/// Plugin for asynchronous exploration systems
pub struct AsyncExplorationPlugin;

impl Plugin for AsyncExplorationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AsyncExplorationManager>()
           .add_systems(Update, (
               async_exploration_update_system,
               auto_mission_assignment_system,
               async_event_processing_system,
               offline_progress_system,
               expedition_timeout_system,
           ).chain());
    }
}