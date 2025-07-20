use specs::{World, WorldExt, Entity};
use serde::{Serialize, Deserialize};
use std::fs;
use std::path::Path;
use crate::components::{Experience, Attributes, Skills, Abilities, Player};

#[derive(Serialize, Deserialize)]
pub struct ProgressionSave {
    pub experience: Experience,
    pub attributes: Attributes,
    pub skills: Skills,
    pub abilities: Abilities,
}

pub struct ProgressionPersistence;

impl ProgressionPersistence {
    pub fn save_player_progression(world: &World, player_entity: Entity, save_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let experiences = world.read_storage::<Experience>();
        let attributes = world.read_storage::<Attributes>();
        let skills = world.read_storage::<Skills>();
        let abilities = world.read_storage::<Abilities>();
        
        if let (Some(exp), Some(attrs), Some(skills), Some(abilities)) = (
            experiences.get(player_entity),
            attributes.get(player_entity),
            skills.get(player_entity),
            abilities.get(player_entity)
        ) {
            let progression_save = ProgressionSave {
                experience: exp.clone(),
                attributes: attrs.clone(),
                skills: skills.clone(),
                abilities: abilities.clone(),
            };
            
            let serialized = serde_json::to_string_pretty(&progression_save)?;
            fs::write(save_path, serialized)?;
            
            Ok(())
        } else {
            Err("Player entity missing progression components".into())
        }
    }
    
    pub fn load_player_progression(world: &mut World, player_entity: Entity, save_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        if !Path::new(save_path).exists() {
            return Err("Save file does not exist".into());
        }
        
        let file_content = fs::read_to_string(save_path)?;
        let progression_save: ProgressionSave = serde_json::from_str(&file_content)?;
        
        // Update the player's progression components
        let mut experiences = world.write_storage::<Experience>();
        let mut attributes = world.write_storage::<Attributes>();
        let mut skills = world.write_storage::<Skills>();
        let mut abilities = world.write_storage::<Abilities>();
        
        experiences.insert(player_entity, progression_save.experience)?;
        attributes.insert(player_entity, progression_save.attributes)?;
        skills.insert(player_entity, progression_save.skills)?;
        abilities.insert(player_entity, progression_save.abilities)?;
        
        Ok(())
    }
    
    pub fn auto_save_progression(world: &World) -> Result<(), Box<dyn std::error::Error>> {
        use specs::Join;
        
        let players = world.read_storage::<Player>();
        let entities = world.entities();
        
        // Find the player entity
        for (entity, _player) in (&entities, &players).join() {
            Self::save_player_progression(world, entity, "saves/player_progression.json")?;
            break; // Assuming single player for now
        }
        
        Ok(())
    }
    
    pub fn create_save_directory() -> Result<(), Box<dyn std::error::Error>> {
        fs::create_dir_all("saves")?;
        Ok(())
    }
}