use specs::{System, WriteStorage, ReadStorage, Entities, Entity, Join, Write};
use crate::components::{DamageInfo, CombatStats, DamageResistances, Player, Name, StatusEffects, StatusEffect, StatusEffectType};
use crate::resources::GameLog;

pub struct EnhancedDamageSystem {}

impl<'a> System<'a> for EnhancedDamageSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, CombatStats>,
        WriteStorage<'a, DamageInfo>,
        ReadStorage<'a, DamageResistances>,
        WriteStorage<'a, StatusEffects>,
        ReadStorage<'a, Player>,
        ReadStorage<'a, Name>,
        Write<'a, GameLog>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, mut combat_stats, mut damage_info, resistances, mut status_effects, player, names, mut gamelog) = data;

        // Process all damage
        let mut damage_to_apply = Vec::new();
        
        for (entity, damage) in (&entities, &damage_info).join() {
            damage_to_apply.push((entity, damage.clone()));
        }
        
        // Clear damage info
        damage_info.clear();
        
        // Apply damage
        for (entity, damage) in damage_to_apply {
            if let Some(stats) = combat_stats.get_mut(entity) {
                let mut final_damage = damage.base_damage;
                
                // Apply resistances if not already applied
                if let Some(resist) = resistances.get(entity) {
                    final_damage = resist.calculate_damage(final_damage, damage.damage_type);
                }
                
                // Apply damage
                stats.hp -= final_damage;
                
                // Log damage for player
                if player.contains(entity) {
                    let damage_desc = if damage.is_critical {
                        format!("You take {} critical {} damage!", final_damage, damage.damage_type.name())
                    } else {
                        format!("You take {} {} damage!", final_damage, damage.damage_type.name())
                    };
                    gamelog.add_entry(damage_desc);
                } else if let Some(name) = names.get(entity) {
                    let damage_desc = if damage.is_critical {
                        format!("{} takes {} critical {} damage!", name.name, final_damage, damage.damage_type.name())
                    } else {
                        format!("{} takes {} {} damage!", name.name, final_damage, damage.damage_type.name())
                    };
                    gamelog.add_entry(damage_desc);
                }
                
                // Apply special damage type effects
                if let Some(effects) = status_effects.get_mut(entity) {
                    match damage.damage_type {
                        crate::components::DamageType::Fire => {
                            // Chance to apply burning
                            effects.add_effect(StatusEffect {
                                effect_type: StatusEffectType::Poisoned, // Using poison as burning for now
                                duration: 3,
                                magnitude: 2,
                            });
                        },
                        crate::components::DamageType::Ice => {
                            // Chance to apply slow
                            effects.add_effect(StatusEffect {
                                effect_type: StatusEffectType::Slow,
                                duration: 2,
                                magnitude: 1,
                            });
                        },
                        crate::components::DamageType::Lightning => {
                            // Chance to stun (using slow as stun)
                            effects.add_effect(StatusEffect {
                                effect_type: StatusEffectType::Slow,
                                duration: 1,
                                magnitude: 2,
                            });
                        },
                        crate::components::DamageType::Poison => {
                            // Apply poison
                            effects.add_effect(StatusEffect {
                                effect_type: StatusEffectType::Poisoned,
                                duration: 5,
                                magnitude: 1,
                            });
                        },
                        crate::components::DamageType::Holy => {
                            // Heal undead, damage evil (simplified)
                            if final_damage > 0 {
                                gamelog.add_entry("Holy energy burns the target!".to_string());
                            }
                        },
                        crate::components::DamageType::Dark => {
                            // Chance to apply curse
                            effects.add_effect(StatusEffect {
                                effect_type: StatusEffectType::Cursed,
                                duration: 10,
                                magnitude: 1,
                            });
                        },
                        _ => {} // Physical and Psychic don't have special effects
                    }
                }
                
                // Check for death
                if stats.hp <= 0 {
                    if player.contains(entity) {
                        gamelog.add_entry("You have been defeated!".to_string());
                    } else if let Some(name) = names.get(entity) {
                        gamelog.add_entry(format!("{} has been defeated!", name.name));
                    }
                }
            }
        }
    }
}