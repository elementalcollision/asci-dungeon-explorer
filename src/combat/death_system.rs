use specs::{System, WriteStorage, ReadStorage, Entities, Entity, Join, ReadExpect, Write};
use crate::components::{CombatStats, Player, Name, Position, BlocksTile, Renderable};
use crate::resources::GameLog;
use crossterm::style::Color;

pub struct DeathSystem {}

impl<'a> System<'a> for DeathSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, CombatStats>,
        ReadStorage<'a, Player>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, Renderable>,
        WriteStorage<'a, Name>,
        WriteStorage<'a, BlocksTile>,
        Write<'a, GameLog>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, mut combat_stats, player, mut positions, mut renderables, names, mut blocks_tile, mut gamelog) = data;

        // Find dead entities
        let mut dead_entities = Vec::new();
        for (entity, stats) in (&entities, &combat_stats).join() {
            if stats.hp <= 0 {
                // Check if it's the player
                let is_player = player.contains(entity);
                
                if is_player {
                    // Player death is handled differently - don't remove them
                    gamelog.add_entry("You have died! Game over.".to_string());
                } else {
                    // For non-player entities, mark them for removal
                    dead_entities.push(entity);
                    
                    // Log the death if the entity has a name
                    if let Some(name) = names.get(entity) {
                        gamelog.add_entry(format!("{} is dead!", name.name));
                    }
                    
                    // Turn the entity into a corpse
                    if let Some(pos) = positions.get(entity) {
                        let pos = *pos;
                        
                        // Remove the BlocksTile component
                        blocks_tile.remove(entity);
                        
                        // Change the renderable to a corpse
                        if let Some(render) = renderables.get_mut(entity) {
                            render.glyph = '%';
                            render.fg = Color::Red;
                        }
                    }
                }
            }
        }
        
        // Remove dead entities
        for entity in dead_entities {
            entities.delete(entity).expect("Unable to delete dead entity");
        }
    }
}