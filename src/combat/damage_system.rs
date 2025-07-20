use specs::{System, WriteStorage, ReadStorage, Entities, Entity, Join, ReadExpect, Write};
use crate::components::{SufferDamage, CombatStats, Player, Name};
use crate::resources::GameLog;

pub struct DamageSystem {}

impl<'a> System<'a> for DamageSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, CombatStats>,
        WriteStorage<'a, SufferDamage>,
        ReadStorage<'a, Player>,
        ReadStorage<'a, Name>,
        Write<'a, GameLog>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, mut combat_stats, mut suffer_damage, player, names, mut gamelog) = data;

        // Process damage
        for (entity, mut stats, damage) in (&entities, &mut combat_stats, &suffer_damage).join() {
            stats.hp -= damage.amount;
            
            // Log damage for player
            if player.contains(entity) {
                gamelog.add_entry(format!("You take {} damage!", damage.amount));
            }
        }

        // Remove the damage component
        suffer_damage.clear();
    }
}