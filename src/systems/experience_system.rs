use specs::{System, WriteStorage, ReadStorage, Entities, Entity, Join, ReadExpect, Write};
use crate::components::{Experience, Name, CombatStats, Attributes, Skills, Abilities, AbilityType, CharacterClass};
use crate::resources::GameLog;

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
        let mut leveled_up_entities = Vec::new();
        
        for (entity, exp, name) in (&entities, &mut experience, &names).join() {
            if exp.current >= exp.level_up_target {
                // Level up!
                exp.level_up();
                leveled_up_entities.push((entity, name.name.clone()));
                
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