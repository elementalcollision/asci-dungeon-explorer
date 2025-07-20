use specs::{System, WriteStorage, ReadStorage, Entities, Entity, Join, Write, ReadExpected};
use crate::components::{
    WantsToAttack, CombatStats, Attacker, Defender, DamageInfo, DamageResistances, 
    DamageType, DefenseResult, Name, Player, Monster, Initiative
};
use crate::resources::{GameLog, RandomNumberGenerator};

pub struct EnhancedCombatSystem {}

impl<'a> System<'a> for EnhancedCombatSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, WantsToAttack>,
        ReadStorage<'a, CombatStats>,
        ReadStorage<'a, Attacker>,
        ReadStorage<'a, Defender>,
        WriteStorage<'a, DamageInfo>,
        ReadStorage<'a, DamageResistances>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, Player>,
        ReadStorage<'a, Monster>,
        Write<'a, GameLog>,
        Write<'a, RandomNumberGenerator>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            entities, 
            mut wants_attack, 
            combat_stats, 
            attackers, 
            defenders, 
            mut damage_info,
            damage_resistances,
            names, 
            players,
            monsters,
            mut gamelog, 
            mut rng
        ) = data;

        // Process attack intents
        let mut attack_intents = Vec::new();
        for (entity, attack) in (&entities, &wants_attack).join() {
            attack_intents.push((entity, attack.target));
        }
        
        // Clear attack intents
        wants_attack.clear();
        
        // Process attacks
        for (attacker_entity, target_entity) in attack_intents {
            // Get attacker and target data
            let attacker_stats = combat_stats.get(attacker_entity);
            let target_stats = combat_stats.get(target_entity);
            let attacker_comp = attackers.get(attacker_entity);
            let defender_comp = defenders.get(target_entity);
            
            if attacker_stats.is_none() || target_stats.is_none() {
                continue;
            }
            
            let attacker_stats = attacker_stats.unwrap();
            let target_stats = target_stats.unwrap();
            
            // Get names for logging
            let attacker_name = names.get(attacker_entity).map_or("Unknown", |n| &n.name);
            let target_name = names.get(target_entity).map_or("Unknown", |n| &n.name);
            
            // Calculate attack roll
            let attack_roll = rng.roll_dice(1, 20) + attacker_stats.power;
            let attack_bonus = attacker_comp.map_or(0, |a| a.attack_bonus);
            let total_attack = attack_roll + attack_bonus;
            
            // Calculate defense
            let base_ac = defender_comp.map_or(10, |d| d.armor_class);
            let defense_bonus = target_stats.defense;
            let total_ac = base_ac + defense_bonus;
            
            // Check if attack hits
            if total_attack >= total_ac {
                // Check for special defense results
                let defense_result = if let Some(defender) = defender_comp {
                    defender.calculate_defense(&mut rng)
                } else {
                    DefenseResult::Hit
                };
                
                match defense_result {
                    DefenseResult::Hit => {
                        // Calculate damage
                        let base_damage = attacker_stats.power;
                        let is_critical = attacker_comp.map_or(false, |a| a.is_critical_hit(&mut rng));
                        
                        let mut final_damage = base_damage;
                        
                        // Apply critical hit
                        if is_critical {
                            let crit_multiplier = attacker_comp.map_or(2.0, |a| a.critical_multiplier);
                            final_damage = (final_damage as f32 * crit_multiplier) as i32;
                        }
                        
                        // Apply damage reduction
                        let damage_reduction = defender_comp.map_or(0, |d| d.damage_reduction);
                        final_damage = i32::max(1, final_damage - damage_reduction);
                        
                        // Apply damage resistances
                        if let Some(resistances) = damage_resistances.get(target_entity) {
                            final_damage = resistances.calculate_damage(final_damage, DamageType::Physical);
                        }
                        
                        // Create damage info
                        damage_info.insert(target_entity, DamageInfo {
                            base_damage: final_damage,
                            damage_type: DamageType::Physical,
                            source: attacker_entity,
                            is_critical,
                            penetration: 0,
                        }).expect("Failed to insert damage info");
                        
                        // Log the attack
                        if is_critical {
                            gamelog.add_entry(format!("{} critically hits {} for {} damage!", 
                                attacker_name, target_name, final_damage));
                        } else {
                            gamelog.add_entry(format!("{} hits {} for {} damage!", 
                                attacker_name, target_name, final_damage));
                        }
                    },
                    DefenseResult::Evaded => {
                        gamelog.add_entry(format!("{} attacks {} but misses!", attacker_name, target_name));
                    },
                    DefenseResult::Blocked => {
                        gamelog.add_entry(format!("{} attacks {} but the attack is blocked!", attacker_name, target_name));
                    },
                    DefenseResult::Parried => {
                        gamelog.add_entry(format!("{} attacks {} but the attack is parried!", attacker_name, target_name));
                    },
                }
            } else {
                gamelog.add_entry(format!("{} attacks {} but misses! (Attack: {} vs AC: {})", 
                    attacker_name, target_name, total_attack, total_ac));
            }
        }
    }
}

pub struct InitiativeSystem {}

impl<'a> System<'a> for InitiativeSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, Initiative>,
        ReadStorage<'a, CombatStats>,
        Write<'a, RandomNumberGenerator>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, mut initiatives, combat_stats, mut rng) = data;

        // Roll initiative for all entities that don't have it set
        for (entity, mut initiative, stats) in (&entities, &mut initiatives, &combat_stats).join() {
            if initiative.current_initiative == 0 {
                // Base initiative on dexterity/speed (using defense as a proxy for now)
                initiative.base_initiative = stats.defense + 10;
                initiative.roll_initiative(&mut rng);
            }
        }
    }
}

pub struct TurnOrderSystem {}

impl<'a> System<'a> for TurnOrderSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, Initiative>,
        ReadStorage<'a, Name>,
        Write<'a, GameLog>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, mut initiatives, names, mut gamelog) = data;

        // Collect all entities with initiative
        let mut turn_order: Vec<(Entity, i32)> = Vec::new();
        for (entity, initiative) in (&entities, &initiatives).join() {
            if !initiative.has_acted {
                turn_order.push((entity, initiative.current_initiative));
            }
        }
        
        // Sort by initiative (highest first)
        turn_order.sort_by(|a, b| b.1.cmp(&a.1));
        
        // Process the highest initiative entity
        if let Some((entity, _)) = turn_order.first() {
            if let Some(mut initiative) = initiatives.get_mut(*entity) {
                initiative.has_acted = true;
                
                // Log whose turn it is
                let entity_name = names.get(*entity).map_or("Unknown", |n| &n.name);
                gamelog.add_entry(format!("It's {}'s turn!", entity_name));
            }
        }
        
        // Check if all entities have acted (end of round)
        let all_acted = (&initiatives).join().all(|init| init.has_acted);
        if all_acted {
            // Reset for next round
            for (_, mut initiative) in (&entities, &mut initiatives).join() {
                initiative.has_acted = false;
            }
            gamelog.add_entry("--- New Combat Round ---".to_string());
        }
    }
}