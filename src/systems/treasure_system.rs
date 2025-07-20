use specs::{System, WriteStorage, ReadStorage, Entities, Entity, Join, Write, Builder};
use crate::components::{
    Treasure, Position, Name, Player, WantsToInteract, Item, Renderable,
    ProvidesHealing, MeleePowerBonus, DefenseBonus, Equippable, LootDrop
};
use crate::resources::{GameLog, RandomNumberGenerator};
use crossterm::style::Color;

pub struct TreasureSystem {}

impl<'a> System<'a> for TreasureSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, WantsToInteract>,
        WriteStorage<'a, Treasure>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, Player>,
        Write<'a, GameLog>,
        Write<'a, RandomNumberGenerator>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, mut wants_interact, mut treasures, positions, names, players, mut gamelog, mut rng) = data;

        // Process treasure interaction requests
        let mut interactions = Vec::new();
        for (entity, interact) in (&entities, &wants_interact).join() {
            interactions.push((entity, interact.target));
        }
        
        // Clear interaction requests
        wants_interact.clear();
        
        // Process each interaction
        for (interactor, target) in interactions {
            if let Some(mut treasure) = treasures.get_mut(target) {
                if !treasure.is_opened {
                    self.open_treasure(
                        interactor,
                        target,
                        &mut treasure,
                        &positions,
                        &names,
                        &players,
                        &entities,
                        &mut gamelog,
                        &mut rng
                    );
                } else {
                    let interactor_name = names.get(interactor).map_or("Someone", |n| &n.name);
                    let treasure_name = names.get(target).map_or("treasure", |n| &n.name);
                    gamelog.add_entry(format!("{} examines the empty {}.", interactor_name, treasure_name));
                }
            }
        }
    }
}

impl TreasureSystem {
    fn open_treasure(
        &self,
        interactor: Entity,
        treasure_entity: Entity,
        treasure: &mut Treasure,
        positions: &ReadStorage<Position>,
        names: &ReadStorage<Name>,
        players: &ReadStorage<Player>,
        entities: &Entities,
        gamelog: &mut GameLog,
        rng: &mut RandomNumberGenerator,
    ) {
        let interactor_name = names.get(interactor).map_or("Someone", |n| &n.name);
        let treasure_name = names.get(treasure_entity).map_or("treasure", |n| &n.name);
        
        // Check if treasure requires a key
        if treasure.requires_key {
            // In a full implementation, this would check for keys in inventory
            if rng.roll_dice(1, 100) <= 20 { // 20% chance to have key for demo
                gamelog.add_entry(format!("{} uses a key to unlock the {}!", interactor_name, treasure_name));
            } else {
                gamelog.add_entry(format!("The {} is locked and {} doesn't have the key.", treasure_name, interactor_name));
                return;
            }
        }
        
        // Open the treasure
        treasure.is_opened = true;
        gamelog.add_entry(format!("{} opens the {}!", interactor_name, treasure_name));
        
        // Generate loot from the treasure's loot table
        let treasure_pos = positions.get(treasure_entity).cloned();
        if let Some(pos) = treasure_pos {
            let mut items_generated = 0;
            
            for entry in &treasure.loot_table.entries {
                let roll = rng.roll_dice(1, 100);
                if roll <= entry.chance {
                    self.create_treasure_loot(&entry.loot_drop, pos, entities, gamelog);
                    items_generated += 1;
                }
            }
            
            if items_generated == 0 {
                gamelog.add_entry(format!("The {} is empty.", treasure_name));
            } else {
                gamelog.add_entry(format!("The {} contains {} item(s)!", treasure_name, items_generated));
            }
        }
    }
    
    fn create_treasure_loot(
        &self,
        loot_drop: &LootDrop,
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
                        glyph: self.get_equipment_glyph(slot),
                        fg: Color::Cyan,
                        bg: Color::Black,
                        render_order: 2,
                    })
                    .with(Equippable { slot: *slot });
                
                if *power_bonus > 0 {
                    item_builder = item_builder.with(MeleePowerBonus { power: *power_bonus });
                }
                
                if *defense_bonus > 0 {
                    item_builder = item_builder.with(DefenseBonus { defense: *defense_bonus });
                }
                
                item_builder.build();
                gamelog.add_entry(format!("Found: {}!", name));
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
                    .with(ProvidesHealing { heal_amount: *healing })
                    .build();
                
                gamelog.add_entry(format!("Found: {}!", name));
            },
            
            LootDrop::Currency { amount } => {
                gamelog.add_entry(format!("Found: {} gold coins!", amount));
            },
        }
    }
    
    fn get_equipment_glyph(&self, slot: &crate::components::EquipmentSlot) -> char {
        match slot {
            crate::components::EquipmentSlot::Melee => '/',
            crate::components::EquipmentSlot::Shield => '(',
            crate::components::EquipmentSlot::Armor => '[',
            crate::components::EquipmentSlot::Helmet => '^',
            crate::components::EquipmentSlot::Boots => 'b',
            crate::components::EquipmentSlot::Gloves => 'g',
            crate::components::EquipmentSlot::Ring => '=',
            crate::components::EquipmentSlot::Amulet => '"',
            crate::components::EquipmentSlot::Ranged => '}',
        }
    }
}

// Component for interaction intent
#[derive(Component, Debug, Clone)]
#[storage(VecStorage)]
pub struct WantsToInteract {
    pub target: Entity,
}

// System for generating treasure containers
pub struct TreasureGenerationSystem {}

impl TreasureGenerationSystem {
    pub fn create_treasure_chest(
        entities: &Entities,
        position: Position,
        treasure_type: crate::components::TreasureType,
        loot_table: crate::components::LootTable,
        requires_key: bool,
    ) -> Entity {
        entities.create()
            .with(Name { 
                name: match treasure_type {
                    crate::components::TreasureType::Chest => "Treasure Chest".to_string(),
                    crate::components::TreasureType::Barrel => "Barrel".to_string(),
                    crate::components::TreasureType::Urn => "Ancient Urn".to_string(),
                    crate::components::TreasureType::Corpse => "Corpse".to_string(),
                    crate::components::TreasureType::SecretCache => "Secret Cache".to_string(),
                }
            })
            .with(position)
            .with(Renderable {
                glyph: treasure_type.get_glyph(),
                fg: treasure_type.get_color(),
                bg: Color::Black,
                render_order: 1,
            })
            .with(Treasure {
                treasure_type,
                loot_table,
                is_opened: false,
                requires_key,
            })
            .build()
    }
    
    pub fn create_standard_loot_table(level: i32, rng: &mut RandomNumberGenerator) -> crate::components::LootTable {
        let mut loot_table = crate::components::LootTable::new();
        
        // Health potions (common)
        loot_table.add_entry(
            LootDrop::Consumable {
                name: "Health Potion".to_string(),
                healing: 15 + (level * 2),
            },
            60 // 60% chance
        );
        
        // Equipment (uncommon)
        loot_table.add_entry(
            LootDrop::Equipment {
                name: format!("Magic Sword +{}", level + 2),
                slot: crate::components::EquipmentSlot::Melee,
                power_bonus: level + 2,
                defense_bonus: 0,
            },
            25 // 25% chance
        );
        
        // Armor (uncommon)
        loot_table.add_entry(
            LootDrop::Equipment {
                name: format!("Magic Armor +{}", level + 1),
                slot: crate::components::EquipmentSlot::Armor,
                power_bonus: 0,
                defense_bonus: level + 1,
            },
            25 // 25% chance
        );
        
        // Currency (common)
        loot_table.add_entry(
            LootDrop::Currency {
                amount: 20 + rng.roll_dice(1, level * 10),
            },
            70 // 70% chance
        );
        
        loot_table
    }
}