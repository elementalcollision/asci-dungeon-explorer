use specs::{System, WriteStorage, ReadStorage, Entities, Entity, Join, Write};
use crate::components::{Experience, CombatStats, Player, Monster, Name};
use crate::resources::GameLog;

pub struct ExperienceGainSystem {}

impl<'a> System<'a> for ExperienceGainSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, Experience>,
        ReadStorage<'a, CombatStats>,
        ReadStorage<'a, Player>,
        ReadStorage<'a, Monster>,
        ReadStorage<'a, Name>,
        Write<'a, GameLog>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, mut experience, combat_stats, players, monsters, names, mut gamelog) = data;

        // Find dead monsters and award experience to players
        let mut dead_monsters = Vec::new();
        
        for (entity, stats, _monster, name) in (&entities, &combat_stats, &monsters, &names).join() {
            if stats.hp <= 0 {
                dead_monsters.push((entity, name.name.clone(), stats.max_hp));
            }
        }
        
        // Award experience to all players for each dead monster
        for (dead_entity, monster_name, monster_max_hp) in dead_monsters {
            // Calculate experience based on monster's max HP and level
            let base_exp = monster_max_hp * 2; // 2 XP per HP point
            
            // Award experience to all players
            for (player_entity, mut exp, _player) in (&entities, &mut experience, &players).join() {
                // Scale experience based on level difference (simple version)
                let scaled_exp = if exp.level > 1 {
                    std::cmp::max(1, base_exp - (exp.level - 1) * 2)
                } else {
                    base_exp
                };
                
                let gained = exp.gain_exp(scaled_exp);
                
                if gained {
                    gamelog.add_entry(format!("You gained {} experience from defeating {}! Level up!", scaled_exp, monster_name));
                } else {
                    gamelog.add_entry(format!("You gained {} experience from defeating {}.", scaled_exp, monster_name));
                }
            }
        }
    }
}