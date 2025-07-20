use specs::{World, WorldExt, Builder, Entity};
use crate::components::*;
use crate::map::TileType;
use crate::resources::RandomNumberGenerator;

pub struct EntityFactory;

impl EntityFactory {
    // Create a player entity
    pub fn create_player(world: &mut World, x: i32, y: i32) -> Entity {
        world.create_entity()
            .with(Position { x, y })
            .with(Renderable {
                glyph: '@',
                fg: (255, 255, 255),
                bg: (0, 0, 0),
                render_order: 0,
            })
            .with(Player {})
            .with(Viewshed {
                visible_tiles: Vec::new(),
                range: 8,
                dirty: true,
            })
            .with(Name {
                name: "Player".to_string(),
            })
            .with(CombatStats {
                max_hp: 30,
                hp: 30,
                defense: 2,
                power: 5,
            })
            .with(PlayerInput::new())
            .with(Inventory::new(26))
            .with(Experience::new())
            .build()
    }
    
    // Create a monster entity
    pub fn create_monster(world: &mut World, x: i32, y: i32, monster_type: i32) -> Entity {
        let (glyph, name, hp, power) = match monster_type {
            0 => ('r', "Rat", 3, 3),      // Rat
            1 => ('g', "Goblin", 6, 4),   // Goblin
            2 => ('o', "Orc", 10, 6),     // Orc
            _ => ('r', "Rat", 3, 3),      // Default to rat
        };
        
        world.create_entity()
            .with(Position { x, y })
            .with(Renderable {
                glyph,
                fg: (255, 0, 0),
                bg: (0, 0, 0),
                render_order: 1,
            })
            .with(Viewshed {
                visible_tiles: Vec::new(),
                range: 6,
                dirty: true,
            })
            .with(Name {
                name: name.to_string(),
            })
            .with(BlocksTile {})
            .with(CombatStats {
                max_hp: hp,
                hp,
                defense: 1,
                power,
            })
            .with(Monster {})
            .build()
    }
    
    // Create an item entity
    pub fn create_health_potion(world: &mut World, x: i32, y: i32) -> Entity {
        world.create_entity()
            .with(Position { x, y })
            .with(Renderable {
                glyph: '!',
                fg: (0, 255, 0),
                bg: (0, 0, 0),
                render_order: 2,
            })
            .with(Name {
                name: "Health Potion".to_string(),
            })
            .with(Item {})
            .with(ProvidesHealing { heal_amount: 8 })
            .build()
    }
    
    // Create stairs down
    pub fn create_stairs_down(world: &mut World, x: i32, y: i32) -> Entity {
        world.create_entity()
            .with(Position { x, y })
            .with(Renderable {
                glyph: '>',
                fg: (0, 255, 255),
                bg: (0, 0, 0),
                render_order: 3,
            })
            .with(Name {
                name: "Stairs Down".to_string(),
            })
            .build()
    }
    
    // Create stairs up
    pub fn create_stairs_up(world: &mut World, x: i32, y: i32) -> Entity {
        world.create_entity()
            .with(Position { x, y })
            .with(Renderable {
                glyph: '<',
                fg: (0, 255, 255),
                bg: (0, 0, 0),
                render_order: 3,
            })
            .with(Name {
                name: "Stairs Up".to_string(),
            })
            .build()
    }
}