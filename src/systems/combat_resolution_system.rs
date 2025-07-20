use specs::{System, WriteStorage, ReadStorage, Entities, Entity, Join, Write};
use crate::components::{
    WantsToAttack, CombatStats, Attacker, Defender, DamageInfo, DamageResistances, 
    DamageType, DefenseResult, Name, Player, Monster, Initiative, Attributes, Skills, SkillType
};
use crate::resources::{GameLog, RandomNumberGenerator};

pub struct CombatResolutionSystem {}

impl<'a> System<'a> for CombatResolutionSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, WantsToAttack>,
        ReadStorage<'a, CombatStats>,
        ReadStorage<'a, Attacker>,
        ReadStorage<'a, Defender>,
        WriteStorage<'a, DamageInfo>,
        ReadStorage<'a, DamageResistances>,
        ReadStorage<'a, Attributes>,
        ReadStorage<'a, Skills>,
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
            attributes,
            skills,
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
        
        // Process attacks with detailed resolution
        for (attacker_entity, target_entity) in attack_intents {
            let resolution = self.resolve_attack(
                attacker_entity,
                target_entity,
                &combat_stats,
                &attackers,
                &defenders,
                &attributes,
                &skills,
                &names,
                &mut rng,
                &mut gamelog
            );
            
            if let Some(damage) = resolution {
                // Apply damage resistances
                let final_damage = if let Some(resistances) = damage_resistances.get(target_entity) {
                    resistances.calculate_damage(damage.base_damage, damage.damage_type)
                } else {
                    damage.base_damage
                };
                
                // Create final damage info
                let final_damage_info = DamageInfo {
                    base_damage: final_damage,
                    damage_type: damage.damage_type,
                    source: damage.source,
                    is_critical: damage.is_critical,
                    penetration: damage.penetration,
                };
                
                damage_info.insert(target_entity, final_damage_info)
                    .expect("Failed to insert damage info");
            }
        }
    }
}

impl CombatResolutionSystem {
    fn resolve_attack(
        &self,
        attacker: Entity,
        target: Entity,
        combat_stats: &ReadStorage<CombatStats>,
        attackers: &ReadStorage<Attacker>,
        defenders: &ReadStorage<Defender>,
        attributes: &ReadStorage<Attributes>,
        skills: &ReadStorage<Skills>,
        names: &ReadStorage<Name>,
        rng: &mut RandomNumberGenerator,
        gamelog: &mut GameLog,
    ) -> Option<DamageInfo> {
        // Get required components
        let attacker_stats = combat_stats.get(attacker)?;
        let target_stats = combat_stats.get(target)?;
        
        // Get names for logging
        let attacker_name = names.get(attacker).map_or("Unknown", |n| &n.name);
        let target_name = names.get(target).map_or("Unknown", |n| &n.name);
        
        // Phase 1: Attack Roll Calculation
        let attack_result = self.calculate_attack_roll(
            attacker, attacker_stats, attackers, attributes, skills, rng
        );
        
        // Phase 2: Defense Calculation
        let defense_result = self.calculate_defense(
            target, target_stats, defenders, attributes, skills, rng
        );
        
        // Phase 3: Hit Determination
        if attack_result.total_attack < defense_result.total_defense {
            gamelog.add_entry(format!("{} attacks {} but misses! (Attack: {} vs Defense: {})", 
                attacker_name, target_name, attack_result.total_attack, defense_result.total_defense));
            return None;
        }
        
        // Phase 4: Special Defense Resolution
        match defense_result.special_result {
            DefenseResult::Evaded => {
                gamelog.add_entry(format!("{} attacks {} but the attack is evaded!", attacker_name, target_name));
                return None;
            },
            DefenseResult::Blocked => {
                gamelog.add_entry(format!("{} attacks {} but the attack is blocked!", attacker_name, target_name));
                return None;
            },
            DefenseResult::Parried => {
                gamelog.add_entry(format!("{} attacks {} but the attack is parried!", attacker_name, target_name));
                // Parry might allow a counter-attack in the future
                return None;
            },
            DefenseResult::Hit => {
                // Continue to damage calculation
            }
        }
        
        // Phase 5: Damage Calculation
        let damage_result = self.calculate_damage(
            attacker, attacker_stats, attackers, attributes, skills, 
            target, target_stats, defenders, rng
        );
        
        // Phase 6: Damage Type and Critical Hit Processing
        let final_damage = self.process_damage_modifiers(damage_result, rng);
        
        // Log the successful attack
        if final_damage.is_critical {
            gamelog.add_entry(format!("{} critically hits {} for {} {} damage!", 
                attacker_name, target_name, final_damage.base_damage, final_damage.damage_type.name()));
        } else {
            gamelog.add_entry(format!("{} hits {} for {} {} damage!", 
                attacker_name, target_name, final_damage.base_damage, final_damage.damage_type.name()));
        }
        
        Some(final_damage)
    }
    
    fn calculate_attack_roll(
        &self,
        attacker: Entity,
        stats: &CombatStats,
        attackers: &ReadStorage<Attacker>,
        attributes: &ReadStorage<Attributes>,
        skills: &ReadStorage<Skills>,
        rng: &mut RandomNumberGenerator,
    ) -> AttackResult {
        // Base attack roll (d20)
        let base_roll = rng.roll_dice(1, 20);
        
        // Attribute bonus (strength for melee)
        let attribute_bonus = if let Some(attrs) = attributes.get(attacker) {
            attrs.get_modifier(crate::components::AttributeType::Strength)
        } else {
            0
        };
        
        // Skill bonus (melee weapons skill)
        let skill_bonus = if let Some(skills) = skills.get(attacker) {
            skills.get_skill_level(SkillType::MeleeWeapons)
        } else {
            0
        };
        
        // Combat stats bonus
        let stats_bonus = stats.power / 2; // Half of power as attack bonus
        
        // Attacker component bonus
        let attacker_bonus = if let Some(attacker) = attackers.get(attacker) {
            attacker.attack_bonus
        } else {
            0
        };
        
        let total_attack = base_roll + attribute_bonus + skill_bonus + stats_bonus + attacker_bonus;
        
        AttackResult {
            base_roll,
            attribute_bonus,
            skill_bonus,
            stats_bonus,
            attacker_bonus,
            total_attack,
        }
    }
    
    fn calculate_defense(
        &self,
        defender: Entity,
        stats: &CombatStats,
        defenders: &ReadStorage<Defender>,
        attributes: &ReadStorage<Attributes>,
        skills: &ReadStorage<Skills>,
        rng: &mut RandomNumberGenerator,
    ) -> DefenseResult {
        // Base armor class
        let base_ac = if let Some(defender) = defenders.get(defender) {
            defender.armor_class
        } else {
            10 // Default AC
        };
        
        // Attribute bonus (dexterity for AC)
        let attribute_bonus = if let Some(attrs) = attributes.get(defender) {
            attrs.get_modifier(crate::components::AttributeType::Dexterity)
        } else {
            0
        };
        
        // Skill bonus (defense skill)
        let skill_bonus = if let Some(skills) = skills.get(defender) {
            skills.get_skill_level(SkillType::Defense)
        } else {
            0
        };
        
        // Combat stats bonus
        let stats_bonus = stats.defense;
        
        let total_defense = base_ac + attribute_bonus + skill_bonus + stats_bonus;
        
        // Check for special defense results
        let special_result = if let Some(defender) = defenders.get(defender) {
            defender.calculate_defense(rng)
        } else {
            crate::components::DefenseResult::Hit
        };
        
        DefenseResult {
            base_ac,
            attribute_bonus,
            skill_bonus,
            stats_bonus,
            total_defense,
            special_result,
        }
    }
    
    fn calculate_damage(
        &self,
        attacker: Entity,
        attacker_stats: &CombatStats,
        attackers: &ReadStorage<Attacker>,
        attributes: &ReadStorage<Attributes>,
        skills: &ReadStorage<Skills>,
        target: Entity,
        target_stats: &CombatStats,
        defenders: &ReadStorage<Defender>,
        rng: &mut RandomNumberGenerator,
    ) -> DamageResult {
        // Base damage from combat stats
        let base_damage = attacker_stats.power;
        
        // Attribute bonus (strength for melee damage)
        let attribute_bonus = if let Some(attrs) = attributes.get(attacker) {
            attrs.get_modifier(crate::components::AttributeType::Strength)
        } else {
            0
        };
        
        // Skill bonus (weapon skill affects damage)
        let skill_bonus = if let Some(skills) = skills.get(attacker) {
            skills.get_skill_level(SkillType::MeleeWeapons) / 2 // Half skill level as damage bonus
        } else {
            0
        };
        
        // Check for critical hit
        let is_critical = if let Some(attacker) = attackers.get(attacker) {
            attacker.is_critical_hit(rng)
        } else {
            false
        };
        
        // Calculate total damage before critical
        let total_damage = base_damage + attribute_bonus + skill_bonus;
        
        // Apply critical hit multiplier
        let final_damage = if is_critical {
            let multiplier = if let Some(attacker) = attackers.get(attacker) {
                attacker.critical_multiplier
            } else {
                2.0
            };
            (total_damage as f32 * multiplier) as i32
        } else {
            total_damage
        };
        
        // Apply damage reduction from defender
        let damage_reduction = if let Some(defender) = defenders.get(target) {
            defender.damage_reduction
        } else {
            0
        };
        
        let reduced_damage = i32::max(1, final_damage - damage_reduction); // Minimum 1 damage
        
        DamageResult {
            base_damage,
            attribute_bonus,
            skill_bonus,
            is_critical,
            damage_reduction,
            final_damage: reduced_damage,
        }
    }
    
    fn process_damage_modifiers(
        &self,
        damage_result: DamageResult,
        _rng: &mut RandomNumberGenerator,
    ) -> DamageInfo {
        // For now, default to physical damage
        // This could be expanded to determine damage type based on weapon, spell, etc.
        DamageInfo {
            base_damage: damage_result.final_damage,
            damage_type: DamageType::Physical,
            source: Entity::from_raw(0), // This would be set properly in the calling function
            is_critical: damage_result.is_critical,
            penetration: 0, // Could be calculated based on weapon properties
        }
    }
}

// Helper structs for combat resolution
#[derive(Debug)]
struct AttackResult {
    base_roll: i32,
    attribute_bonus: i32,
    skill_bonus: i32,
    stats_bonus: i32,
    attacker_bonus: i32,
    total_attack: i32,
}

#[derive(Debug)]
struct DefenseResult {
    base_ac: i32,
    attribute_bonus: i32,
    skill_bonus: i32,
    stats_bonus: i32,
    total_defense: i32,
    special_result: crate::components::DefenseResult,
}

#[derive(Debug)]
struct DamageResult {
    base_damage: i32,
    attribute_bonus: i32,
    skill_bonus: i32,
    is_critical: bool,
    damage_reduction: i32,
    final_damage: i32,
}