use bevy::prelude::*;
use serde_json;
use std::fs;
use std::path::{Path, PathBuf};
use std::io::{self, Read, Write};
use crate::guild::*;

/// Resource for managing guild persistence
#[derive(Resource)]
pub struct GuildPersistence {
    pub save_directory: PathBuf,
    pub auto_save_interval: f32,
    pub last_save_time: f32,
}

impl Default for GuildPersistence {
    fn default() -> Self {
        GuildPersistence {
            save_directory: PathBuf::from("saves/guilds"),
            auto_save_interval: 300.0, // 5 minutes
            last_save_time: 0.0,
        }
    }
}

impl GuildPersistence {
    /// Create a new guild persistence manager with custom save directory
    pub fn new<P: AsRef<Path>>(save_directory: P) -> Self {
        GuildPersistence {
            save_directory: save_directory.as_ref().to_path_buf(),
            ..Default::default()
        }
    }

    /// Save guild manager data
    pub fn save_guild_manager(&self, guild_manager: &GuildManager) -> io::Result<()> {
        // Create save directory if it doesn't exist
        fs::create_dir_all(&self.save_directory)?;

        // Save guild manager data
        let manager_path = self.save_directory.join("guild_manager.json");
        let manager_data = serde_json::to_string_pretty(guild_manager)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        
        let mut file = fs::File::create(manager_path)?;
        file.write_all(manager_data.as_bytes())?;

        // Save individual guilds
        for (id, guild) in &guild_manager.guilds {
            self.save_guild(id, guild)?;
        }

        Ok(())
    }

    /// Save a single guild
    pub fn save_guild(&self, id: &str, guild: &Guild) -> io::Result<()> {
        let guild_path = self.save_directory.join(format!("guild_{}.json", id));
        let guild_data = serde_json::to_string_pretty(guild)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        
        let mut file = fs::File::create(guild_path)?;
        file.write_all(guild_data.as_bytes())?;

        Ok(())
    }

    /// Load guild manager data
    pub fn load_guild_manager(&self) -> io::Result<GuildManager> {
        let manager_path = self.save_directory.join("guild_manager.json");
        
        if !manager_path.exists() {
            return Ok(GuildManager::default());
        }

        let mut file = fs::File::open(manager_path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        let guild_manager = serde_json::from_str(&contents)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        Ok(guild_manager)
    }

    /// Load a single guild
    pub fn load_guild(&self, id: &str) -> io::Result<Guild> {
        let guild_path = self.save_directory.join(format!("guild_{}.json", id));
        
        let mut file = fs::File::open(guild_path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        let guild = serde_json::from_str(&contents)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        Ok(guild)
    }
}

/// System for auto-saving guild data
pub fn guild_auto_save_system(
    time: Res<Time>,
    mut persistence: ResMut<GuildPersistence>,
    guild_manager: Res<GuildManager>,
) {
    persistence.last_save_time += time.delta_seconds();

    if persistence.last_save_time >= persistence.auto_save_interval {
        if let Err(e) = persistence.save_guild_manager(&guild_manager) {
            error!("Failed to auto-save guild data: {}", e);
        } else {
            info!("Guild data auto-saved successfully");
        }
        persistence.last_save_time = 0.0;
    }
}

/// Plugin for guild persistence
pub struct GuildPersistencePlugin;

impl Plugin for GuildPersistencePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GuildPersistence>()
            .add_systems(Update, guild_auto_save_system);
    }
}