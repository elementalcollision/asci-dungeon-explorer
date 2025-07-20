#[cfg(test)]
mod tests {
    use super::*;
    use specs::{World, WorldExt, Builder};
    use crate::components::*;
    use crate::resources::*;
    use crate::map::*;

    #[test]
    fn test_entity_death() {
        let mut world = World::new();
        
        // Register components
        world.register::<Position>();
        world.register::<CombatStats>();
        world.register::<Name>();
        world.register::<Renderable>();
        world.register::<Dead>();
        world.register::<DeathAnimation>();
        world.register::<Player>();
        world.register::<Monster>();
        world.register::<Item>();
        world.register::<Inventory>();
        
        // Add resources
        world.insert(GameLog::new());
        world.insert(RandomNumberGenerator::new());
        world.insert(Map::new_empty(10, 10));
        
        // Create a test entity with 0 HP
        let entity = world.create_entity()
            .with(Position { x: 5, y: 5 })
            .with(CombatStats { max_hp: 10, hp: 0, defense: 5, power: 5 })
            .with(Name { name: "Test Monster".to_string() })
            .with(Renderable {
                glyph: 'M',
                fg: crossterm::style::Color::Red,
                bg: crossterm::style::Color::Black,
                render_order: 2,
            })
            .with(Monster)
            .build();
        
        // Run the death system
        let mut death_system = DeathSystem {};
        death_system.run_now(&world);
        world.maintain();
        
        // Check that the entity is marked as dead
        let dead_storage = world.read_storage::<Dead>();
        assert!(dead_storage.get(entity).is_some(), "Entity should be marked as dead");
        
        // Check that a death animation was created
        let animation_storage = world.read_storage::<DeathAnimation>();
        assert!(animation_storage.get(entity).is_some(), "Death animation should be created");
    }
    
    #[test]
    fn test_corpse_creation() {
        let mut world = World::new();
        
        // Register components
        world.register::<Position>();
        world.register::<Renderable>();
        world.register::<Corpse>();
        
        // Create death system
        let death_system = DeathSystem {};
        
        // Test corpse creation
        let entities = world.entities();
        let mut corpses = world.write_storage::<Corpse>();
        let mut renderables = world.write_storage::<Renderable>();
        let mut positions = world.write_storage::<Position>();
        
        death_system.create_corpse(
            &entities,
            &mut corpses,
            &mut renderables,
            &mut positions,
            Position { x: 3, y: 3 },
            None,
        );
        
        // Check that a corpse was created
        let corpse_count = (&corpses).join().count();
        assert_eq!(corpse_count, 1, "One corpse should be created");
        
        // Check corpse properties
        for (corpse, renderable, position) in (&corpses, &renderables, &positions).join() {
            assert_eq!(corpse.decay_timer, 100, "Corpse should have decay timer");
            assert_eq!(renderable.glyph, '%', "Corpse should have % glyph");
            assert_eq!(position.x, 3, "Corpse should be at correct position");
            assert_eq!(position.y, 3, "Corpse should be at correct position");
        }
    }
    
    #[test]
    fn test_death_animation_types() {
        let mut rng = RandomNumberGenerator::new();
        let death_system = DeathSystem {};
        
        // Test that different animation types are returned
        let mut animation_types = std::collections::HashSet::new();
        for _ in 0..20 {
            let animation_type = death_system.choose_death_animation(&mut rng);
            animation_types.insert(format!("{:?}", animation_type));
        }
        
        // Should have at least 2 different animation types in 20 tries
        assert!(animation_types.len() >= 2, "Should generate different animation types");
    }
}