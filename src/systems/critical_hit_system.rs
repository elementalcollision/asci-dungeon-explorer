use specs::{System, WriteStorage, ReadStorage, Entities, Entity, Join, Write};
use crate::components::{
    DamageInfo, Attacker, CombatStats, Name, Player, StatusEffects, StatusEffect, StatusEffectType
};
use crate::resources::{GameLog, RandomNumberGenerator};

pub struct CriticalHitSystem {}

impl<'a> System<'a> for CriticalHitSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, DamageInfo>,
        ReadStorage<'a, Attacker>,
        ReadStorage<'a, CombatStats>,
        WriteStorage<'a, StatusEffects>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, Player>,
        Write<'a, GameLog>,
        Write<'a, RandomNumberGenerator>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, mut damage_info, attackers, combat_stats, mut status_effects, names, players, mut gamelog, mut rng) = data;

        // Process critical hits and apply special effects
        let mut critical_hits = Vec::new();
        
        for (entity, damage) in (&entities, &damage_info).join() {
            if damage.is_critical {
                critical_hits.push((entity, damage.clone()));
            }
        }
        
        for (target_entity, damage) in critical_hits {
            // Apply critical hit effects based on damage type and circumstances
            self.apply_critical_effects(
                target_entity,
                &damage,
                &mut status_effects,
                &names,
                &players,
                &mut gamelog,
                &mut rng
            );
            
            // Enhanced critical hit damage calculation
            if let Some(attacker_comp) = attackers.get(damage.source) {
                let enhanced_damage = self.calculate_enhanced_critical_damage(
                    &damage,
                    attacker_comp,
                    &mut rng
                );
                
                // Update damage info with enhanced critical damage
                if let Some(mut damage_info) = damage_info.get_mut(target_entity) {
                    damage_info.base_damage = enhanced_damage;
                }
            }
        }
    }
}

impl CriticalHitSystem {
    fn apply_critical_effects(
        &self,
        target: Entity,
        damage: &DamageInfo,
        status_effects: &mut WriteStorage<StatusEffects>,
        names: &ReadStorage<Name>,
        players: &ReadStorage<Player>,
        gamelog: &mut GameLog,
        rng: &mut RandomNumberGenerator,
    ) {
        let target_name = names.get(target).map_or("Unknown", |n| &n.name);
        
        // Get or create status effects component
        let effects = if let Some(effects) = status_effects.get_mut(target) {
            effects
        } else {
            status_effects.insert(target, StatusEffects::new())
                .expect("Failed to insert status effects");
            status_effects.get_mut(target).unwrap()
        };
        
        // Apply critical hit effects based on damage type
        match damage.damage_type {
            crate::components::DamageType::Physical => {
                // Physical crits can cause bleeding or stunning
                let effect_roll = rng.roll_dice(1, 100);
                if effect_roll <= 30 { // 30% chance
                    effects.add_effect(StatusEffect {
                        effect_type: StatusEffectType::Poisoned, // Using as bleeding
                        duration: 3,
                        magnitude: 2,
                    });
                    gamelog.add_entry(format!("{} is bleeding from the critical hit!", target_name));
                } else if effect_roll <= 50 { // 20% chance for stun
                    effects.add_effect(StatusEffect {
                        effect_type: StatusEffectType::Slow, // Using as stun
                        duration: 1,
                        magnitude: 3,
                    });
                    gamelog.add_entry(format!("{} is stunned by the critical hit!", target_name));
                }
            },
            crate::components::DamageType::Fire => {
                // Fire crits cause burning
                effects.add_effect(StatusEffect {
                    effect_type: StatusEffectType::Poisoned, // Using as burning
                    duration: 4,
                    magnitude: 3,
                });
                gamelog.add_entry(format!("{} is set ablaze by the critical hit!", target_name));
            },
            crate::components::DamageType::Ice => {
                // Ice crits cause freezing/slowing
                effects.add_effect(StatusEffect {
                    effect_type: StatusEffectType::Slow,
                    duration: 3,
                    magnitude: 2,
                });
                gamelog.add_entry(format!("{} is frozen by the critical hit!", target_name));
            },
            crate::components::DamageType::Lightning => {
                // Lightning crits cause paralysis
                effects.add_effect(StatusEffect {
                    effect_type: StatusEffectType::Slow, // Using as paralysis
                    duration: 2,
                    magnitude: 4,
                });
                gamelog.add_entry(format!("{} is paralyzed by the critical hit!", target_name));
            },
            crate::components::DamageType::Poison => {
                // Poison crits cause enhanced poisoning
                effects.add_effect(StatusEffect {
                    effect_type: StatusEffectType::Poisoned,
                    duration: 6,
                    magnitude: 4,
                });
                gamelog.add_entry(format!("{} is severely poisoned by the critical hit!", target_name));
            },
            crate::components::DamageType::Holy => {
                // Holy crits can cause blessing on allies or extra damage to undead
                if players.contains(target) {
                    effects.add_effect(StatusEffect {
                        effect_type: StatusEffectType::Blessed,
                        duration: 5,
                        magnitude: 2,
                    });
                    gamelog.add_entry(format!("{} is blessed by the holy critical hit!", target_name));
                } else {
                    gamelog.add_entry(format!("{} is seared by holy energy!", target_name));
                }
            },
            crate::components::DamageType::Dark => {
                // Dark crits cause cursing
                effects.add_effect(StatusEffect {
                    effect_type: StatusEffectType::Cursed,
                    duration: 8,
                    magnitude: 2,
                });
                gamelog.add_entry(format!("{} is cursed by the dark critical hit!", target_name));
            },
            crate::components::DamageType::Psychic => {
                // Psychic crits can cause confusion or fear
                let effect_roll = rng.roll_dice(1, 2);
                if effect_roll == 1 {
                    effects.add_effect(StatusEffect {
                        effect_type: StatusEffectType::Cursed, // Using as confusion
                        duration: 3,
                        magnitude: 1,
                    });
                    gamelog.add_entry(format!("{} is confused by the psychic critical hit!", target_name));
                } else {
                    effects.add_effect(StatusEffect {
                        effect_type: StatusEffectType::Slow, // Using as fear
                        duration: 2,
                        magnitude: 2,
                    });
                    gamelog.add_entry(format!("{} is terrified by the psychic critical hit!", target_name));
                }
            },
        }
    }
    
    fn calculate_enhanced_critical_damage(
        &self,
        damage: &DamageInfo,
        attacker: &Attacker,
        rng: &mut RandomNumberGenerator,
    ) -> i32 {
        // Base critical damage
        let base_critical = (damage.base_damage as f32 * attacker.critical_multiplier) as i32;
        
        // Add random variance for more exciting crits
        let variance_roll = rng.roll_dice(1, 6) - 3; // -2 to +3 variance
        let variance_damage = base_critical / 10 * variance_roll; // 10% variance per point
        
        // Ensure minimum damage
        i32::max(damage.base_damage + 1, base_critical + variance_damage)
    }
}

// System for calculating critical hit chances based on various factors
pub struct CriticalChanceSystem {}

impl<'a> System<'a> for CriticalChanceSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, Attacker>,
        ReadStorage<'a, crate::components::Attributes>,
        ReadStorage<'a, crate::components::Skills>,
        ReadStorage<'a, crate::components::Equipped>,
        ReadStorage<'a, crate::components::Inventory>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, mut attackers, attributes, skills, equipped_items, inventories) = data;

        // Update critical hit chances based on attributes, skills, and equipment
        for (entity, mut attacker) in (&entities, &mut attackers).join() {
            let mut total_crit_chance = 0.05; // Base 5% crit chance
            
            // Attribute bonus (dexterity affects crit chance)
            if let Some(attrs) = attributes.get(entity) {
                let dex_modifier = attrs.get_modifier(crate::components::AttributeType::Dexterity);
                total_crit_chance += (dex_modifier as f32) * 0.01; // 1% per dex modifier point
            }
            
            // Skill bonus (melee weapons skill affects crit chance)
            if let Some(skills) = skills.get(entity) {
                let weapon_skill = skills.get_skill_level(crate::components::SkillType::MeleeWeapons);
                total_crit_chance += (weapon_skill as f32) * 0.005; // 0.5% per skill level
            }
            
            // Equipment bonuses (weapons with crit bonuses)
            if let Some(inventory) = inventories.get(entity) {
                for &item_entity in &inventory.items {
                    if equipped_items.get(item_entity).is_some() {
                        // This would check for crit bonus on equipped items
                        // For now, we'll add a small bonus for having equipment
                        total_crit_chance += 0.01; // 1% bonus per equipped item
                    }
                }
            }
            
            // Cap critical chance at reasonable levels
            attacker.critical_chance = total_crit_chance.min(0.5); // Max 50% crit chance
        }
    }
}