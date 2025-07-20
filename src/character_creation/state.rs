use specs::{World, WorldExt, Entity, Builder};
use crate::components::*;
use crate::resources::GameLog;
use crossterm::style::Color;

pub struct CharacterCreationState {
    pub player_name: String,
    pub selected_class: ClassType,
    pub selected_background: BackgroundType,
    pub attributes: Attributes,
    pub selected_attribute: AttributeType,
    pub selected_equipment: usize,
    pub available_equipment: Vec<(String, EquipmentSlot)>,
    pub selected_equipment_indices: Vec<usize>,
}

impl CharacterCreationState {
    pub fn new() -> Self {
        CharacterCreationState {
            player_name: String::new(),
            selected_class: ClassType::Fighter,
            selected_background: BackgroundType::Soldier,
            attributes: Attributes::new(),
            selected_attribute: AttributeType::Strength,
            selected_equipment: 0,
            available_equipment: vec![
                ("Sword".to_string(), EquipmentSlot::Melee),
                ("Bow".to_string(), EquipmentSlot::Ranged),
                ("Shield".to_string(), EquipmentSlot::Shield),
                ("Leather Armor".to_string(), EquipmentSlot::Armor),
                ("Helmet".to_string(), EquipmentSlot::Helmet),
                ("Boots".to_string(), EquipmentSlot::Boots),
                ("Gloves".to_string(), EquipmentSlot::Gloves),
                ("Ring of Protection".to_string(), EquipmentSlot::Ring),
                ("Amulet of Health".to_string(), EquipmentSlot::Amulet),
            ],
            selected_equipment_indices: Vec::new(),
        }
    }
    
    pub fn apply_class_bonuses(&mut self) {
        // Apply +2 to primary attribute and +1 to secondary attribute
        match self.selected_class.primary_attribute() {
            AttributeType::Strength => self.attributes.strength += 2,
            AttributeType::Dexterity => self.attributes.dexterity += 2,
            AttributeType::Constitution => self.attributes.constitution += 2,
            AttributeType::Intelligence => self.attributes.intelligence += 2,
            AttributeType::Wisdom => self.attributes.wisdom += 2,
            AttributeType::Charisma => self.attributes.charisma += 2,
        }
        
        match self.selected_class.secondary_attribute() {
            AttributeType::Strength => self.attributes.strength += 1,
            AttributeType::Dexterity => self.attributes.dexterity += 1,
            AttributeType::Constitution => self.attributes.constitution += 1,
            AttributeType::Intelligence => self.attributes.intelligence += 1,
            AttributeType::Wisdom => self.attributes.wisdom += 1,
            AttributeType::Charisma => self.attributes.charisma += 1,
        }
    }
    
    pub fn apply_background_bonuses(&mut self) {
        // Apply +1 to background attribute
        match self.selected_background.attribute_bonus() {
            AttributeType::Strength => self.attributes.strength += 1,
            AttributeType::Dexterity => self.attributes.dexterity += 1,
            AttributeType::Constitution => self.attributes.constitution += 1,
            AttributeType::Intelligence => self.attributes.intelligence += 1,
            AttributeType::Wisdom => self.attributes.wisdom += 1,
            AttributeType::Charisma => self.attributes.charisma += 1,
        }
    }
    
    pub fn create_player(&self, world: &mut World, x: i32, y: i32) -> Entity {
        // Calculate HP based on class and constitution modifier
        let con_modifier = self.attributes.get_modifier(AttributeType::Constitution);
        let base_hp = self.selected_class.starting_hp();
        let max_hp = base_hp + con_modifier;
        
        // Create the player entity with all components
        let player = world.create_entity()
            .with(Position { x, y })
            .with(Renderable {
                glyph: '@',
                fg: Color::White,
                bg: Color::Black,
                render_order: 0,
            })
            .with(Player {})
            .with(Viewshed {
                visible_tiles: Vec::new(),
                range: 8,
                dirty: true,
            })
            .with(Name {
                name: self.player_name.clone(),
            })
            .with(CombatStats {
                max_hp: max_hp.max(1), // Ensure HP is at least 1
                hp: max_hp.max(1),
                defense: 2 + self.attributes.get_modifier(AttributeType::Dexterity),
                power: 5 + self.attributes.get_modifier(AttributeType::Strength),
            })
            .with(PlayerInput::new())
            .with(Inventory::new(26))
            .with(Experience::new())
            .with(self.attributes.clone())
            .with(CharacterClass { class_type: self.selected_class })
            .with(Background { background_type: self.selected_background })
            .with(Skills::new())
            .with(Abilities::new())
            .build();
        
        // Add selected equipment to inventory
        for &equipment_idx in &self.selected_equipment_indices {
            let (name, slot) = &self.available_equipment[equipment_idx];
            self.create_equipment(world, player, name, *slot);
        }
        
        // Add a welcome message
        let mut log = world.write_resource::<GameLog>();
        log.add_entry(format!("Welcome, {}! Your adventure begins...", self.player_name));
        
        player
    }
    
    fn create_equipment(&self, world: &mut World, owner: Entity, name: &str, slot: EquipmentSlot) -> Entity {
        // Create equipment with appropriate bonuses based on type
        let (power_bonus, defense_bonus) = match slot {
            EquipmentSlot::Melee => (2, 0),
            EquipmentSlot::Ranged => (1, 0),
            EquipmentSlot::Shield => (0, 2),
            EquipmentSlot::Armor => (0, 3),
            EquipmentSlot::Helmet => (0, 1),
            EquipmentSlot::Boots => (0, 1),
            EquipmentSlot::Gloves => (1, 0),
            EquipmentSlot::Ring => (0, 1),
            EquipmentSlot::Amulet => (1, 1),
        };
        
        let glyph = match slot {
            EquipmentSlot::Melee => '/',
            EquipmentSlot::Ranged => '}',
            EquipmentSlot::Shield => '(',
            EquipmentSlot::Armor => '[',
            EquipmentSlot::Helmet => '^',
            EquipmentSlot::Boots => 'b',
            EquipmentSlot::Gloves => 'g',
            EquipmentSlot::Ring => '=',
            EquipmentSlot::Amulet => '"',
        };
        
        let mut item_builder = world.create_entity()
            .with(Item {})
            .with(Name { name: name.to_string() })
            .with(Renderable {
                glyph,
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
        
        // Add to inventory and equip it
        let item = item_builder.build();
        
        {
            let mut inventory = world.write_storage::<Inventory>();
            if let Some(inv) = inventory.get_mut(owner) {
                inv.items.push(item);
            }
        }
        
        {
            let mut equipped = world.write_storage::<Equipped>();
            equipped.insert(item, Equipped { owner, slot }).expect("Failed to equip item");
        }
        
        item
    }
}