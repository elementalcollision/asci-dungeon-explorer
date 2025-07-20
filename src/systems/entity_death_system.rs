use specs::{System, WriteStorage, ReadStorage, Entities, Entity, Join, Write, Builder};
use crate::components::{
    CombatStats, Name, Player, Monster, Position, Renderable, BlocksTile,
    DeathAnimation, Corpse, DeathTrigger, DeathEvent, Item, ProvidesHealing
};
use crate::resources::{GameLog, RandomNumberGenerator};
use crossterm::style::Color;

pub struct EntityDeathSystem {}

impl<'a> System<'a> for EntityDeathSystem {
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, CombatStats>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, Player>,
        ReadStorage<'a, Monster>,
        ReadStorage<'a, Position>,
        WriteStorage<'a, Renderable>,
        WriteStorage<'a, BlocksTile>,
        WriteStorage<'a, DeathAnimation>,
        WriteStorage<'a, Corpse>,
        ReadStorage<'a, DeathTrigger>,
        WriteStorage<'a, DeathEvent>,
        Write<'a, GameLog>,
        Write<'a, RandomNumberGenerator>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            entities, 
            combat_stats, 
            names, 
            players, 
            monsters,
            positions,
            mut renderables,
            mut blocks_tile,
            mut death_animations,
            mut corpses,
            death_triggers,
            mut death_events,
            mut gamelog, 
            mut rng
        ) = data;

        // Find entities that have died
        let mut dead_entities = Vec::new();
        
        for (entity, stats, name) in (&entities, &combat_stats, &names).join() {
            if stats.hp <= 0 {
                let is_player = players.contains(entity);
                let is_monster = monsters.contains(entity);
                let position = positions.get(entity).cloned();
                let death_trigger = death_triggers.get(entity).cloned();
                
                dead_entities.push((entity, name.name.clone(), is_player, is_monster, position, death_trigger));
            }
        }
        
        // Process each death
        for (dead_entity, entity_name, is_player, is_monster, position, death_trigger) in dead_entities {
            // Create death event
            let death_event = DeathEvent {
                entity: dead_entity,
                entity_name: entity_name.clone(),
                is_player,
                is_monster,
                position,
                death_time: std::time::SystemTime::now(),
            };
            
            death_events.insert(dead_entity, death_event)
                .expect("Failed to insert death event");
            
            // Handle player death differently
            if is_player {
                self.handle_player_death(
                    dead_entity,
                    &entity_name,
                    position,
                    &mut death_animations,
                    &mut gamelog
                );
            } else {
                // Handle monster/NPC death
                self.handle_entity_death(
                    dead_entity,
                    &entity_name,
                    position,
                    death_trigger,
                    &entities,
                    &mut renderables,
                    &mut blocks_tile,
                    &mut death_animations,
                    &mut corpses,
                    &mut gamelog,
                    &mut rng
                );
            }
        }
    }
}

impl EntityDeathSystem {
    fn handle_player_death(
        &self,
        player_entity: Entity,
        player_name: &str,
        position: Option<Position>,
        death_animations: &mut WriteStorage<DeathAnimation>,
        gamelog: &mut GameLog,
    ) {
        // Create death animation for player
        if let Some(pos) = position {
            let animation = DeathAnimation {
                animation_type: DeathAnimationType::PlayerDeath,
                duration: 3.0, // 3 seconds
                max_duration: 3.0,
                position: pos,
                current_frame: 0,
                max_frames: 10,
            };
            
            death_animations.insert(player_entity, animation)
                .expect("Failed to insert player death animation");
        }
        
        gamelog.add_entry(format!("*** {} HAS FALLEN! ***", player_name.to_uppercase()));
        gamelog.add_entry("The world grows dark as your vision fades...".to_string());
        
        // Player death is handled by the death and revival system
        // Don't remove the player entity here
    }
    
    fn handle_entity_death(
        &self,
        dead_entity: Entity,
        entity_name: &str,
        position: Option<Position>,
        death_trigger: Option<DeathTrigger>,
        entities: &Entities,
        renderables: &mut WriteStorage<Renderable>,
        blocks_tile: &mut WriteStorage<BlocksTile>,
        death_animations: &mut WriteStorage<DeathAnimation>,
        corpses: &mut WriteStorage<Corpse>,
        gamelog: &mut GameLog,
        rng: &mut RandomNumberGenerator,
    ) {
        // Log the death
        gamelog.add_entry(format!("{} has been slain!", entity_name));
        
        // Create death animation
        if let Some(pos) = position {
            let animation = DeathAnimation {
                animation_type: DeathAnimationType::MonsterDeath,
                duration: 1.5, // 1.5 seconds
                max_duration: 1.5,
                position: pos,
                current_frame: 0,
                max_frames: 6,
            };
            
            death_animations.insert(dead_entity, animation)
                .expect("Failed to insert death animation");
        }
        
        // Remove blocking component so other entities can move through
        blocks_tile.remove(dead_entity);
        
        // Process death triggers
        if let Some(trigger) = death_trigger {
            self.process_death_trigger(&trigger, position, entities, gamelog, rng);
        }
        
        // Convert to corpse or remove entity
        if let Some(pos) = position {
            // Chance to leave a corpse
            if rng.roll_dice(1, 100) <= 70 { // 70% chance to leave corpse
                self.create_corpse(dead_entity, entity_name, pos, renderables, corpses);
            } else {
                // Entity disappears completely
                gamelog.add_entry(format!("The {} crumbles to dust.", entity_name));
                entities.delete(dead_entity).expect("Failed to delete entity");
            }
        } else {
            // No position, just remove
            entities.delete(dead_entity).expect("Failed to delete entity");
        }
    }
    
    fn create_corpse(
        &self,
        dead_entity: Entity,
        entity_name: &str,
        position: Position,
        renderables: &mut WriteStorage<Renderable>,
        corpses: &mut WriteStorage<Corpse>,
    ) {
        // Change the entity's appearance to a corpse
        if let Some(renderable) = renderables.get_mut(dead_entity) {
            renderable.glyph = '%';
            renderable.fg = Color::DarkRed;
            renderable.render_order = 0; // Corpses render below living entities
        }
        
        // Add corpse component
        let corpse = Corpse {
            original_name: entity_name.to_string(),
            decay_time: 100, // Corpse lasts 100 turns
            max_decay_time: 100,
            can_be_looted: true,
            has_been_looted: false,
        };
        
        corpses.insert(dead_entity, corpse)
            .expect("Failed to insert corpse component");
    }
    
    fn process_death_trigger(
        &self,
        trigger: &DeathTrigger,
        position: Option<Position>,
        entities: &Entities,
        gamelog: &mut GameLog,
        rng: &mut RandomNumberGenerator,
    ) {
        match &trigger.trigger_type {
            DeathTriggerType::Explosion { damage, radius } => {
                if let Some(pos) = position {
                    gamelog.add_entry(format!("The creature explodes, dealing {} damage in a {} tile radius!", damage, radius));
                    // In a full implementation, this would damage nearby entities
                }
            },
            DeathTriggerType::SpawnEnemies { enemy_type, count } => {
                if let Some(pos) = position {
                    gamelog.add_entry(format!("{} {} emerge from the corpse!", count, enemy_type));
                    // In a full implementation, this would spawn new enemies
                }
            },
            DeathTriggerType::CastSpell { spell_name, effect } => {
                gamelog.add_entry(format!("As it dies, the creature casts {}!", spell_name));
                // In a full implementation, this would cast the spell
            },
            DeathTriggerType::DropSpecialItem { item_name } => {
                if let Some(pos) = position {
                    // Create a special item
                    entities.create()
                        .with(Item {})
                        .with(Name { name: item_name.clone() })
                        .with(Position { x: pos.x, y: pos.y })
                        .with(Renderable {
                            glyph: '*',
                            fg: Color::Yellow,
                            bg: Color::Black,
                            render_order: 2,
                        })
                        .with(ProvidesHealing { heal_amount: 25 }) // Default to healing item
                        .build();
                    
                    gamelog.add_entry(format!("A {} materializes from the creature's essence!", item_name));
                }
            },
            DeathTriggerType::PlaySound { sound_name } => {
                gamelog.add_entry(format!("♪ *{}*", sound_name));
            },
        }
    }
}

// System for updating death animations
pub struct DeathAnimationSystem {}

impl<'a> System<'a> for DeathAnimationSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, DeathAnimation>,
        WriteStorage<'a, Renderable>,
        Write<'a, GameLog>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, mut death_animations, mut renderables, mut gamelog) = data;

        let mut completed_animations = Vec::new();
        
        for (entity, mut animation) in (&entities, &mut death_animations).join() {
            // Update animation
            animation.duration -= 0.016; // Assuming ~60 FPS
            
            // Calculate current frame
            let progress = 1.0 - (animation.duration / animation.max_duration);
            animation.current_frame = (progress * animation.max_frames as f32) as i32;
            
            // Update visual effects based on animation type
            if let Some(renderable) = renderables.get_mut(entity) {
                match animation.animation_type {
                    DeathAnimationType::PlayerDeath => {
                        // Player death animation - fade to red
                        let alpha = (animation.duration / animation.max_duration).clamp(0.0, 1.0);
                        if alpha > 0.5 {
                            renderable.fg = Color::Red;
                        } else {
                            renderable.fg = Color::DarkRed;
                        }
                    },
                    DeathAnimationType::MonsterDeath => {
                        // Monster death animation - fade and flicker
                        let frame = animation.current_frame % 3;
                        match frame {
                            0 => renderable.fg = Color::Red,
                            1 => renderable.fg = Color::DarkRed,
                            _ => renderable.fg = Color::Grey,
                        }
                    },
                    DeathAnimationType::Explosion => {
                        // Explosion animation - bright flash
                        let frame = animation.current_frame % 4;
                        match frame {
                            0 => {
                                renderable.glyph = '*';
                                renderable.fg = Color::Yellow;
                            },
                            1 => {
                                renderable.glyph = '#';
                                renderable.fg = Color::Red;
                            },
                            2 => {
                                renderable.glyph = '+';
                                renderable.fg = Color::DarkRed;
                            },
                            _ => {
                                renderable.glyph = '.';
                                renderable.fg = Color::Grey;
                            },
                        }
                    },
                }
            }
            
            // Check if animation is complete
            if animation.duration <= 0.0 {
                completed_animations.push(entity);
            }
        }
        
        // Remove completed animations
        for entity in completed_animations {
            death_animations.remove(entity);
            
            // For non-player entities, this is when we actually remove them
            // (after the death animation completes)
            let combat_stats = world.read_storage::<CombatStats>();
            let players = world.read_storage::<Player>();
            
            if !players.contains(entity) {
                if let Some(stats) = combat_stats.get(entity) {
                    if stats.hp <= 0 {
                        entities.delete(entity).expect("Failed to delete dead entity");
                    }
                }
            }
        }
    }
}

// System for managing corpse decay
pub struct CorpseDecaySystem {}

impl<'a> System<'a> for CorpseDecaySystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, Corpse>,
        WriteStorage<'a, Renderable>,
        ReadStorage<'a, Name>,
        Write<'a, GameLog>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, mut corpses, mut renderables, names, mut gamelog) = data;

        let mut decayed_corpses = Vec::new();
        
        for (entity, mut corpse) in (&entities, &mut corpses).join() {
            // Update decay timer
            corpse.decay_time -= 1;
            
            // Update appearance based on decay
            if let Some(renderable) = renderables.get_mut(entity) {
                let decay_progress = 1.0 - (corpse.decay_time as f32 / corpse.max_decay_time as f32);
                
                if decay_progress > 0.8 {
                    // Almost completely decayed
                    renderable.fg = Color::DarkGrey;
                    renderable.glyph = '.';
                } else if decay_progress > 0.5 {
                    // Heavily decayed
                    renderable.fg = Color::Grey;
                } else if decay_progress > 0.2 {
                    // Moderately decayed
                    renderable.fg = Color::DarkRed;
                }
            }
            
            // Check if corpse should be removed
            if corpse.decay_time <= 0 {
                decayed_corpses.push(entity);
            }
        }
        
        // Remove fully decayed corpses
        for entity in decayed_corpses {
            if let Some(name) = names.get(entity) {
                gamelog.add_entry(format!("The {} corpse crumbles to dust.", name.name));
            }
            entities.delete(entity).expect("Failed to delete decayed corpse");
        }
    }
}