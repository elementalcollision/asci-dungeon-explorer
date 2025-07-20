use specs::{System, WriteStorage, ReadStorage, Entities, Entity, Join, Write};
use crate::components::{
    Position, AbilityType, WantsToUseAbility, Player, Monster, Name, CombatStats
};
use crate::resources::GameLog;

pub struct AbilityTargetingSystem {}

impl<'a> System<'a> for AbilityTargetingSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, WantsToUseAbility>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, Player>,
        ReadStorage<'a, Monster>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, CombatStats>,
        Write<'a, GameLog>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, mut wants_use_ability, positions, players, monsters, names, combat_stats, mut gamelog) = data;

        // Process targeting for abilities that need it
        let mut targeting_requests = Vec::new();
        
        for (entity, ability_use) in (&entities, &wants_use_ability).join() {
            if ability_use.target.is_none() && self.ability_needs_target(ability_use.ability) {
                targeting_requests.push((entity, ability_use.ability));
            }
        }
        
        // Auto-target for abilities that need targets
        for (caster, ability_type) in targeting_requests {
            let target = self.find_best_target(
                caster,
                ability_type,
                &entities,
                &positions,
                &players,
                &monsters,
                &names,
                &combat_stats
            );
            
            if let Some(target_entity) = target {
                // Update the ability use with the target
                if let Some(ability_use) = wants_use_ability.get_mut(caster) {
                    ability_use.target = Some(target_entity);
                    
                    let caster_name = names.get(caster).map_or("Unknown", |n| &n.name);
                    let target_name = names.get(target_entity).map_or("Unknown", |n| &n.name);
                    
                    gamelog.add_entry(format!("{} targets {} with {}!", 
                        caster_name, target_name, ability_type.name()));
                }
            } else {
                // No valid target found, remove the ability use
                wants_use_ability.remove(caster);
                
                let caster_name = names.get(caster).map_or("Unknown", |n| &n.name);
                gamelog.add_entry(format!("{} cannot find a valid target for {}!", 
                    caster_name, ability_type.name()));
            }
        }
    }
}

impl AbilityTargetingSystem {
    fn ability_needs_target(&self, ability: AbilityType) -> bool {
        match ability {
            // Offensive abilities that need targets
            AbilityType::PowerAttack |
            AbilityType::ShieldBash |
            AbilityType::Backstab |
            AbilityType::Fireball |
            AbilityType::IceSpike |
            AbilityType::MagicMissile |
            AbilityType::PreciseShot => true,
            
            // Healing can target self or others
            AbilityType::Heal => false, // Can target self if no target specified
            
            // Self-buff abilities don't need targets
            AbilityType::SecondWind |
            AbilityType::Evasion |
            AbilityType::ShadowStep |
            AbilityType::Teleport |
            AbilityType::BlessWeapon |
            AbilityType::DivineProtection |
            AbilityType::TrackEnemy |
            AbilityType::NaturalRemedy => false,
            
            // Area effect abilities don't need specific targets
            AbilityType::Cleave |
            AbilityType::TurnUndead => false,
            
            _ => false,
        }
    }
    
    fn find_best_target(
        &self,
        caster: Entity,
        ability_type: AbilityType,
        entities: &Entities,
        positions: &ReadStorage<Position>,
        players: &ReadStorage<Player>,
        monsters: &ReadStorage<Monster>,
        names: &ReadStorage<Name>,
        combat_stats: &ReadStorage<CombatStats>,
    ) -> Option<Entity> {
        let caster_pos = positions.get(caster)?;
        let is_caster_player = players.contains(caster);
        
        let max_range = self.get_ability_range(ability_type);
        let mut best_target = None;
        let mut best_score = f32::MIN;
        
        // Find targets based on ability type and caster type
        for (entity, pos, stats) in (entities, positions, combat_stats).join() {
            if entity == caster {
                continue; // Don't target self for offensive abilities
            }
            
            // Calculate distance
            let distance = ((pos.x - caster_pos.x).pow(2) + (pos.y - caster_pos.y).pow(2)) as f32;
            let distance = distance.sqrt();
            
            if distance > max_range as f32 {
                continue; // Out of range
            }
            
            // Determine if this is a valid target
            let is_valid_target = match ability_type {
                // Offensive abilities
                AbilityType::PowerAttack |
                AbilityType::ShieldBash |
                AbilityType::Backstab |
                AbilityType::Fireball |
                AbilityType::IceSpike |
                AbilityType::MagicMissile |
                AbilityType::PreciseShot => {
                    // Players target monsters, monsters target players
                    if is_caster_player {
                        monsters.contains(entity)
                    } else {
                        players.contains(entity)
                    }
                },
                
                // Healing abilities
                AbilityType::Heal => {
                    // Target allies with missing health
                    let is_ally = if is_caster_player {
                        players.contains(entity)
                    } else {
                        monsters.contains(entity)
                    };
                    
                    is_ally && stats.hp < stats.max_hp
                },
                
                _ => false,
            };
            
            if !is_valid_target {
                continue;
            }
            
            // Calculate targeting score
            let score = self.calculate_target_score(
                ability_type,
                entity,
                distance,
                stats,
                names
            );
            
            if score > best_score {
                best_score = score;
                best_target = Some(entity);
            }
        }
        
        best_target
    }
    
    fn get_ability_range(&self, ability_type: AbilityType) -> i32 {
        match ability_type {
            // Melee abilities
            AbilityType::PowerAttack |
            AbilityType::ShieldBash |
            AbilityType::Backstab => 1,
            
            // Ranged abilities
            AbilityType::Fireball |
            AbilityType::IceSpike |
            AbilityType::MagicMissile |
            AbilityType::PreciseShot => 8,
            
            // Healing
            AbilityType::Heal => 5,
            
            _ => 1,
        }
    }
    
    fn calculate_target_score(
        &self,
        ability_type: AbilityType,
        target: Entity,
        distance: f32,
        stats: &CombatStats,
        names: &ReadStorage<Name>,
    ) -> f32 {
        let mut score = 0.0;
        
        match ability_type {
            // Offensive abilities prefer closer, weaker targets
            AbilityType::PowerAttack |
            AbilityType::ShieldBash |
            AbilityType::Backstab |
            AbilityType::Fireball |
            AbilityType::IceSpike |
            AbilityType::MagicMissile |
            AbilityType::PreciseShot => {
                // Prefer closer targets
                score += 10.0 - distance;
                
                // Prefer targets with lower health (easier to kill)
                let health_percentage = stats.hp as f32 / stats.max_hp as f32;
                score += (1.0 - health_percentage) * 5.0;
                
                // Prefer targets with lower defense
                score += (20.0 - stats.defense as f32) * 0.1;
            },
            
            // Healing abilities prefer closer allies with more missing health
            AbilityType::Heal => {
                // Prefer closer targets
                score += 10.0 - distance;
                
                // Prefer targets with more missing health
                let missing_health = stats.max_hp - stats.hp;
                score += missing_health as f32 * 0.5;
                
                // Slightly prefer targets with higher max health (more important)
                score += stats.max_hp as f32 * 0.1;
            },
            
            _ => {
                score = distance; // Default: prefer closer targets
            }
        }
        
        score
    }
}

// System for managing ability cooldowns and resource costs
pub struct AbilityCooldownSystem {}

impl<'a> System<'a> for AbilityCooldownSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, crate::components::Abilities>,
        ReadStorage<'a, Name>,
        Write<'a, GameLog>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, mut abilities, names, mut gamelog) = data;

        // Update cooldowns for all entities
        for (entity, mut ability_comp) in (&entities, &mut abilities).join() {
            let mut cooldowns_updated = Vec::new();
            
            // Check which abilities came off cooldown this turn
            for ability_type in AbilityType::get_all_abilities() {
                if ability_comp.has_ability(ability_type) {
                    let old_cooldown = ability_comp.get_cooldown(ability_type);
                    if old_cooldown == 1 { // About to come off cooldown
                        cooldowns_updated.push(ability_type);
                    }
                }
            }
            
            // Update cooldowns
            ability_comp.update_cooldowns();
            
            // Log abilities that came off cooldown
            if !cooldowns_updated.is_empty() {
                let entity_name = names.get(entity).map_or("Unknown", |n| &n.name);
                for ability_type in cooldowns_updated {
                    gamelog.add_entry(format!("{}'s {} is ready to use again!", 
                        entity_name, ability_type.name()));
                }
            }
        }
    }
}

// Extension to AbilityType to get all abilities
impl AbilityType {
    pub fn get_all_abilities() -> Vec<AbilityType> {
        vec![
            // Fighter abilities
            AbilityType::PowerAttack,
            AbilityType::Cleave,
            AbilityType::ShieldBash,
            AbilityType::SecondWind,
            
            // Rogue abilities
            AbilityType::Backstab,
            AbilityType::Evasion,
            AbilityType::ShadowStep,
            AbilityType::DisarmTrap,
            
            // Mage abilities
            AbilityType::Fireball,
            AbilityType::IceSpike,
            AbilityType::MagicMissile,
            AbilityType::Teleport,
            
            // Cleric abilities
            AbilityType::Heal,
            AbilityType::TurnUndead,
            AbilityType::BlessWeapon,
            AbilityType::DivineProtection,
            
            // Ranger abilities
            AbilityType::PreciseShot,
            AbilityType::AnimalCompanion,
            AbilityType::TrackEnemy,
            AbilityType::NaturalRemedy,
        ]
    }
    
    pub fn get_mana_cost(&self) -> i32 {
        match self {
            // Fighter abilities (low mana cost)
            AbilityType::PowerAttack => 5,
            AbilityType::Cleave => 8,
            AbilityType::ShieldBash => 3,
            AbilityType::SecondWind => 10,
            
            // Rogue abilities (medium mana cost)
            AbilityType::Backstab => 6,
            AbilityType::Evasion => 8,
            AbilityType::ShadowStep => 12,
            AbilityType::DisarmTrap => 0,
            
            // Mage abilities (high mana cost)
            AbilityType::Fireball => 15,
            AbilityType::IceSpike => 12,
            AbilityType::MagicMissile => 10,
            AbilityType::Teleport => 20,
            
            // Cleric abilities (medium-high mana cost)
            AbilityType::Heal => 12,
            AbilityType::TurnUndead => 15,
            AbilityType::BlessWeapon => 10,
            AbilityType::DivineProtection => 18,
            
            // Ranger abilities (low-medium mana cost)
            AbilityType::PreciseShot => 4,
            AbilityType::AnimalCompanion => 25,
            AbilityType::TrackEnemy => 6,
            AbilityType::NaturalRemedy => 8,
        }
    }
    
    pub fn get_stamina_cost(&self) -> i32 {
        match self {
            // Fighter abilities (high stamina cost)
            AbilityType::PowerAttack => 8,
            AbilityType::Cleave => 12,
            AbilityType::ShieldBash => 6,
            AbilityType::SecondWind => 5,
            
            // Rogue abilities (medium stamina cost)
            AbilityType::Backstab => 10,
            AbilityType::Evasion => 8,
            AbilityType::ShadowStep => 6,
            AbilityType::DisarmTrap => 4,
            
            // Mage abilities (low stamina cost)
            AbilityType::Fireball => 3,
            AbilityType::IceSpike => 3,
            AbilityType::MagicMissile => 2,
            AbilityType::Teleport => 5,
            
            // Cleric abilities (low-medium stamina cost)
            AbilityType::Heal => 4,
            AbilityType::TurnUndead => 6,
            AbilityType::BlessWeapon => 3,
            AbilityType::DivineProtection => 8,
            
            // Ranger abilities (medium stamina cost)
            AbilityType::PreciseShot => 6,
            AbilityType::AnimalCompanion => 10,
            AbilityType::TrackEnemy => 4,
            AbilityType::NaturalRemedy => 5,
        }
    }
}