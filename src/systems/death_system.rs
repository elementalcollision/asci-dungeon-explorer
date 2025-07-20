use specs::{System, WriteStorage, ReadStorage, Entities, Entity, Join, Write, ReadExpect, WriteExpect, Component};
use crate::components::{
    CombatStats, Player, Name, Position, Renderable, Item, Inventory,
    Dead, DeathCause, Corpse, DeathAnimation, DeathAnimationType
};
use crate::resources::{GameLog, RandomNumberGenerator};
use crate::map::Map;

// System to handle entity death
pub struct DeathSystem {}

impl<'a> System<'a> for DeathSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, CombatStats>,
        WriteStorage<'a, Dead>,
        WriteStorage<'a, Corpse>,
        WriteStorage<'a, DeathAnimation>,
        ReadStorage<'a, Player>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, Renderable>,
        ReadStorage<'a, Item>,
        WriteStorage<'a, Inventory>,
        Write<'a, GameLog>,
        Write<'a, RandomNumberGenerator>,
        WriteExpected<'a, Map>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            entities,
            mut combat_stats,
            mut dead,
            mut corpses,
            mut death_animations,
            players,
            names,
            positions,
            renderables,
            items,
            mut inventories,
            mut gamelog,
            mut rng,
            mut map,
        ) = data;

        // Find entities that should die
        let mut entities_to_kill = Vec::new();
        for (entity, stats) in (&entities, &combat_stats).join() {
            if stats.hp <= 0 {
                entities_to_kill.push(entity);
            }
        }

        // Process each death
        for entity in entities_to_kill {
            self.process_death(
                entity,
                &entities,
                &mut dead,
                &mut corpses,
                &mut death_animations,
                &players,
                &names,
                &positions,
                &renderables,
                &items,
                &mut inventories,
                &mut gamelog,
                &mut rng,
                &mut map,
            );
        }

        // Update death animations
        self.update_death_animations(&mut death_animations, &mut renderables);

        // Clean up finished animations and create corpses
        let mut to_remove = Vec::new();
        let mut corpses_to_create = Vec::new();

        for (entity, animation) in (&entities, &death_animations).join() {
            if animation.elapsed >= animation.duration {
                to_remove.push(entity);
                
                // Create corpse if entity had a position
                if let Some(pos) = positions.get(entity) {
                    corpses_to_create.push((pos.clone(), entity));
                }
            }
        }

        // Remove finished animations
        for entity in to_remove {
            death_animations.remove(entity);
        }

        // Create corpses
        for (pos, original_entity) in corpses_to_create {
            self.create_corpse(
                &entities,
                &mut corpses,
                &mut renderables,
                &mut positions,
                pos,
                Some(original_entity),
            );
        }

        // Handle corpse decay
        self.handle_corpse_decay(&entities, &mut corpses, &mut gamelog);
    }
}

impl DeathSystem {
    fn process_death(
        &self,
        entity: Entity,
        entities: &Entities,
        dead: &mut WriteStorage<Dead>,
        corpses: &mut WriteStorage<Corpse>,
        death_animations: &mut WriteStorage<DeathAnimation>,
        players: &ReadStorage<Player>,
        names: &ReadStorage<Name>,
        positions: &ReadStorage<Position>,
        renderables: &ReadStorage<Renderable>,
        items: &ReadStorage<Item>,
        inventories: &mut WriteStorage<Inventory>,
        gamelog: &mut GameLog,
        rng: &mut RandomNumberGenerator,
        map: &mut Map,
    ) {
        // Mark entity as dead
        dead.insert(entity, Dead {
            cause: DeathCause::Combat(entity), // TODO: Track actual cause
            time_of_death: 0, // TODO: Add game time tracking
        }).expect("Unable to insert Dead component");

        // Log death message
        if let Some(name) = names.get(entity) {
            if players.get(entity).is_some() {
                gamelog.entries.push(format!("You have died!"));
            } else {
                gamelog.entries.push(format!("{} dies!", name.name));
            }
        } else {
            gamelog.entries.push("Something dies!".to_string());
        }

        // Drop inventory items
        if let Some(inventory) = inventories.get_mut(entity) {
            if let Some(pos) = positions.get(entity) {
                self.drop_inventory_items(
                    &inventory.items.clone(),
                    pos,
                    entities,
                    positions,
                    renderables,
                    items,
                    map,
                );
            }
            inventory.items.clear();
        }

        // TODO: Drop equipped items when equipment system is implemented

        // Start death animation
        if let Some(renderable) = renderables.get(entity) {
            let animation_type = self.choose_death_animation(rng);
            death_animations.insert(entity, DeathAnimation {
                animation_type,
                duration: 1.0, // 1 second animation
                elapsed: 0.0,
                original_glyph: renderable.glyph,
                original_color: renderable.fg,
            }).expect("Unable to insert death animation");
        }

        // Trigger death events
        self.trigger_death_events(entity, players, gamelog);
    }

    fn choose_death_animation(&self, rng: &mut RandomNumberGenerator) -> DeathAnimationType {
        match rng.roll_dice(1, 4) {
            1 => DeathAnimationType::Fade,
            2 => DeathAnimationType::Dissolve,
            3 => DeathAnimationType::Explosion,
            _ => DeathAnimationType::Collapse,
        }
    }

    fn update_death_animations(
        &self,
        death_animations: &mut WriteStorage<DeathAnimation>,
        renderables: &mut WriteStorage<Renderable>,
    ) {
        for (animation, renderable) in (death_animations, renderables).join() {
            animation.elapsed += 0.016; // Assume ~60 FPS

            let progress = (animation.elapsed / animation.duration).min(1.0);
            
            match animation.animation_type {
                DeathAnimationType::Fade => {
                    // Fade to darker colors
                    if progress < 0.5 {
                        renderable.fg = crossterm::style::Color::DarkGrey;
                    } else {
                        renderable.fg = crossterm::style::Color::Black;
                    }
                },
                DeathAnimationType::Dissolve => {
                    // Change glyph to represent dissolution
                    if progress < 0.33 {
                        renderable.glyph = '▓';
                    } else if progress < 0.66 {
                        renderable.glyph = '▒';
                    } else {
                        renderable.glyph = '░';
                    }
                },
                DeathAnimationType::Explosion => {
                    // Flash bright then fade
                    if progress < 0.2 {
                        renderable.fg = crossterm::style::Color::White;
                        renderable.glyph = '*';
                    } else if progress < 0.6 {
                        renderable.fg = crossterm::style::Color::Red;
                        renderable.glyph = '×';
                    } else {
                        renderable.fg = crossterm::style::Color::DarkRed;
                        renderable.glyph = '·';
                    }
                },
                DeathAnimationType::Collapse => {
                    // Change glyph to show collapse
                    if progress < 0.5 {
                        renderable.glyph = '≈';
                    } else {
                        renderable.glyph = '_';
                        renderable.fg = crossterm::style::Color::DarkGrey;
                    }
                },
            }
        }
    }

    fn create_corpse(
        &self,
        entities: &Entities,
        corpses: &mut WriteStorage<Corpse>,
        renderables: &mut WriteStorage<Renderable>,
        positions: &mut WriteStorage<Position>,
        position: Position,
        original_entity: Option<Entity>,
    ) {
        let corpse_entity = entities.create();
        
        corpses.insert(corpse_entity, Corpse {
            original_entity,
            decay_timer: 100, // Corpse lasts 100 turns
            loot_generated: false,
        }).expect("Unable to insert corpse");

        renderables.insert(corpse_entity, Renderable {
            glyph: '%',
            fg: crossterm::style::Color::DarkRed,
            bg: crossterm::style::Color::Black,
            render_order: 1,
        }).expect("Unable to insert corpse renderable");

        positions.insert(corpse_entity, position)
            .expect("Unable to insert corpse position");
    }

    fn drop_inventory_items(
        &self,
        items: &[Entity],
        position: &Position,
        entities: &Entities,
        positions: &WriteStorage<Position>,
        renderables: &WriteStorage<Renderable>,
        item_components: &ReadStorage<Item>,
        map: &mut Map,
    ) {
        for &item_entity in items {
            // Find a nearby empty spot to drop the item
            if let Some(drop_pos) = self.find_drop_position(position, map) {
                // Move item to drop position
                if let Some(mut item_pos) = positions.get_mut(item_entity) {
                    item_pos.x = drop_pos.0;
                    item_pos.y = drop_pos.1;
                }
            }
        }
    }



    fn find_drop_position(&self, center: &Position, map: &Map) -> Option<(i32, i32)> {
        // Try positions in expanding circles around the center
        for radius in 0..5 {
            for dx in -radius..=radius {
                for dy in -radius..=radius {
                    if dx.abs() != radius && dy.abs() != radius && radius > 0 {
                        continue; // Only check the perimeter
                    }

                    let x = center.x + dx;
                    let y = center.y + dy;

                    if x >= 0 && x < map.width && y >= 0 && y < map.height {
                        let idx = map.xy_idx(x, y);
                        if map.tiles[idx].walkable {
                            return Some((x, y));
                        }
                    }
                }
            }
        }
        None
    }

    fn handle_corpse_decay(
        &self,
        entities: &Entities,
        corpses: &mut WriteStorage<Corpse>,
        gamelog: &mut GameLog,
    ) {
        let mut to_remove = Vec::new();

        for (entity, corpse) in (entities, corpses).join() {
            corpse.decay_timer -= 1;
            
            if corpse.decay_timer <= 0 {
                to_remove.push(entity);
            }
        }

        // Remove decayed corpses
        for entity in to_remove {
            entities.delete(entity).expect("Unable to delete decayed corpse");
        }
    }

    fn trigger_death_events(
        &self,
        entity: Entity,
        players: &ReadStorage<Player>,
        gamelog: &mut GameLog,
    ) {
        // Check if player died
        if players.get(entity).is_some() {
            // Player death - this could trigger game over, respawn, etc.
            gamelog.entries.push("Game Over! Press 'R' to restart or 'Q' to quit.".to_string());
        }

        // TODO: Add other death-triggered events
        // - Quest updates
        // - Achievement unlocks
        // - Environmental changes
        // - NPC reactions
    }
}

// System to clean up dead entities after animations complete
pub struct DeadEntityCleanupSystem {}

impl<'a> System<'a> for DeadEntityCleanupSystem {
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, Dead>,
        ReadStorage<'a, DeathAnimation>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, dead, death_animations) = data;

        let mut to_delete = Vec::new();

        // Find dead entities without active death animations
        for (entity, _dead) in (&entities, &dead).join() {
            if death_animations.get(entity).is_none() {
                to_delete.push(entity);
            }
        }

        // Delete the entities
        for entity in to_delete {
            entities.delete(entity).expect("Unable to delete dead entity");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use specs::{World, WorldExt, Builder};
    use crate::resources::*;
    use crate::map::*;

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
    
    #[test]
    fn test_corpse_decay_timer() {
        let corpse = Corpse {
            original_entity: None,
            decay_timer: 100,
            loot_generated: false,
        };
        
        assert_eq!(corpse.decay_timer, 100);
        assert!(!corpse.loot_generated);
    }
    
    #[test]
    fn test_death_causes() {
        let combat_death = DeathCause::Combat(specs::Entity::from_raw(1));
        let env_death = DeathCause::Environment;
        let poison_death = DeathCause::Poison;
        
        // Test that death causes can be created
        match combat_death {
            DeathCause::Combat(_) => {},
            _ => panic!("Should be combat death"),
        }
        
        match env_death {
            DeathCause::Environment => {},
            _ => panic!("Should be environment death"),
        }
        
        match poison_death {
            DeathCause::Poison => {},
            _ => panic!("Should be poison death"),
        }
    }
}