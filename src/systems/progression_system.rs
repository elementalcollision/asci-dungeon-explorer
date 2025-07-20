use specs::{System, WriteStorage, ReadStorage, Entities, Join, ReadExpect, WriteExpect};
use crate::components::*;
use crate::resources::GameLog;
use crate::game_state::RunState;

// System to handle attribute point allocation
pub struct AttributePointSystem;

impl<'a> System<'a> for AttributePointSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, Attributes>,
        ReadStorage<'a, Player>,
        WriteStorage<'a, Experience>,
        WriteExpect<'a, GameLog>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, mut attributes, players, mut experiences, mut game_log) = data;

        // This system doesn't do anything automatically
        // It's used by the UI to allocate attribute points
    }
    
    // Method to allocate an attribute point
    pub fn allocate_attribute_point(
        attributes: &mut WriteStorage<'a, Attributes>,
        experiences: &mut WriteStorage<'a, Experience>,
        game_log: &mut WriteExpect<'a, GameLog>,
        entity: Entity,
        attribute_type: AttributeType
    ) -> bool {
        if let (Some(attr), Some(exp)) = (attributes.get_mut(entity), experiences.get_mut(entity)) {
            if exp.unspent_points > 0 {
                // Check if attribute is at max (20)
                let current_value = match attribute_type {
                    AttributeType::Strength => attr.strength,
                    AttributeType::Dexterity => attr.dexterity,
                    AttributeType::Constitution => attr.constitution,
                    AttributeType::Intelligence => attr.intelligence,
                    AttributeType::Wisdom => attr.wisdom,
                    AttributeType::Charisma => attr.charisma,
                };
                
                if current_value < 20 {
                    // Allocate the point
                    match attribute_type {
                        AttributeType::Strength => attr.strength += 1,
                        AttributeType::Dexterity => attr.dexterity += 1,
                        AttributeType::Constitution => attr.constitution += 1,
                        AttributeType::Intelligence => attr.intelligence += 1,
                        AttributeType::Wisdom => attr.wisdom += 1,
                        AttributeType::Charisma => attr.charisma += 1,
                    }
                    
                    exp.unspent_points -= 1;
                    
                    // Add a message to the game log
                    game_log.add_entry(format!("You increased your {:?} to {}!", attribute_type, current_value + 1));
                    
                    return true;
                } else {
                    game_log.add_entry(format!("Your {:?} is already at maximum (20)!", attribute_type));
                }
            } else {
                game_log.add_entry("You don't have any attribute points to spend!".to_string());
            }
        }
        
        false
    }
}

// System to handle skill point allocation
pub struct SkillPointSystem;

impl<'a> System<'a> for SkillPointSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, Skills>,
        ReadStorage<'a, Player>,
        ReadStorage<'a, Attributes>,
        WriteExpect<'a, GameLog>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, mut skills, players, attributes, mut game_log) = data;

        // This system doesn't do anything automatically
        // It's used by the UI to allocate skill points
    }
    
    // Method to allocate a skill point
    pub fn allocate_skill_point(
        skills: &mut WriteStorage<'a, Skills>,
        attributes: &ReadStorage<'a, Attributes>,
        game_log: &mut WriteExpect<'a, GameLog>,
        entity: Entity,
        skill_type: SkillType
    ) -> bool {
        if let (Some(skill), Some(attr)) = (skills.get_mut(entity), attributes.get(entity)) {
            if skill.unspent_skill_points > 0 {
                let current_level = skill.get_skill_level(skill_type);
                
                // Check if skill is at max (5)
                if current_level < 5 {
                    // Check attribute requirements
                    let primary_attr = skill_type.primary_attribute();
                    let attr_value = match primary_attr {
                        AttributeType::Strength => attr.strength,
                        AttributeType::Dexterity => attr.dexterity,
                        AttributeType::Constitution => attr.constitution,
                        AttributeType::Intelligence => attr.intelligence,
                        AttributeType::Wisdom => attr.wisdom,
                        AttributeType::Charisma => attr.charisma,
                    };
                    
                    // Require attribute to be at least 3 * skill level
                    if attr_value >= (current_level + 1) * 3 {
                        // Allocate the point
                        if skill.increase_skill(skill_type) {
                            // Add a message to the game log
                            game_log.add_entry(format!("You increased your {} skill to {}!", 
                                skill_type.name(), current_level + 1));
                            
                            return true;
                        }
                    } else {
                        game_log.add_entry(format!("You need at least {} {:?} to increase this skill further!", 
                            (current_level + 1) * 3, primary_attr));
                    }
                } else {
                    game_log.add_entry(format!("Your {} skill is already at maximum (5)!", skill_type.name()));
                }
            } else {
                game_log.add_entry("You don't have any skill points to spend!".to_string());
            }
        }
        
        false
    }
}

// System to handle ability usage
pub struct AbilitySystem;

impl<'a> System<'a> for AbilitySystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, Abilities>,
        ReadStorage<'a, Player>,
        WriteExpect<'a, GameLog>,
        WriteExpect<'a, RunState>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, mut abilities, players, mut game_log, mut run_state) = data;

        // Update cooldowns
        for (_entity, ability) in (&entities, &mut abilities).join() {
            ability.update_cooldowns();
        }
        
        // The actual ability usage is handled by the UI and combat systems
    }
    
    // Method to use an ability
    pub fn use_ability(
        abilities: &mut WriteStorage<'a, Abilities>,
        game_log: &mut WriteExpect<'a, GameLog>,
        run_state: &mut WriteExpect<'a, RunState>,
        entity: Entity,
        ability_type: AbilityType
    ) -> bool {
        if let Some(ability) = abilities.get_mut(entity) {
            // Check if the entity has this ability
            if ability.has_ability(ability_type) {
                // Check if the ability is on cooldown
                if !ability.is_on_cooldown(ability_type) {
                    // Set the ability on cooldown
                    ability.set_cooldown(ability_type, ability_type.cooldown());
                    
                    // Add a message to the game log
                    game_log.add_entry(format!("You used {}!", ability_type.name()));
                    
                    // Handle ability effects based on type
                    match ability_type {
                        // Some abilities might need targeting
                        AbilityType::Fireball | AbilityType::IceSpike | AbilityType::MagicMissile => {
                            // Set run state to targeting mode
                            *run_state = RunState::ShowTargeting { 
                                range: 6, 
                                item: ability_type as usize 
                            };
                        },
                        // Other abilities have immediate effects
                        _ => {
                            // Ability effects would be handled here or by other systems
                        }
                    }
                    
                    return true;
                } else {
                    game_log.add_entry(format!("{} is on cooldown for {} more turns!", 
                        ability_type.name(), ability.get_cooldown(ability_type)));
                }
            } else {
                game_log.add_entry(format!("You don't have the {} ability!", ability_type.name()));
            }
        }
        
        false
    }
}