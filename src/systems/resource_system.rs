use specs::{System, WriteStorage, ReadStorage, Entities, Entity, Join, Write};
use crate::components::{PlayerResources, StatusEffects, StatusEffectType, CombatStats, Player};
use crate::resources::GameLog;

pub struct ResourceRegenerationSystem {}

impl<'a> System<'a> for ResourceRegenerationSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, PlayerResources>,
        ReadStorage<'a, StatusEffects>,
        ReadStorage<'a, Player>,
        Write<'a, GameLog>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, mut resources, status_effects, players, mut gamelog) = data;

        for (entity, mut resource, _player) in (&entities, &mut resources, &players).join() {
            // Update regeneration timers
            resource.mana_regen_timer += 1;
            resource.stamina_regen_timer += 1;
            
            // Calculate regeneration rates (affected by status effects)
            let mut mana_regen_modifier = 1.0;
            let mut stamina_regen_modifier = 1.0;
            
            if let Some(effects) = status_effects.get(entity) {
                for effect in &effects.effects {
                    match effect.effect_type {
                        StatusEffectType::ManaRegenBoost => {
                            mana_regen_modifier += effect.magnitude as f32 * 0.1;
                        },
                        StatusEffectType::ManaRegenPenalty => {
                            mana_regen_modifier -= effect.magnitude as f32 * 0.1;
                        },
                        StatusEffectType::StaminaRegenBoost => {
                            stamina_regen_modifier += effect.magnitude as f32 * 0.1;
                        },
                        StatusEffectType::StaminaRegenPenalty => {
                            stamina_regen_modifier -= effect.magnitude as f32 * 0.1;
                        },
                        _ => {}
                    }
                }
            }
            
            // Regenerate mana every 3 turns
            if resource.mana_regen_timer >= 3 {
                let mana_gain = ((resource.mana_regen_rate as f32) * mana_regen_modifier) as i32;
                if mana_gain > 0 && resource.mana < resource.max_mana {
                    resource.restore_mana(mana_gain);
                    resource.mana_regen_timer = 0;
                }
            }
            
            // Regenerate stamina every 2 turns
            if resource.stamina_regen_timer >= 2 {
                let stamina_gain = ((resource.stamina_regen_rate as f32) * stamina_regen_modifier) as i32;
                if stamina_gain > 0 && resource.stamina < resource.max_stamina {
                    resource.restore_stamina(stamina_gain);
                    resource.stamina_regen_timer = 0;
                }
            }
        }
    }
}

pub struct StatusEffectSystem {}

impl<'a> System<'a> for StatusEffectSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, StatusEffects>,
        WriteStorage<'a, CombatStats>,
        WriteStorage<'a, PlayerResources>,
        Write<'a, GameLog>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, mut status_effects, mut combat_stats, mut resources, mut gamelog) = data;

        for (entity, mut effects) in (&entities, &mut status_effects).join() {
            // Apply status effect damage/healing
            for effect in &effects.effects {
                match effect.effect_type {
                    StatusEffectType::Poisoned => {
                        if let Some(stats) = combat_stats.get_mut(entity) {
                            let damage = effect.magnitude;
                            stats.hp -= damage;
                            gamelog.add_entry(format!("Poison deals {} damage!", damage));
                        }
                    },
                    StatusEffectType::Blessed => {
                        if let Some(stats) = combat_stats.get_mut(entity) {
                            let healing = effect.magnitude;
                            stats.hp = i32::min(stats.hp + healing, stats.max_hp);
                            gamelog.add_entry(format!("Blessing heals {} HP!", healing));
                        }
                    },
                    _ => {}
                }
            }
            
            // Update effect durations
            effects.update_effects();
        }
    }
}

pub struct AbilityUsageSystem {}

impl<'a> System<'a> for AbilityUsageSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, crate::components::WantsToUseAbility>,
        WriteStorage<'a, PlayerResources>,
        WriteStorage<'a, crate::components::Abilities>,
        Write<'a, GameLog>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, mut wants_use_ability, mut resources, mut abilities, mut gamelog) = data;

        let mut ability_uses = Vec::new();
        
        // Collect ability usage requests
        for (entity, ability_use) in (&entities, &wants_use_ability).join() {
            ability_uses.push((entity, ability_use.ability, ability_use.mana_cost, ability_use.stamina_cost));
        }
        
        // Clear ability usage requests
        wants_use_ability.clear();
        
        // Process ability usage
        for (entity, ability_type, mana_cost, stamina_cost) in ability_uses {
            let mut can_use = true;
            let mut insufficient_resources = Vec::new();
            
            // Check resource requirements
            if let Some(resource) = resources.get_mut(entity) {
                if resource.mana < mana_cost {
                    can_use = false;
                    insufficient_resources.push("mana");
                }
                if resource.stamina < stamina_cost {
                    can_use = false;
                    insufficient_resources.push("stamina");
                }
                
                if can_use {
                    // Consume resources
                    resource.consume_mana(mana_cost);
                    resource.consume_stamina(stamina_cost);
                    
                    // Set ability on cooldown
                    if let Some(ability_comp) = abilities.get_mut(entity) {
                        ability_comp.set_cooldown(ability_type, ability_type.cooldown());
                    }
                    
                    gamelog.add_entry(format!("You used {}! (Cost: {} mana, {} stamina)", 
                        ability_type.name(), mana_cost, stamina_cost));
                } else {
                    gamelog.add_entry(format!("Not enough {} to use {}!", 
                        insufficient_resources.join(" and "), ability_type.name()));
                }
            }
        }
    }
}