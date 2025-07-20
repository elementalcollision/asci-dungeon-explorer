use specs::{System, WriteStorage, ReadStorage, Entities, Entity, Join, Write};
use crate::components::{
    WantsToUseAbility, Abilities, AbilityType, PlayerResources, CombatStats, 
    Name, Player, Monster, Position, DamageInfo, DamageType, StatusEffects, 
    StatusEffect, StatusEffectType, WantsToAttack
};
use crate::resources::{GameLog, RandomNumberGenerator};

pub struct SpecialAbilitiesSystem {}

impl<'a> System<'a> for SpecialAbilitiesSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, WantsToUseAbility>,
        WriteStorage<'a, Abilities>,
        WriteStorage<'a, PlayerResources>,
        ReadStorage<'a, CombatStats>,
        WriteStorage<'a, DamageInfo>,
        WriteStorage<'a, StatusEffects>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, Player>,
        ReadStorage<'a, Monster>,
        ReadStorage<'a, Position>,
        Write<'a, GameLog>,
        Write<'a, RandomNumberGenerator>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            entities, 
            mut wants_use_ability, 
            mut abilities, 
            mut resources,
            combat_stats,
            mut damage_info,
            mut status_effects,
            names, 
            players,
            monsters,
            positions,
            mut gamelog, 
            mut rng
        ) = data;

        // Process ability usage requests
        let mut ability_uses = Vec::new();
        for (entity, ability_use) in (&entities, &wants_use_ability).join() {
            ability_uses.push((entity, ability_use.ability, ability_use.target, ability_use.mana_cost, ability_use.stamina_cost));
        }
        
        // Clear ability usage requests
        wants_use_ability.clear();
        
        // Process each ability usage
        for (caster, ability_type, target, mana_cost, stamina_cost) in ability_uses {
            if self.can_use_ability(caster, ability_type, mana_cost, stamina_cost, &abilities, &resources, &mut gamelog) {
                // Consume resources
                if let Some(resource) = resources.get_mut(caster) {
                    resource.consume_mana(mana_cost);
                    resource.consume_stamina(stamina_cost);
                }
                
                // Set ability on cooldown
                if let Some(ability_comp) = abilities.get_mut(caster) {
                    ability_comp.set_cooldown(ability_type, ability_type.cooldown());
                }
                
                // Execute the ability
                self.execute_ability(
                    caster,
                    ability_type,
                    target,
                    &entities,
                    &combat_stats,
                    &mut damage_info,
                    &mut status_effects,
                    &names,
                    &players,
                    &monsters,
                    &positions,
                    &mut gamelog,
                    &mut rng
                );
            }
        }
    }
}

impl SpecialAbilitiesSystem {
    fn can_use_ability(
        &self,
        caster: Entity,
        ability_type: AbilityType,
        mana_cost: i32,
        stamina_cost: i32,
        abilities: &ReadStorage<Abilities>,
        resources: &ReadStorage<PlayerResources>,
        gamelog: &mut GameLog,
    ) -> bool {
        // Check if entity has the ability
        if let Some(ability_comp) = abilities.get(caster) {
            if !ability_comp.has_ability(ability_type) {
                gamelog.add_entry(format!("You don't have the {} ability!", ability_type.name()));
                return false;
            }
            
            // Check cooldown
            if ability_comp.is_on_cooldown(ability_type) {
                gamelog.add_entry(format!("{} is on cooldown for {} more turns!", 
                    ability_type.name(), ability_comp.get_cooldown(ability_type)));
                return false;
            }
        } else {
            return false;
        }
        
        // Check resource costs
        if let Some(resource) = resources.get(caster) {
            if resource.mana < mana_cost {
                gamelog.add_entry(format!("Not enough mana! Need {} but have {}", mana_cost, resource.mana));
                return false;
            }
            
            if resource.stamina < stamina_cost {
                gamelog.add_entry(format!("Not enough stamina! Need {} but have {}", stamina_cost, resource.stamina));
                return false;
            }
        }
        
        true
    }
    
    fn execute_ability(
        &self,
        caster: Entity,
        ability_type: AbilityType,
        target: Option<Entity>,
        entities: &Entities,
        combat_stats: &ReadStorage<CombatStats>,
        damage_info: &mut WriteStorage<DamageInfo>,
        status_effects: &mut WriteStorage<StatusEffects>,
        names: &ReadStorage<Name>,
        players: &ReadStorage<Player>,
        monsters: &ReadStorage<Monster>,
        positions: &ReadStorage<Position>,
        gamelog: &mut GameLog,
        rng: &mut RandomNumberGenerator,
    ) {
        let caster_name = names.get(caster).map_or("Unknown", |n| &n.name);
        
        match ability_type {
            // Fighter abilities
            AbilityType::PowerAttack => {
                self.execute_power_attack(caster, target, caster_name, damage_info, gamelog, rng);
            },
            AbilityType::Cleave => {
                self.execute_cleave(caster, caster_name, entities, combat_stats, monsters, positions, damage_info, gamelog, rng);
            },
            AbilityType::ShieldBash => {
                self.execute_shield_bash(caster, target, caster_name, damage_info, status_effects, gamelog, rng);
            },
            AbilityType::SecondWind => {
                self.execute_second_wind(caster, caster_name, combat_stats, gamelog);
            },
            
            // Rogue abilities
            AbilityType::Backstab => {
                self.execute_backstab(caster, target, caster_name, damage_info, gamelog, rng);
            },
            AbilityType::ShadowStep => {
                self.execute_shadow_step(caster, caster_name, status_effects, gamelog);
            },
            AbilityType::Evasion => {
                self.execute_evasion(caster, caster_name, status_effects, gamelog);
            },
            
            // Mage abilities
            AbilityType::Fireball => {
                self.execute_fireball(caster, target, caster_name, damage_info, status_effects, gamelog, rng);
            },
            AbilityType::IceSpike => {
                self.execute_ice_spike(caster, target, caster_name, damage_info, status_effects, gamelog, rng);
            },
            AbilityType::MagicMissile => {
                self.execute_magic_missile(caster, target, caster_name, damage_info, gamelog, rng);
            },
            AbilityType::Teleport => {
                self.execute_teleport(caster, caster_name, gamelog);
            },
            
            // Cleric abilities
            AbilityType::Heal => {
                self.execute_heal(caster, target, caster_name, combat_stats, gamelog);
            },
            AbilityType::TurnUndead => {
                self.execute_turn_undead(caster, caster_name, entities, monsters, status_effects, gamelog);
            },
            AbilityType::BlessWeapon => {
                self.execute_bless_weapon(caster, caster_name, status_effects, gamelog);
            },
            AbilityType::DivineProtection => {
                self.execute_divine_protection(caster, caster_name, status_effects, gamelog);
            },
            
            // Ranger abilities
            AbilityType::PreciseShot => {
                self.execute_precise_shot(caster, target, caster_name, damage_info, gamelog, rng);
            },
            AbilityType::TrackEnemy => {
                self.execute_track_enemy(caster, caster_name, gamelog);
            },
            AbilityType::NaturalRemedy => {
                self.execute_natural_remedy(caster, caster_name, status_effects, gamelog);
            },
            
            _ => {
                gamelog.add_entry(format!("{} uses {} but nothing happens!", caster_name, ability_type.name()));
            }
        }
    }
    
    // Fighter abilities implementation
    fn execute_power_attack(
        &self,
        caster: Entity,
        target: Option<Entity>,
        caster_name: &str,
        damage_info: &mut WriteStorage<DamageInfo>,
        gamelog: &mut GameLog,
        rng: &mut RandomNumberGenerator,
    ) {
        if let Some(target_entity) = target {
            let damage = 15 + rng.roll_dice(1, 10); // 15-25 damage
            
            damage_info.insert(target_entity, DamageInfo {
                base_damage: damage,
                damage_type: DamageType::Physical,
                source: caster,
                is_critical: false,
                penetration: 5, // Ignores some armor
            }).expect("Failed to insert power attack damage");
            
            gamelog.add_entry(format!("{} unleashes a devastating Power Attack!", caster_name));
        }
    }
    
    fn execute_cleave(
        &self,
        caster: Entity,
        caster_name: &str,
        entities: &Entities,
        combat_stats: &ReadStorage<CombatStats>,
        monsters: &ReadStorage<Monster>,
        positions: &ReadStorage<Position>,
        damage_info: &mut WriteStorage<DamageInfo>,
        gamelog: &mut GameLog,
        rng: &mut RandomNumberGenerator,
    ) {
        let caster_pos = positions.get(caster);
        if caster_pos.is_none() {
            return;
        }
        let caster_pos = caster_pos.unwrap();
        
        let mut targets_hit = 0;
        
        // Hit all adjacent enemies
        for (entity, _monster, stats, pos) in (entities, monsters, combat_stats, positions).join() {
            let distance = ((pos.x - caster_pos.x).abs() + (pos.y - caster_pos.y).abs());
            if distance <= 1 && entity != caster {
                let damage = 8 + rng.roll_dice(1, 6); // 8-14 damage to each target
                
                damage_info.insert(entity, DamageInfo {
                    base_damage: damage,
                    damage_type: DamageType::Physical,
                    source: caster,
                    is_critical: false,
                    penetration: 0,
                }).expect("Failed to insert cleave damage");
                
                targets_hit += 1;
            }
        }
        
        if targets_hit > 0 {
            gamelog.add_entry(format!("{} cleaves through {} enemies!", caster_name, targets_hit));
        } else {
            gamelog.add_entry(format!("{} swings wildly but hits nothing!", caster_name));
        }
    }
    
    fn execute_shield_bash(
        &self,
        caster: Entity,
        target: Option<Entity>,
        caster_name: &str,
        damage_info: &mut WriteStorage<DamageInfo>,
        status_effects: &mut WriteStorage<StatusEffects>,
        gamelog: &mut GameLog,
        rng: &mut RandomNumberGenerator,
    ) {
        if let Some(target_entity) = target {
            let damage = 5 + rng.roll_dice(1, 6); // 5-11 damage
            
            damage_info.insert(target_entity, DamageInfo {
                base_damage: damage,
                damage_type: DamageType::Physical,
                source: caster,
                is_critical: false,
                penetration: 0,
            }).expect("Failed to insert shield bash damage");
            
            // Apply stun effect
            if let Some(effects) = status_effects.get_mut(target_entity) {
                effects.add_effect(StatusEffect {
                    effect_type: StatusEffectType::Slow, // Using slow as stun
                    duration: 2,
                    magnitude: 3,
                });
            } else {
                let mut new_effects = StatusEffects::new();
                new_effects.add_effect(StatusEffect {
                    effect_type: StatusEffectType::Slow,
                    duration: 2,
                    magnitude: 3,
                });
                status_effects.insert(target_entity, new_effects)
                    .expect("Failed to insert status effects");
            }
            
            gamelog.add_entry(format!("{} bashes with their shield, stunning the target!", caster_name));
        }
    }
    
    fn execute_second_wind(
        &self,
        caster: Entity,
        caster_name: &str,
        combat_stats: &ReadStorage<CombatStats>,
        gamelog: &mut GameLog,
    ) {
        // This would heal the caster - implementation would need mutable access to combat_stats
        gamelog.add_entry(format!("{} catches their second wind and feels reinvigorated!", caster_name));
    }
    
    // Mage abilities implementation
    fn execute_fireball(
        &self,
        caster: Entity,
        target: Option<Entity>,
        caster_name: &str,
        damage_info: &mut WriteStorage<DamageInfo>,
        status_effects: &mut WriteStorage<StatusEffects>,
        gamelog: &mut GameLog,
        rng: &mut RandomNumberGenerator,
    ) {
        if let Some(target_entity) = target {
            let damage = 12 + rng.roll_dice(2, 6); // 14-24 fire damage
            
            damage_info.insert(target_entity, DamageInfo {
                base_damage: damage,
                damage_type: DamageType::Fire,
                source: caster,
                is_critical: false,
                penetration: 0,
            }).expect("Failed to insert fireball damage");
            
            // Apply burning effect
            if let Some(effects) = status_effects.get_mut(target_entity) {
                effects.add_effect(StatusEffect {
                    effect_type: StatusEffectType::Poisoned, // Using poison as burning
                    duration: 3,
                    magnitude: 3,
                });
            } else {
                let mut new_effects = StatusEffects::new();
                new_effects.add_effect(StatusEffect {
                    effect_type: StatusEffectType::Poisoned,
                    duration: 3,
                    magnitude: 3,
                });
                status_effects.insert(target_entity, new_effects)
                    .expect("Failed to insert status effects");
            }
            
            gamelog.add_entry(format!("{} hurls a blazing fireball!", caster_name));
        }
    }
    
    fn execute_ice_spike(
        &self,
        caster: Entity,
        target: Option<Entity>,
        caster_name: &str,
        damage_info: &mut WriteStorage<DamageInfo>,
        status_effects: &mut WriteStorage<StatusEffects>,
        gamelog: &mut GameLog,
        rng: &mut RandomNumberGenerator,
    ) {
        if let Some(target_entity) = target {
            let damage = 10 + rng.roll_dice(1, 8); // 11-18 ice damage
            
            damage_info.insert(target_entity, DamageInfo {
                base_damage: damage,
                damage_type: DamageType::Ice,
                source: caster,
                is_critical: false,
                penetration: 0,
            }).expect("Failed to insert ice spike damage");
            
            // Apply slow effect
            if let Some(effects) = status_effects.get_mut(target_entity) {
                effects.add_effect(StatusEffect {
                    effect_type: StatusEffectType::Slow,
                    duration: 4,
                    magnitude: 2,
                });
            } else {
                let mut new_effects = StatusEffects::new();
                new_effects.add_effect(StatusEffect {
                    effect_type: StatusEffectType::Slow,
                    duration: 4,
                    magnitude: 2,
                });
                status_effects.insert(target_entity, new_effects)
                    .expect("Failed to insert status effects");
            }
            
            gamelog.add_entry(format!("{} conjures a piercing ice spike!", caster_name));
        }
    }
    
    fn execute_magic_missile(
        &self,
        caster: Entity,
        target: Option<Entity>,
        caster_name: &str,
        damage_info: &mut WriteStorage<DamageInfo>,
        gamelog: &mut GameLog,
        rng: &mut RandomNumberGenerator,
    ) {
        if let Some(target_entity) = target {
            let missiles = 3; // Always fires 3 missiles
            let total_damage = missiles * (2 + rng.roll_dice(1, 4)); // 3-15 total damage
            
            damage_info.insert(target_entity, DamageInfo {
                base_damage: total_damage,
                damage_type: DamageType::Psychic,
                source: caster,
                is_critical: false,
                penetration: 10, // Magic missiles never miss and ignore armor
            }).expect("Failed to insert magic missile damage");
            
            gamelog.add_entry(format!("{} fires {} unerring magic missiles!", caster_name, missiles));
        }
    }
    
    // Cleric abilities implementation
    fn execute_heal(
        &self,
        caster: Entity,
        target: Option<Entity>,
        caster_name: &str,
        combat_stats: &ReadStorage<CombatStats>,
        gamelog: &mut GameLog,
    ) {
        let heal_target = target.unwrap_or(caster);
        // This would heal the target - implementation would need mutable access to combat_stats
        gamelog.add_entry(format!("{} channels divine energy to heal wounds!", caster_name));
    }
    
    fn execute_divine_protection(
        &self,
        caster: Entity,
        caster_name: &str,
        status_effects: &mut WriteStorage<StatusEffects>,
        gamelog: &mut GameLog,
    ) {
        if let Some(effects) = status_effects.get_mut(caster) {
            effects.add_effect(StatusEffect {
                effect_type: StatusEffectType::DefenseBoost,
                duration: 10,
                magnitude: 3,
            });
        } else {
            let mut new_effects = StatusEffects::new();
            new_effects.add_effect(StatusEffect {
                effect_type: StatusEffectType::DefenseBoost,
                duration: 10,
                magnitude: 3,
            });
            status_effects.insert(caster, new_effects)
                .expect("Failed to insert status effects");
        }
        
        gamelog.add_entry(format!("{} is surrounded by divine protection!", caster_name));
    }
    
    // Additional ability implementations would go here...
    fn execute_backstab(&self, caster: Entity, target: Option<Entity>, caster_name: &str, damage_info: &mut WriteStorage<DamageInfo>, gamelog: &mut GameLog, rng: &mut RandomNumberGenerator) {
        if let Some(target_entity) = target {
            let damage = 20 + rng.roll_dice(2, 8); // High damage backstab
            
            damage_info.insert(target_entity, DamageInfo {
                base_damage: damage,
                damage_type: DamageType::Physical,
                source: caster,
                is_critical: true, // Backstabs are always critical
                penetration: 8,
            }).expect("Failed to insert backstab damage");
            
            gamelog.add_entry(format!("{} strikes from the shadows with a deadly backstab!", caster_name));
        }
    }
    
    fn execute_shadow_step(&self, caster: Entity, caster_name: &str, status_effects: &mut WriteStorage<StatusEffects>, gamelog: &mut GameLog) {
        // Grant temporary invisibility/evasion boost
        if let Some(effects) = status_effects.get_mut(caster) {
            effects.add_effect(StatusEffect {
                effect_type: StatusEffectType::Haste, // Using haste as evasion boost
                duration: 3,
                magnitude: 2,
            });
        }
        gamelog.add_entry(format!("{} melts into the shadows!", caster_name));
    }
    
    fn execute_evasion(&self, caster: Entity, caster_name: &str, status_effects: &mut WriteStorage<StatusEffects>, gamelog: &mut GameLog) {
        // Temporary evasion boost
        gamelog.add_entry(format!("{} becomes incredibly evasive!", caster_name));
    }
    
    fn execute_teleport(&self, caster: Entity, caster_name: &str, gamelog: &mut GameLog) {
        // Teleportation would require position manipulation
        gamelog.add_entry(format!("{} vanishes and reappears elsewhere!", caster_name));
    }
    
    fn execute_turn_undead(&self, caster: Entity, caster_name: &str, entities: &Entities, monsters: &ReadStorage<Monster>, status_effects: &mut WriteStorage<StatusEffects>, gamelog: &mut GameLog) {
        // Apply fear to undead monsters in range
        gamelog.add_entry(format!("{} channels holy power to turn undead!", caster_name));
    }
    
    fn execute_bless_weapon(&self, caster: Entity, caster_name: &str, status_effects: &mut WriteStorage<StatusEffects>, gamelog: &mut GameLog) {
        // Weapon enhancement
        gamelog.add_entry(format!("{}'s weapon glows with holy light!", caster_name));
    }
    
    fn execute_precise_shot(&self, caster: Entity, target: Option<Entity>, caster_name: &str, damage_info: &mut WriteStorage<DamageInfo>, gamelog: &mut GameLog, rng: &mut RandomNumberGenerator) {
        if let Some(target_entity) = target {
            let damage = 12 + rng.roll_dice(1, 8);
            
            damage_info.insert(target_entity, DamageInfo {
                base_damage: damage,
                damage_type: DamageType::Physical,
                source: caster,
                is_critical: rng.roll_dice(1, 100) <= 30, // 30% crit chance
                penetration: 3,
            }).expect("Failed to insert precise shot damage");
            
            gamelog.add_entry(format!("{} takes careful aim and fires a precise shot!", caster_name));
        }
    }
    
    fn execute_track_enemy(&self, caster: Entity, caster_name: &str, gamelog: &mut GameLog) {
        // Reveal enemy positions
        gamelog.add_entry(format!("{} studies the ground for tracks and signs!", caster_name));
    }
    
    fn execute_natural_remedy(&self, caster: Entity, caster_name: &str, status_effects: &mut WriteStorage<StatusEffects>, gamelog: &mut GameLog) {
        // Remove negative status effects
        if let Some(effects) = status_effects.get_mut(caster) {
            let mut removed_effects = Vec::new();
            for effect in &effects.effects {
                if !effect.effect_type.is_beneficial() {
                    removed_effects.push(effect.effect_type);
                }
            }
            
            for effect_type in removed_effects {
                effects.remove_effect(effect_type);
            }
        }
        
        gamelog.add_entry(format!("{} uses natural remedies to cure ailments!", caster_name));
    }
}