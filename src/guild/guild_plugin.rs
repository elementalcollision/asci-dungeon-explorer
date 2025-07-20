use bevy::prelude::*;
use crate::guild::*;

/// Main plugin for the guild system
pub struct GuildPlugin;

impl Plugin for GuildPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            GuildCorePlugin,
            GuildPersistencePlugin,
            GuildResourcePlugin,
            GuildProgressionPlugin,
            GuildProgressionUIPlugin,
            AgentBehaviorPlugin,
            AgentDecisionPlugin,
            AgentProgressionPlugin,
            AgentEquipmentPlugin,
            MissionSystemPlugin,
            WorldInstancePlugin,
            GuildUIPlugin,
            SyncExplorationPlugin,
            SyncExplorationUIPlugin,
            AsyncExplorationPlugin,
            AsyncExplorationUIPlugin,
        ));
    }
}

/// System for initializing default guilds
pub fn initialize_default_guilds(
    mut commands: Commands,
    mut guild_manager: ResMut<GuildManager>,
    time: Res<Time>,
) {
    // Only run once
    if !guild_manager.guilds.is_empty() {
        return;
    }

    let current_time = time.elapsed_seconds_f64();

    // Create default NPC guilds
    let adventurers_guild_id = guild_manager.create_guild(
        "Adventurers Guild".to_string(),
        Entity::from_raw(0), // Placeholder entity
        current_time,
    );

    let mages_guild_id = guild_manager.create_guild(
        "Mages Guild".to_string(),
        Entity::from_raw(0), // Placeholder entity
        current_time,
    );

    let thieves_guild_id = guild_manager.create_guild(
        "Thieves Guild".to_string(),
        Entity::from_raw(0), // Placeholder entity
        current_time,
    );

    // Add some initial resources to NPC guilds
    if let Some(guild) = guild_manager.get_guild_mut(&adventurers_guild_id) {
        guild.add_resource(GuildResource::Gold, 5000);
        guild.add_resource(GuildResource::Reputation, 500);
        guild.add_resource(GuildResource::Supplies, 2000);
        guild.description = "A renowned guild of adventurers who explore dungeons and complete quests.".to_string();
        
        // Add some facilities
        guild.build_facility(GuildFacility::TrainingHall);
        guild.build_facility(GuildFacility::Infirmary);
    }

    if let Some(guild) = guild_manager.get_guild_mut(&mages_guild_id) {
        guild.add_resource(GuildResource::Gold, 8000);
        guild.add_resource(GuildResource::MagicEssence, 1000);
        guild.add_resource(GuildResource::Reputation, 700);
        guild.description = "A prestigious guild of mages dedicated to the study and advancement of magical arts.".to_string();
        
        // Add some facilities
        guild.build_facility(GuildFacility::Library);
        guild.build_facility(GuildFacility::MagicLab);
    }

    if let Some(guild) = guild_manager.get_guild_mut(&thieves_guild_id) {
        guild.add_resource(GuildResource::Gold, 10000);
        guild.add_resource(GuildResource::Supplies, 1000);
        guild.add_resource(GuildResource::RareArtifacts, 50);
        guild.description = "A secretive guild of thieves operating from the shadows.".to_string();
        
        // Add some facilities
        guild.build_facility(GuildFacility::Vault);
    }

    info!("Initialized default NPC guilds");
}

/// Setup function for the guild system
pub fn setup_guild_system(app: &mut App) {
    app.add_plugins(GuildPlugin)
        .add_systems(Startup, initialize_default_guilds);
}