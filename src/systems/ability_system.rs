use specs::{System, WriteStorage, ReadStorage, Entities, Entity, Join};
use crate::components::{Abilities};

pub struct AbilitySystem {}

impl<'a> System<'a> for AbilitySystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, Abilities>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, mut abilities) = data;

        // Update ability cooldowns
        for (_entity, ability_comp) in (&entities, &mut abilities).join() {
            ability_comp.update_cooldowns();
        }
    }
}