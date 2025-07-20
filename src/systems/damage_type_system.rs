use specs::{System, WriteStorage, ReadStorage, Entities, Entity, Join, Write};
use crate::components::{
    DamageInfo, DamageResistances, DamageType, CombatStats, Name, Player, Monster,
    StatusEffects, StatusEffect, StatusEffectType
};
use crate::resources::{GameLog, RandomNumberGenerator};

pub struct DamageTypeSystem {}

impl<'a> System<'a> for DamageTypeSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, DamageInfo>,
        WriteStorage<'a, DamageResistances>,
        ReadStorage<'a, CombatStats>,
        WriteStorage<'a, StatusEffects>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, Player>,
        ReadStorage<'a, Monster>,
        Write<'a, GameLog>,
        Write<'a, RandomNumberGenerator>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            entities, 
            mut damage_info, 
            mut resistances, 
            combat_stats,
            mut status_effects,
            names, 
            players,
            monsters,
            mut gamelog, 
            mut rng
        ) = data;

        // Process damage type interactions and apply resistances
        let mut damage_applications = Vec::new();
        
        for (entity, damage) in (&entities, &damage_info).join() {
            damage_applications.push((entity, damage.clone()));
        }
        
        for (target_entity, mut damage) in damage_applications {
            // Apply damage type specific effects
            self.apply_damage_type_effects(
                target_entity,
                &damage,
                &mut status_effects,
                &names,
                &mut gamelog,
                &mut rng
            );
            
            // Calculate resistance-modified damage
            let final_damage = if let Some(resist) = resistances.get(target_entity) {
                let modified_damage = resist.calculate_damage(damage.base_damage, damage.damage_type);
                
                // Log resistance effects
                if modified_damage < damage.base_damage {
                    let target_name = names.get(target_entity).map_or("Unknown", |n| &n.name);
                    let resistance_percent = ((damage.base_damage - modified_damage) as f32 / damage.base_damage as f32 * 100.0) as i32;
                    gamelog.add_entry(format!("{} resists {}% of the {} damage!", 
                        target_name, resistance_percent, damage.damage_type.name()));
                }
                
                modified_damage
            } else {
                damage.base_damage
            };
            
            // Update damage info with final damage
            damage.base_damage = final_damage;
            
            // Apply environmental damage type interactions
            self.apply_environmental_effects(
                target_entity,
                &damage,
                &combat_stats,
                &mut resistances,
                &names,
                &mut gamelog
            );
            
            // Update the damage info in the storage
            if let Some(mut stored_damage) = damage_info.get_mut(target_entity) {
                *stored_damage = damage;
            }
        }
    }
}

impl DamageTypeSystem {
    fn apply_damage_type_effects(
        &self,
        target: Entity,
        damage: &DamageInfo,
        status_effects: &mut WriteStorage<StatusEffects>,
        names: &ReadStorage<Name>,
        gamelog: &mut GameLog,
        rng: &mut RandomNumberGenerator,
    ) {
        let target_name = names.get(target).map_or("Unknown", |n| &n.name);
        
        // Get or create status effects
        let effects = if let Some(effects) = status_effects.get_mut(target) {
            effects
        } else {
            status_effects.insert(target, StatusEffects::new())
                .expect("Failed to insert status effects");
            status_effects.get_mut(target).unwrap()
        };
        
        // Apply damage type specific effects with probability
        match damage.damage_type {
            DamageType::Fire => {
                if rng.roll_dice(1, 100) <= 25 { // 25% chance
                    effects.add_effect(StatusEffect {
                        effect_type: StatusEffectType::Poisoned, // Burning
                        duration: 3,
                        magnitude: 2,
                    });
                    gamelog.add_entry(format!("{} catches fire!", target_name));
                }
            },
            DamageType::Ice => {
                if rng.roll_dice(1, 100) <= 30 { // 30% chance
                    effects.add_effect(StatusEffect {
                        effect_type: StatusEffectType::Slow,
                        duration: 2,
                        magnitude: 1,
                    });
                    gamelog.add_entry(format!("{} is slowed by the cold!", target_name));
                }
            },
            DamageType::Lightning => {
                if rng.roll_dice(1, 100) <= 20 { // 20% chance
                    effects.add_effect(StatusEffect {
                        effect_type: StatusEffectType::Slow, // Paralysis
                        duration: 1,
                        magnitude: 3,
                    });
                    gamelog.add_entry(format!("{} is paralyzed by electricity!", target_name));
                }
            },
            DamageType::Poison => {
                // Poison always applies poison effect
                effects.add_effect(StatusEffect {
                    effect_type: StatusEffectType::Poisoned,
                    duration: 4,
                    magnitude: 1,
                });
                gamelog.add_entry(format!("{} is poisoned!", target_name));
            },
            DamageType::Holy => {
                // Holy damage can purify negative effects
                let mut purified_effects = Vec::new();
                for effect in &effects.effects {
                    if !effect.effect_type.is_beneficial() {
                        purified_effects.push(effect.effect_type);
                    }
                }
                
                for effect_type in purified_effects {
                    effects.remove_effect(effect_type);
                    gamelog.add_entry(format!("Holy energy purifies {} of {}!", 
                        target_name, effect_type.name()));
                }
            },
            DamageType::Dark => {
                if rng.roll_dice(1, 100) <= 35 { // 35% chance
                    effects.add_effect(StatusEffect {
                        effect_type: StatusEffectType::Cursed,
                        duration: 5,
                        magnitude: 1,
                    });
                    gamelog.add_entry(format!("{} is cursed by dark energy!", target_name));
                }
            },
            DamageType::Psychic => {
                if rng.roll_dice(1, 100) <= 15 { // 15% chance
                    effects.add_effect(StatusEffect {
                        effect_type: StatusEffectType::Cursed, // Confusion
                        duration: 3,
                        magnitude: 2,
                    });
                    gamelog.add_entry(format!("{} is confused by psychic energy!", target_name));
                }
            },
            DamageType::Physical => {
                // Physical damage has no special effects by default
            },
        }
    }
    
    fn apply_environmental_effects(
        &self,
        target: Entity,
        damage: &DamageInfo,
        combat_stats: &ReadStorage<CombatStats>,
        resistances: &mut WriteStorage<DamageResistances>,
        names: &ReadStorage<Name>,
        gamelog: &mut GameLog,
    ) {
        // Apply damage type interactions (e.g., fire vs ice)
        if let Some(mut resist) = resistances.get_mut(target) {
            match damage.damage_type {
                DamageType::Fire => {
                    // Fire damage reduces ice resistance temporarily
                    let current_ice_resist = resist.get_resistance(DamageType::Ice);
                    if current_ice_resist > 0.0 {
                        resist.add_resistance(DamageType::Ice, (current_ice_resist - 0.1).max(0.0));
                        
                        let target_name = names.get(target).map_or("Unknown", |n| &n.name);
                        gamelog.add_entry(format!("{}'s ice resistance is weakened by fire!", target_name));
                    }
                },
                DamageType::Ice => {
                    // Ice damage reduces fire resistance temporarily
                    let current_fire_resist = resist.get_resistance(DamageType::Fire);
                    if current_fire_resist > 0.0 {
                        resist.add_resistance(DamageType::Fire, (current_fire_resist - 0.1).max(0.0));
                        
                        let target_name = names.get(target).map_or("Unknown", |n| &n.name);
                        gamelog.add_entry(format!("{}'s fire resistance is weakened by ice!", target_name));
                    }
                },
                DamageType::Holy => {
                    // Holy damage is extra effective against dark-aligned creatures
                    // This would be determined by creature type in a full implementation
                },
                DamageType::Dark => {
                    // Dark damage is extra effective against holy-aligned creatures
                },
                _ => {}
            }
        }
    }
}

// System for managing and updating damage resistances
pub struct ResistanceManagementSystem {}

impl<'a> System<'a> for ResistanceManagementSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, DamageResistances>,
        ReadStorage<'a, crate::components::Equipped>,
        ReadStorage<'a, crate::components::Inventory>,
        ReadStorage<'a, StatusEffects>,
        ReadStorage<'a, Name>,
        Write<'a, GameLog>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, mut resistances, equipped_items, inventories, status_effects, names, mut gamelog) = data;

        // Update resistances based on equipment and status effects
        for (entity, mut resist) in (&entities, &mut resistances).join() {
            // Reset resistances to base values (this would be stored separately in a full implementation)
            let mut base_resistances = DamageResistances::new();
            
            // Apply equipment-based resistances
            if let Some(inventory) = inventories.get(entity) {
                for &item_entity in &inventory.items {
                    if equipped_items.get(item_entity).is_some() {
                        // This would check item properties for resistance bonuses
                        // For demonstration, we'll add small resistances for equipped items
                        base_resistances.add_resistance(DamageType::Physical, 0.05);
                    }
                }
            }
            
            // Apply status effect based resistances
            if let Some(effects) = status_effects.get(entity) {
                for effect in &effects.effects {
                    match effect.effect_type {
                        StatusEffectType::Blessed => {
                            base_resistances.add_resistance(DamageType::Dark, 0.2);
                            base_resistances.add_resistance(DamageType::Holy, -0.1); // Negative resistance = vulnerability
                        },
                        StatusEffectType::Cursed => {
                            base_resistances.add_resistance(DamageType::Holy, 0.2);
                            base_resistances.add_resistance(DamageType::Dark, -0.1);
                        },
                        _ => {}
                    }
                }
            }
            
            // Update the entity's resistances
            *resist = base_resistances;
        }
    }
}