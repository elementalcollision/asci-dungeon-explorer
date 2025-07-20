use serde::{Serialize, Deserialize};
use specs::{World, WorldExt, Entity, Join};
use std::collections::HashMap;
use crate::components::{Position, Renderable, Name, Item};
use crate::items::item_components::*;

// Serializable representation of an item
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SerializableItem {
    pub name: String,
    pub properties: ItemProperties,
    pub position: Position,
    pub renderable: Renderable,
    pub stack: Option<ItemStack>,
    pub identification: Option<ItemIdentification>,
    pub magical_properties: Option<MagicalItem>,
    pub bonuses: Option<ItemBonuses>,
}

impl SerializableItem {
    pub fn from_entity(world: &World, entity: Entity) -> Option<Self> {
        let names = world.read_storage::<Name>();
        let properties = world.read_storage::<ItemProperties>();
        let positions = world.read_storage::<Position>();
        let renderables = world.read_storage::<Renderable>();
        let stacks = world.read_storage::<ItemStack>();
        let identifications = world.read_storage::<ItemIdentification>();
        let magical_items = world.read_storage::<MagicalItem>();
        let bonuses = world.read_storage::<ItemBonuses>();

        // Check if entity has required components
        let name = names.get(entity)?;
        let props = properties.get(entity)?;
        let position = positions.get(entity)?;
        let renderable = renderables.get(entity)?;

        Some(SerializableItem {
            name: name.name.clone(),
            properties: props.clone(),
            position: position.clone(),
            renderable: renderable.clone(),
            stack: stacks.get(entity).cloned(),
            identification: identifications.get(entity).cloned(),
            magical_properties: magical_items.get(entity).cloned(),
            bonuses: bonuses.get(entity).cloned(),
        })
    }

    pub fn to_entity(&self, world: &mut World) -> Entity {
        let mut entity_builder = world.create_entity()
            .with(Item)
            .with(Name { name: self.name.clone() })
            .with(self.properties.clone())
            .with(self.position.clone())
            .with(self.renderable.clone());

        if let Some(stack) = &self.stack {
            entity_builder = entity_builder.with(stack.clone());
        }

        if let Some(identification) = &self.identification {
            entity_builder = entity_builder.with(identification.clone());
        }

        if let Some(magical) = &self.magical_properties {
            entity_builder = entity_builder.with(magical.clone());
        }

        if let Some(bonuses) = &self.bonuses {
            entity_builder = entity_builder.with(bonuses.clone());
        }

        entity_builder.build()
    }
}

// Item database for managing item templates
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ItemDatabase {
    pub items: HashMap<String, ItemTemplate>,
}

impl ItemDatabase {
    pub fn new() -> Self {
        ItemDatabase {
            items: HashMap::new(),
        }
    }

    pub fn add_item_template(&mut self, id: String, template: ItemTemplate) {
        self.items.insert(id, template);
    }

    pub fn get_item_template(&self, id: &str) -> Option<&ItemTemplate> {
        self.items.get(id)
    }

    pub fn create_default_database() -> Self {
        let mut db = ItemDatabase::new();

        // Add weapon templates
        db.add_item_template("iron_sword".to_string(), ItemTemplate {
            name: "Iron Sword".to_string(),
            description: "A well-crafted iron sword.".to_string(),
            item_type: ItemType::Weapon(WeaponType::Sword),
            rarity: ItemRarity::Common,
            base_value: 50,
            weight: 3.0,
            glyph: '/',
            color: (128, 128, 128), // Gray
            durability: Some(100),
            stack_size: 1,
            tags: vec![ItemTag::TwoHanded],
            bonuses: Some(ItemBonuses {
                attribute_bonuses: HashMap::new(),
                skill_bonuses: HashMap::new(),
                combat_bonuses: CombatBonuses {
                    attack_bonus: 5,
                    damage_bonus: 8,
                    ..Default::default()
                },
                special_bonuses: Vec::new(),
            }),
            enchantments: Vec::new(),
        });

        db.add_item_template("health_potion".to_string(), ItemTemplate {
            name: "Health Potion".to_string(),
            description: "A red potion that restores health.".to_string(),
            item_type: ItemType::Consumable(ConsumableType::Potion),
            rarity: ItemRarity::Common,
            base_value: 25,
            weight: 0.5,
            glyph: '!',
            color: (255, 0, 0), // Red
            durability: None,
            stack_size: 10,
            tags: vec![ItemTag::Healing],
            bonuses: None,
            enchantments: Vec::new(),
        });

        db.add_item_template("leather_armor".to_string(), ItemTemplate {
            name: "Leather Armor".to_string(),
            description: "Basic leather armor for protection.".to_string(),
            item_type: ItemType::Armor(ArmorType::Chest),
            rarity: ItemRarity::Common,
            base_value: 40,
            weight: 8.0,
            glyph: '[',
            color: (139, 69, 19), // Brown
            durability: Some(80),
            stack_size: 1,
            tags: vec![ItemTag::Light],
            bonuses: Some(ItemBonuses {
                attribute_bonuses: HashMap::new(),
                skill_bonuses: HashMap::new(),
                combat_bonuses: CombatBonuses {
                    defense_bonus: 5,
                    ..Default::default()
                },
                special_bonuses: Vec::new(),
            }),
            enchantments: Vec::new(),
        });

        // Add more templates as needed...
        db
    }

    pub fn save_to_file(&self, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(filename, json)?;
        Ok(())
    }

    pub fn load_from_file(filename: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let json = std::fs::read_to_string(filename)?;
        let db = serde_json::from_str(&json)?;
        Ok(db)
    }
}

// Item template for creating items from data
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ItemTemplate {
    pub name: String,
    pub description: String,
    pub item_type: ItemType,
    pub rarity: ItemRarity,
    pub base_value: i32,
    pub weight: f32,
    pub glyph: char,
    pub color: (u8, u8, u8),
    pub durability: Option<i32>,
    pub stack_size: i32,
    pub tags: Vec<ItemTag>,
    pub bonuses: Option<ItemBonuses>,
    pub enchantments: Vec<EnchantmentTemplate>,
}

impl ItemTemplate {
    pub fn create_item(&self, world: &mut World, position: Position) -> Entity {
        let properties = ItemProperties::new(self.name.clone(), self.item_type.clone())
            .with_description(self.description.clone())
            .with_rarity(self.rarity.clone())
            .with_value(self.base_value)
            .with_weight(self.weight)
            .with_stack_size(self.stack_size);

        let properties = if let Some(durability) = self.durability {
            properties.with_durability(durability)
        } else {
            properties
        };

        let mut entity_builder = world.create_entity()
            .with(Item)
            .with(Name { name: self.name.clone() })
            .with(properties)
            .with(position)
            .with(Renderable {
                glyph: self.glyph,
                fg: crossterm::style::Color::Rgb {
                    r: self.color.0,
                    g: self.color.1,
                    b: self.color.2,
                },
                bg: crossterm::style::Color::Black,
                render_order: 2,
            });

        if self.stack_size > 1 {
            entity_builder = entity_builder.with(ItemStack::new(1, self.stack_size));
        }

        if let Some(bonuses) = &self.bonuses {
            entity_builder = entity_builder.with(bonuses.clone());
        }

        if !self.enchantments.is_empty() {
            let mut magical_item = MagicalItem::new(1);
            for enchantment_template in &self.enchantments {
                let enchantment = Enchantment {
                    name: enchantment_template.name.clone(),
                    description: enchantment_template.description.clone(),
                    enchantment_type: enchantment_template.enchantment_type.clone(),
                    power: enchantment_template.power,
                    duration: enchantment_template.duration,
                };
                magical_item.add_enchantment(enchantment);
            }
            entity_builder = entity_builder.with(magical_item);
        }

        entity_builder.build()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EnchantmentTemplate {
    pub name: String,
    pub description: String,
    pub enchantment_type: EnchantmentType,
    pub power: i32,
    pub duration: Option<i32>,
}

// Item collection serialization for save games
#[derive(Serialize, Deserialize, Debug)]
pub struct ItemCollection {
    pub items: Vec<SerializableItem>,
}

impl ItemCollection {
    pub fn from_world(world: &World) -> Self {
        let entities = world.entities();
        let items = world.read_storage::<Item>();
        
        let mut serializable_items = Vec::new();
        
        for (entity, _item) in (&entities, &items).join() {
            if let Some(serializable_item) = SerializableItem::from_entity(world, entity) {
                serializable_items.push(serializable_item);
            }
        }
        
        ItemCollection {
            items: serializable_items,
        }
    }

    pub fn to_world(&self, world: &mut World) -> Vec<Entity> {
        let mut entities = Vec::new();
        
        for item in &self.items {
            let entity = item.to_entity(world);
            entities.push(entity);
        }
        
        entities
    }

    pub fn save_to_file(&self, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(filename, json)?;
        Ok(())
    }

    pub fn load_from_file(filename: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let json = std::fs::read_to_string(filename)?;
        let collection = serde_json::from_str(&json)?;
        Ok(collection)
    }
}

// Utility functions for item serialization
pub fn serialize_items_in_area(
    world: &World,
    min_x: i32,
    min_y: i32,
    max_x: i32,
    max_y: i32,
) -> ItemCollection {
    let entities = world.entities();
    let items = world.read_storage::<Item>();
    let positions = world.read_storage::<Position>();
    
    let mut serializable_items = Vec::new();
    
    for (entity, _item, position) in (&entities, &items, &positions).join() {
        if position.x >= min_x && position.x <= max_x && 
           position.y >= min_y && position.y <= max_y {
            if let Some(serializable_item) = SerializableItem::from_entity(world, entity) {
                serializable_items.push(serializable_item);
            }
        }
    }
    
    ItemCollection {
        items: serializable_items,
    }
}

pub fn count_items_by_type(world: &World) -> HashMap<String, i32> {
    let entities = world.entities();
    let items = world.read_storage::<Item>();
    let properties = world.read_storage::<ItemProperties>();
    
    let mut counts = HashMap::new();
    
    for (_entity, _item, props) in (&entities, &items, &properties).join() {
        let type_name = format!("{:?}", props.item_type);
        *counts.entry(type_name).or_insert(0) += 1;
    }
    
    counts
}

#[cfg(test)]
mod tests {
    use super::*;
    use specs::{World, WorldExt};

    #[test]
    fn test_item_serialization() {
        let mut world = World::new();
        world.register::<Item>();
        world.register::<Name>();
        world.register::<ItemProperties>();
        world.register::<Position>();
        world.register::<Renderable>();

        // Create a test item
        let entity = world.create_entity()
            .with(Item)
            .with(Name { name: "Test Sword".to_string() })
            .with(ItemProperties::new("Test Sword".to_string(), ItemType::Weapon(WeaponType::Sword)))
            .with(Position { x: 5, y: 5 })
            .with(Renderable {
                glyph: '/',
                fg: crossterm::style::Color::Grey,
                bg: crossterm::style::Color::Black,
                render_order: 2,
            })
            .build();

        // Serialize the item
        let serializable = SerializableItem::from_entity(&world, entity);
        assert!(serializable.is_some());

        let serializable = serializable.unwrap();
        assert_eq!(serializable.name, "Test Sword");
        assert_eq!(serializable.position.x, 5);
        assert_eq!(serializable.position.y, 5);
    }

    #[test]
    fn test_item_database() {
        let db = ItemDatabase::create_default_database();
        
        assert!(db.get_item_template("iron_sword").is_some());
        assert!(db.get_item_template("health_potion").is_some());
        assert!(db.get_item_template("nonexistent_item").is_none());
    }

    #[test]
    fn test_item_template_creation() {
        let mut world = World::new();
        world.register::<Item>();
        world.register::<Name>();
        world.register::<ItemProperties>();
        world.register::<Position>();
        world.register::<Renderable>();
        world.register::<ItemBonuses>();

        let db = ItemDatabase::create_default_database();
        let template = db.get_item_template("iron_sword").unwrap();
        
        let entity = template.create_item(&mut world, Position { x: 0, y: 0 });
        
        let names = world.read_storage::<Name>();
        let name = names.get(entity).unwrap();
        assert_eq!(name.name, "Iron Sword");
    }
}