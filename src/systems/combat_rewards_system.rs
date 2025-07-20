use specs::{System, WriteStorage, ReadStorage, Entities, Entity, Join, Write, Builder};
use crate::components::{
    CombatStats, Experience, Name, Player, Monster, Position, Item, Renderable,
    ProvidesHealing, MeleePowerBonus, DefenseBonus, Equippable, EquipmentSlot,
    LootTable, LootDrop, UniqueEnemy, CombatReward
};
use crate::resources::{GameLog, RandomNumberGenerator};
use crossterm::style::Color;

pub struct CombatRewardsSystem {}

impl<'a> System<'a> for CombatRewardsSystem {
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, CombatStats>,
        WriteStorage<'a, Experience>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, Player>,
        ReadStorage<'a, Monster>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, LootTable>,
        ReadStorage<'a, UniqueEnemy>,
        WriteStorage<'a, CombatReward>,
        Write<'a, GameLog>,
        Write<'a, RandomNumberGenerator>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            entities, 
            combat_stats, 
            mut experience, 
            names, 
            players, 
            monsters,
            positions,
            loot_tables,
            unique_enemies,
            mut combat_rewards,
            mut gamelog, 
            mut rng
        ) = data;

        // Find dead monsters and process rewards
        let mut dead_monsters = Vec::new();
        
        for (entity, stats, _monster, name) in (&entities, &combat_stats, &monsters, &names).join() {
            if stats.hp <= 0 {
                let monster_pos = positions.get(entity).cloned();
                let loot_table = loot_tables.get(entity).cloned();
                let is_unique = unique_enemies.contains(entity);
                
                dead_monsters.push((entity, name.name.clone(), stats.clone(), monster_pos, loot_table, is_unique));
            }
        }
        
        // Process rewards for each dead monster
        for (dead_entity, monster_name, monster_stats, monster_pos, loot_table, is_unique) in dead_monsters {
            // Calculate and distribute experience
            self.distribute_experience(
                dead_entity,
                &monster_name,
                &monster_stats,
                is_unique,
                &entities,
                &mut experience,
                &players,
                &positions,
                &mut gamelog,
                &mut rng
            );
            
            // Generate and drop loot
            if let Some(pos) = monster_pos {
                self.generate_loot(
                    dead_entity,
                    &monster_name,
                    &monster_stats,
                    pos,
                    loot_table,
                    is_unique,
                    &entities,
                    &mut gamelog,
                    &mut rng
                );
            }
            
            // Create combat reward summary
            let reward = CombatReward {
                source_entity: dead_entity,
                source_name: monster_name.clone(),
                experience_gained: self.calculate_base_experience(&monster_stats, is_unique),
                loot_generated: true,
                special_drops: is_unique,
            };
            
            combat_rewards.insert(dead_entity, reward)
                .expect("Failed to insert combat reward");
        }
    }
}

impl CombatRewardsSystem {
    fn distribute_experience(
        &self,
        dead_entity: Entity,
        monster_name: &str,
        monster_stats: &CombatStats,
        is_unique: bool,
        entities: &Entities,
        experience: &mut WriteStorage<Experience>,
        players: &ReadStorage<Player>,
        positions: &ReadStorage<Position>,
        gamelog: &mut GameLog,
        rng: &mut RandomNumberGenerator,
    ) {
        let base_exp = self.calculate_base_experience(monster_stats, is_unique);
        let dead_pos = positions.get(dead_entity);
        
        // Find all players who should receive experience
        let mut eligible_players = Vec::new();
        
        for (entity, _player, exp_comp, pos) in (entities, players, experience, positions).join() {
            // Check if player is within experience range (simplified - could be more complex)
            let in_range = if let Some(dead_position) = dead_pos {
                let distance = ((pos.x - dead_position.x).pow(2) + (pos.y - dead_position.y).pow(2)) as f32;
                distance.sqrt() <= 10.0 // Within 10 tiles
            } else {
                true // If no position, give experience anyway
            };
            
            if in_range {
                eligible_players.push((entity, exp_comp.level));
            }
        }
        
        if eligible_players.is_empty() {
            return;
        }
        
        // Distribute experience among eligible players
        let exp_per_player = if eligible_players.len() == 1 {
            base_exp
        } else {
            // Slightly reduced experience when shared, but not too punishing
            (base_exp as f32 * 0.8 / eligible_players.len() as f32) as i32
        };
        
        for (player_entity, player_level) in eligible_players {
            if let Some(exp_comp) = experience.get_mut(player_entity) {
                // Scale experience based on level difference
                let level_diff = player_level - self.estimate_monster_level(monster_stats);
                let scaled_exp = self.scale_experience_by_level(exp_per_player, level_diff);
                
                let gained_exp = exp_comp.gain_exp(scaled_exp);
                
                if gained_exp {
                    gamelog.add_entry(format!("You gained {} experience from {} and leveled up!", 
                        scaled_exp, monster_name));
                } else {
                    gamelog.add_entry(format!("You gained {} experience from {}.", 
                        scaled_exp, monster_name));
                }
            }
        }
    }
    
    fn generate_loot(
        &self,
        dead_entity: Entity,
        monster_name: &str,
        monster_stats: &CombatStats,
        position: Position,
        loot_table: Option<LootTable>,
        is_unique: bool,
        entities: &Entities,
        gamelog: &mut GameLog,
        rng: &mut RandomNumberGenerator,
    ) {
        let mut items_dropped = Vec::new();
        
        // Use custom loot table if available, otherwise generate based on monster stats
        if let Some(table) = loot_table {
            items_dropped.extend(self.roll_loot_table(&table, rng));
        } else {
            items_dropped.extend(self.generate_default_loot(monster_stats, is_unique, rng));
        }
        
        // Generate special drops for unique enemies
        if is_unique {
            items_dropped.extend(self.generate_unique_loot(monster_name, monster_stats, rng));
        }
        
        // Create item entities and place them at the monster's position
        for loot_drop in items_dropped {
            self.create_loot_item(loot_drop, position, entities, gamelog);
        }
        
        if !items_dropped.is_empty() {
            gamelog.add_entry(format!("{} drops {} item(s)!", monster_name, items_dropped.len()));
        }
    }
    
    fn calculate_base_experience(&self, monster_stats: &CombatStats, is_unique: bool) -> i32 {
        // Base experience calculation
        let base_exp = monster_stats.max_hp + (monster_stats.power * 2) + (monster_stats.defense * 2);
        
        // Bonus for unique enemies
        if is_unique {
            (base_exp as f32 * 2.5) as i32
        } else {
            base_exp
        }
    }
    
    fn estimate_monster_level(&self, monster_stats: &CombatStats) -> i32 {
        // Rough estimation of monster level based on stats
        ((monster_stats.max_hp + monster_stats.power + monster_stats.defense) / 10).max(1)
    }
    
    fn scale_experience_by_level(&self, base_exp: i32, level_diff: i32) -> i32 {
        // Reduce experience for fighting lower level monsters
        // Increase slightly for fighting higher level monsters
        let multiplier = match level_diff {
            diff if diff >= 3 => 0.1,  // Much higher level player
            diff if diff >= 1 => 0.5,  // Higher level player
            0 => 1.0,                  // Same level
            diff if diff >= -2 => 1.2, // Slightly higher level monster
            _ => 1.5,                  // Much higher level monster
        };
        
        (base_exp as f32 * multiplier) as i32
    }
    
    fn roll_loot_table(&self, loot_table: &LootTable, rng: &mut RandomNumberGenerator) -> Vec<LootDrop> {
        let mut drops = Vec::new();
        
        for entry in &loot_table.entries {
            let roll = rng.roll_dice(1, 100);
            if roll <= entry.chance {
                drops.push(entry.loot_drop.clone());
            }
        }
        
        drops
    }
    
    fn generate_default_loot(&self, monster_stats: &CombatStats, is_unique: bool, rng: &mut RandomNumberGenerator) -> Vec<LootDrop> {
        let mut drops = Vec::new();
        
        // Base loot chance based on monster power
        let loot_chance = 30 + (monster_stats.power * 5); // 30-80% base chance
        
        if rng.roll_dice(1, 100) <= loot_chance {
            // Generate appropriate loot based on monster level
            let monster_level = self.estimate_monster_level(monster_stats);
            
            // Healing items (common)
            if rng.roll_dice(1, 100) <= 40 {
                drops.push(LootDrop::Consumable {
                    name: "Health Potion".to_string(),
                    healing: 10 + (monster_level * 2),
                });
            }
            
            // Equipment (less common)
            if rng.roll_dice(1, 100) <= 25 {
                drops.push(self.generate_equipment_drop(monster_level, rng));
            }
            
            // Gold/currency (if implemented)
            if rng.roll_dice(1, 100) <= 60 {
                let gold_amount = 5 + rng.roll_dice(1, monster_level * 3);
                drops.push(LootDrop::Currency { amount: gold_amount });
            }
        }
        
        drops
    }
    
    fn generate_unique_loot(&self, monster_name: &str, monster_stats: &CombatStats, rng: &mut RandomNumberGenerator) -> Vec<LootDrop> {
        let mut drops = Vec::new();
        
        // Unique enemies always drop something special
        let monster_level = self.estimate_monster_level(monster_stats);
        
        // Generate a unique item based on the monster's name/type
        let unique_item = match monster_name.to_lowercase().as_str() {
            name if name.contains("dragon") => {
                LootDrop::Equipment {
                    name: "Dragon Scale Armor".to_string(),
                    slot: EquipmentSlot::Armor,
                    power_bonus: 0,
                    defense_bonus: 8 + monster_level,
                }
            },
            name if name.contains("lich") => {
                LootDrop::Equipment {
                    name: "Lich's Staff".to_string(),
                    slot: EquipmentSlot::Melee,
                    power_bonus: 6 + monster_level,
                    defense_bonus: 0,
                }
            },
            name if name.contains("demon") => {
                LootDrop::Equipment {
                    name: "Demon Blade".to_string(),
                    slot: EquipmentSlot::Melee,
                    power_bonus: 8 + monster_level,
                    defense_bonus: 0,
                }
            },
            _ => {
                // Generic unique item
                LootDrop::Equipment {
                    name: format!("{}'s Trophy", monster_name),
                    slot: EquipmentSlot::Ring,
                    power_bonus: 2 + monster_level / 2,
                    defense_bonus: 2 + monster_level / 2,
                }
            }
        };
        
        drops.push(unique_item);
        
        // Unique enemies also drop extra healing items
        drops.push(LootDrop::Consumable {
            name: "Greater Health Potion".to_string(),
            healing: 25 + (monster_level * 3),
        });
        
        // Extra currency
        drops.push(LootDrop::Currency { 
            amount: 50 + rng.roll_dice(1, monster_level * 10) 
        });
        
        drops
    }
    
    fn generate_equipment_drop(&self, monster_level: i32, rng: &mut RandomNumberGenerator) -> LootDrop {
        let equipment_types = [
            EquipmentSlot::Melee,
            EquipmentSlot::Shield,
            EquipmentSlot::Armor,
            EquipmentSlot::Helmet,
            EquipmentSlot::Boots,
            EquipmentSlot::Gloves,
        ];
        
        let slot = equipment_types[rng.roll_dice(1, equipment_types.len()) as usize - 1];
        
        let (name, power_bonus, defense_bonus) = match slot {
            EquipmentSlot::Melee => {
                let power = 2 + monster_level + rng.roll_dice(1, 3);
                (format!("Iron Sword +{}", power), power, 0)
            },
            EquipmentSlot::Shield => {
                let defense = 1 + monster_level / 2 + rng.roll_dice(1, 2);
                (format!("Iron Shield +{}", defense), 0, defense)
            },
            EquipmentSlot::Armor => {
                let defense = 2 + monster_level + rng.roll_dice(1, 2);
                (format!("Chain Mail +{}", defense), 0, defense)
            },
            EquipmentSlot::Helmet => {
                let defense = 1 + monster_level / 3;
                (format!("Iron Helmet +{}", defense), 0, defense)
            },
            EquipmentSlot::Boots => {
                let defense = 1 + monster_level / 4;
                (format!("Leather Boots +{}", defense), 0, defense)
            },
            EquipmentSlot::Gloves => {
                let power = 1 + monster_level / 3;
                (format!("Leather Gloves +{}", power), power, 0)
            },
            _ => ("Generic Item".to_string(), 1, 1),
        };
        
        LootDrop::Equipment {
            name,
            slot,
            power_bonus,
            defense_bonus,
        }
    }
    
    fn create_loot_item(
        &self,
        loot_drop: LootDrop,
        position: Position,
        entities: &Entities,
        gamelog: &mut GameLog,
    ) {
        match loot_drop {
            LootDrop::Equipment { name, slot, power_bonus, defense_bonus } => {
                let mut item_builder = entities.create()
                    .with(Item {})
                    .with(Name { name: name.clone() })
                    .with(Position { x: position.x, y: position.y })
                    .with(Renderable {
                        glyph: self.get_equipment_glyph(&slot),
                        fg: Color::Cyan,
                        bg: Color::Black,
                        render_order: 2,
                    })
                    .with(Equippable { slot });
                
                if power_bonus > 0 {
                    item_builder = item_builder.with(MeleePowerBonus { power: power_bonus });
                }
                
                if defense_bonus > 0 {
                    item_builder = item_builder.with(DefenseBonus { defense: defense_bonus });
                }
                
                item_builder.build();
                gamelog.add_entry(format!("A {} appears!", name));
            },
            
            LootDrop::Consumable { name, healing } => {
                entities.create()
                    .with(Item {})
                    .with(Name { name: name.clone() })
                    .with(Position { x: position.x, y: position.y })
                    .with(Renderable {
                        glyph: '!',
                        fg: Color::Magenta,
                        bg: Color::Black,
                        render_order: 2,
                    })
                    .with(ProvidesHealing { heal_amount: healing })
                    .build();
                
                gamelog.add_entry(format!("A {} appears!", name));
            },
            
            LootDrop::Currency { amount } => {
                // For now, just log the currency drop
                // In a full implementation, this would create a currency item or add to player inventory
                gamelog.add_entry(format!("{} gold coins scatter on the ground!", amount));
            },
        }
    }
    
    fn get_equipment_glyph(&self, slot: &EquipmentSlot) -> char {
        match slot {
            EquipmentSlot::Melee => '/',
            EquipmentSlot::Shield => '(',
            EquipmentSlot::Armor => '[',
            EquipmentSlot::Helmet => '^',
            EquipmentSlot::Boots => 'b',
            EquipmentSlot::Gloves => 'g',
            EquipmentSlot::Ring => '=',
            EquipmentSlot::Amulet => '"',
            EquipmentSlot::Ranged => '}',
        }
    }
}