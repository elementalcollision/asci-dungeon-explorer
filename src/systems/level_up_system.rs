use specs::{System, WriteStorage, ReadStorage, Entities, Entity, Join, ReadExpect, Write};
use crate::components::{Experience, Attributes, Skills, Abilities, AbilityType, CharacterClass, CombatStats, Name};
use crate::resources::GameLog;

// Event to signal that an entity has leveled up
pub struct LevelUpEvent {
    pub entity: Entity,
    pub new_level: i32,
}

pub struct ExperienceSystem {}

impl<'a> System<'a> for ExperienceSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, Experience>,
        ReadStorage<'a, Name>,
        WriteStorage<'a, CombatStats>,
        ReadStorage<'a, CharacterClass>,
        Write<'a, GameLog>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, mut experience, names, mut combat_stats, character_classes, mut gamelog) = data;

        // Check for level ups
        for (entity, exp, name) in (&entities, &mut experience, &names).join() {
            if exp.current >= exp.level_up_target {
                // Level up!
                exp.level_up();
                
                // Increase HP based on class
                if let Some(class) = character_classes.get(entity) {
                    if let Some(stats) = combat_stats.get_mut(entity) {
                        let hp_gain = class.class_type.hp_per_level();
                        stats.max_hp += hp_gain;
                        stats.hp += hp_gain; // Also heal on level up
                        
                        gamelog.add_entry(format!("{} leveled up to level {}! HP increased by {}.", 
                            name.name, exp.level, hp_gain));
                    }
                } else {
                    // Default HP gain if no class is found
                    if let Some(stats) = combat_stats.get_mut(entity) {
                        stats.max_hp += 5;
                        stats.hp += 5;
                        
                        gamelog.add_entry(format!("{} leveled up to level {}! HP increased by 5.", 
                            name.name, exp.level));
                    }
                }
            }
        }
    }
}

pub struct LevelUpSystem {}

impl<'a> System<'a> for LevelUpSystem {
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, Experience>,
        WriteStorage<'a, Attributes>,
        WriteStorage<'a, Skills>,
        WriteStorage<'a, Abilities>,
        ReadStorage<'a, CharacterClass>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, experience, mut attributes, mut skills, mut abilities, character_classes) = data;

        // Process entities with unspent attribute points
        for (entity, exp, attrs) in (&entities, &experience, &mut attributes).join() {
            if exp.unspent_points > 0 {
                // Add unspent points to attributes
                attrs.unspent_points += exp.unspent_points;
            }
        }
        
        // Process skill points (every 2 levels)
        for (entity, exp, mut skill_comp) in (&entities, &experience, &mut skills).join() {
            // Grant skill points every 2 levels
            if exp.level % 2 == 0 {
                skill_comp.unspent_skill_points += 1;
            }
        }
        
        // Check for ability unlocks based on level
        for (entity, exp, mut ability_comp, class) in (&entities, &experience, &mut abilities, &character_classes).join() {
            // Get abilities for this class
            let class_abilities = AbilityType::get_class_abilities(class.class_type);
            
            // Check each ability to see if it should be unlocked at this level
            for ability in class_abilities {
                if ability.required_level() <= exp.level && !ability_comp.has_ability(ability) {
                    ability_comp.add_ability(ability);
                }
            }
        }
    }
}