use specs::{World, WorldExt, Builder, Entity};
use crate::components::{Position, Name, Renderable, Item};
use crate::items::{
    ItemProperties, ItemType, ConsumableType, ItemRarity, ItemStack,
    consumable_system::{
        Consumable, ConsumableEffect, StatusEffectType, StatusEffect,
        ConsumableRestriction
    }
};
use crate::resources::RandomNumberGenerator;
use std::collections::HashMap;

/// Factory for creating different types of consumable items
pub struct ConsumableFactory;

impl ConsumableFactory {
    pub fn new() -> Self {
        ConsumableFactory
    }

    /// Create a health potion
    pub fn create_health_potion(
        &self,
        world: &mut World,
        position: Position,
        potency: PotionPotency,
    ) -> Entity {
        let (name, healing_amount, value, color) = match potency {
            PotionPotency::Minor => ("Minor Health Potion", 15, 25, crossterm::style::Color::DarkRed),
            PotionPotency::Lesser => ("Health Potion", 25, 50, crossterm::style::Color::Red),
            PotionPotency::Greater => ("Greater Health Potion", 50, 100, crossterm::style::Color::Magenta),
            PotionPotency::Superior => ("Superior Health Potion", 75, 200, crossterm::style::Color::Yellow),
            PotionPotency::Ultimate => ("Ultimate Health Potion", 100, 500, crossterm::style::Color::White),
        };

        let consumable = Consumable::new(ConsumableType::Potion)
            .with_effects(vec![
                ConsumableEffect::Healing {
                    amount: healing_amount,
                    over_time: false,
                }
            ])
            .with_use_time(1.0)
            .with_cooldown(2.0);

        let properties = ItemProperties::new(name.to_string(), ItemType::Consumable(ConsumableType::Potion))
            .with_description(format!("A potion that restores {} health instantly.", healing_amount))
            .with_value(value)
            .with_weight(0.5)
            .with_stack_size(10);

        world.create_entity()
            .with(Item)
            .with(Name { name: name.to_string() })
            .with(properties)
            .with(consumable)
            .with(ItemStack::new(1, 10))
            .with(position)
            .with(Renderable {
                glyph: '!',
                fg: color,
                bg: crossterm::style::Color::Black,
                render_order: 2,
            })
            .build()
    }

    /// Create a mana potion
    pub fn create_mana_potion(
        &self,
        world: &mut World,
        position: Position,
        potency: PotionPotency,
    ) -> Entity {
        let (name, mana_amount, value) = match potency {
            PotionPotency::Minor => ("Minor Mana Potion", 15, 30),
            PotionPotency::Lesser => ("Mana Potion", 25, 60),
            PotionPotency::Greater => ("Greater Mana Potion", 50, 120),
            PotionPotency::Superior => ("Superior Mana Potion", 75, 240),
            PotionPotency::Ultimate => ("Ultimate Mana Potion", 100, 600),
        };

        let consumable = Consumable::new(ConsumableType::Potion)
            .with_effects(vec![
                ConsumableEffect::ManaRestore {
                    amount: mana_amount,
                    over_time: false,
                }
            ])
            .with_use_time(1.0)
            .with_cooldown(2.0);

        let properties = ItemProperties::new(name.to_string(), ItemType::Consumable(ConsumableType::Potion))
            .with_description(format!("A potion that restores {} mana instantly.", mana_amount))
            .with_value(value)
            .with_weight(0.5)
            .with_stack_size(10);

        world.create_entity()
            .with(Item)
            .with(Name { name: name.to_string() })
            .with(properties)
            .with(consumable)
            .with(ItemStack::new(1, 10))
            .with(position)
            .with(Renderable {
                glyph: '!',
                fg: crossterm::style::Color::Blue,
                bg: crossterm::style::Color::Black,
                render_order: 2,
            })
            .build()
    }

    /// Create a regeneration potion
    pub fn create_regeneration_potion(
        &self,
        world: &mut World,
        position: Position,
        duration: f32,
        power: i32,
    ) -> Entity {
        let name = format!("Potion of Regeneration");
        let value = (duration * power as f32 * 2.0) as i32;

        let consumable = Consumable::new(ConsumableType::Potion)
            .with_effects(vec![
                ConsumableEffect::StatusEffect {
                    effect_type: StatusEffectType::Regeneration,
                    duration,
                    power,
                }
            ])
            .with_use_time(1.5)
            .with_cooldown(5.0);

        let properties = ItemProperties::new(name.clone(), ItemType::Consumable(ConsumableType::Potion))
            .with_description(format!("Grants regeneration for {:.0} seconds, healing {} per second.", duration, power))
            .with_value(value)
            .with_weight(0.6)
            .with_stack_size(5);

        world.create_entity()
            .with(Item)
            .with(Name { name })
            .with(properties)
            .with(consumable)
            .with(ItemStack::new(1, 5))
            .with(position)
            .with(Renderable {
                glyph: '!',
                fg: crossterm::style::Color::Green,
                bg: crossterm::style::Color::Black,
                render_order: 2,
            })
            .build()
    }

    /// Create a stat boost potion
    pub fn create_stat_potion(
        &self,
        world: &mut World,
        position: Position,
        attribute: String,
        boost: i32,
        duration: f32,
    ) -> Entity {
        let name = format!("Potion of {}", attribute);
        let value = (boost * duration as i32 * 5);

        let consumable = Consumable::new(ConsumableType::Potion)
            .with_effects(vec![
                ConsumableEffect::AttributeBoost {
                    attribute: attribute.clone(),
                    amount: boost,
                    duration,
                }
            ])
            .with_use_time(1.0)
            .with_cooldown(10.0); // Longer cooldown for stat boosts

        let properties = ItemProperties::new(name.clone(), ItemType::Consumable(ConsumableType::Potion))
            .with_description(format!("Increases {} by {} for {:.0} seconds.", attribute, boost, duration))
            .with_value(value)
            .with_weight(0.4)
            .with_stack_size(3);

        world.create_entity()
            .with(Item)
            .with(Name { name })
            .with(properties)
            .with(consumable)
            .with(ItemStack::new(1, 3))
            .with(position)
            .with(Renderable {
                glyph: '!',
                fg: crossterm::style::Color::Cyan,
                bg: crossterm::style::Color::Black,
                render_order: 2,
            })
            .build()
    }

    /// Create food items
    pub fn create_food(
        &self,
        world: &mut World,
        position: Position,
        food_type: FoodType,
    ) -> Entity {
        let (name, healing, duration, value, weight, stack_size, glyph) = match food_type {
            FoodType::Bread => ("Bread", 8, 30.0, 5, 0.2, 20, '%'),
            FoodType::Cheese => ("Cheese", 12, 45.0, 8, 0.3, 15, '%'),
            FoodType::Meat => ("Cooked Meat", 20, 60.0, 15, 0.5, 10, '%'),
            FoodType::Apple => ("Apple", 5, 20.0, 3, 0.1, 25, '%'),
            FoodType::Rations => ("Travel Rations", 15, 120.0, 25, 1.0, 5, '%'),
        };

        let consumable = Consumable::new(ConsumableType::Food)
            .with_effects(vec![
                ConsumableEffect::Healing {
                    amount: healing,
                    over_time: true,
                },
                ConsumableEffect::StatusEffect {
                    effect_type: StatusEffectType::WellFed,
                    duration,
                    power: 1,
                }
            ])
            .with_use_time(3.0); // Food takes longer to consume

        let properties = ItemProperties::new(name.to_string(), ItemType::Consumable(ConsumableType::Food))
            .with_description(format!("Nutritious food that heals {} health over time and provides nourishment.", healing))
            .with_value(value)
            .with_weight(weight)
            .with_stack_size(stack_size);

        world.create_entity()
            .with(Item)
            .with(Name { name: name.to_string() })
            .with(properties)
            .with(consumable)
            .with(ItemStack::new(1, stack_size))
            .with(position)
            .with(Renderable {
                glyph,
                fg: crossterm::style::Color::DarkYellow,
                bg: crossterm::style::Color::Black,
                render_order: 2,
            })
            .build()
    }

    /// Create magic scrolls
    pub fn create_scroll(
        &self,
        world: &mut World,
        position: Position,
        scroll_type: ScrollType,
    ) -> Entity {
        let (name, effects, value, charges, rarity) = match scroll_type {
            ScrollType::Healing => (
                "Scroll of Healing",
                vec![ConsumableEffect::Healing { amount: 40, over_time: false }],
                75,
                1,
                ItemRarity::Uncommon,
            ),
            ScrollType::Fireball => (
                "Scroll of Fireball",
                vec![ConsumableEffect::SpellCast { spell_id: "fireball".to_string() }],
                100,
                1,
                ItemRarity::Uncommon,
            ),
            ScrollType::Teleport => (
                "Scroll of Teleport",
                vec![ConsumableEffect::Teleport { range: 10, random: false }],
                150,
                1,
                ItemRarity::Rare,
            ),
            ScrollType::Identify => (
                "Scroll of Identify",
                vec![ConsumableEffect::Identify { count: 1 }],
                50,
                1,
                ItemRarity::Common,
            ),
            ScrollType::MagicMapping => (
                "Scroll of Magic Mapping",
                vec![ConsumableEffect::RevealMap { radius: 20 }],
                200,
                1,
                ItemRarity::Rare,
            ),
        };

        let consumable = Consumable::new(ConsumableType::Scroll)
            .with_effects(effects)
            .with_charges(charges)
            .with_use_time(2.0)
            .with_cooldown(1.0);

        let properties = ItemProperties::new(name.to_string(), ItemType::Consumable(ConsumableType::Scroll))
            .with_description("A magical scroll that can be read to cast a spell.".to_string())
            .with_rarity(rarity)
            .with_value(value)
            .with_weight(0.1)
            .with_stack_size(5);

        world.create_entity()
            .with(Item)
            .with(Name { name: name.to_string() })
            .with(properties)
            .with(consumable)
            .with(ItemStack::new(1, 5))
            .with(position)
            .with(Renderable {
                glyph: '?',
                fg: crossterm::style::Color::White,
                bg: crossterm::style::Color::Black,
                render_order: 2,
            })
            .build()
    }

    /// Create antidotes and cure potions
    pub fn create_cure_potion(
        &self,
        world: &mut World,
        position: Position,
        condition: StatusEffectType,
    ) -> Entity {
        let name = format!("Potion of Cure {:?}", condition);
        let value = match condition {
            StatusEffectType::Poison => 40,
            StatusEffectType::Disease => 60,
            StatusEffectType::Curse => 100,
            _ => 30,
        };

        let consumable = Consumable::new(ConsumableType::Potion)
            .with_effects(vec![
                ConsumableEffect::CureCondition { condition: condition.clone() }
            ])
            .with_use_time(1.0)
            .with_cooldown(1.0);

        let properties = ItemProperties::new(name.clone(), ItemType::Consumable(ConsumableType::Potion))
            .with_description(format!("Cures the {:?} condition.", condition))
            .with_value(value)
            .with_weight(0.3)
            .with_stack_size(5);

        world.create_entity()
            .with(Item)
            .with(Name { name })
            .with(properties)
            .with(consumable)
            .with(ItemStack::new(1, 5))
            .with(position)
            .with(Renderable {
                glyph: '!',
                fg: crossterm::style::Color::Yellow,
                bg: crossterm::style::Color::Black,
                render_order: 2,
            })
            .build()
    }

    /// Create special consumables with restrictions
    pub fn create_emergency_potion(
        &self,
        world: &mut World,
        position: Position,
    ) -> Entity {
        let consumable = Consumable::new(ConsumableType::Potion)
            .with_effects(vec![
                ConsumableEffect::Healing { amount: 100, over_time: false },
                ConsumableEffect::StatusEffect {
                    effect_type: StatusEffectType::Haste,
                    duration: 30.0,
                    power: 2,
                }
            ])
            .with_use_time(0.5) // Very fast to use
            .with_cooldown(300.0) // 5 minute cooldown
            .add_restriction(ConsumableRestriction::HealthThreshold(0.25)); // Only when below 25% health

        let properties = ItemProperties::new("Emergency Potion".to_string(), ItemType::Consumable(ConsumableType::Potion))
            .with_description("A powerful potion that can only be used in dire circumstances (below 25% health).".to_string())
            .with_rarity(ItemRarity::Epic)
            .with_value(500)
            .with_weight(0.3)
            .with_stack_size(1);

        world.create_entity()
            .with(Item)
            .with(Name { name: "Emergency Potion".to_string() })
            .with(properties)
            .with(consumable)
            .with(position)
            .with(Renderable {
                glyph: '!',
                fg: crossterm::style::Color::Magenta,
                bg: crossterm::style::Color::Black,
                render_order: 2,
            })
            .build()
    }

    /// Create random consumable based on context
    pub fn create_random_consumable(
        &self,
        world: &mut World,
        position: Position,
        context: ConsumableContext,
        rng: &mut RandomNumberGenerator,
    ) -> Entity {
        match context {
            ConsumableContext::Combat => {
                let roll = rng.roll_dice(1, 100);
                if roll <= 60 {
                    self.create_health_potion(world, position, PotionPotency::Lesser)
                } else if roll <= 80 {
                    self.create_mana_potion(world, position, PotionPotency::Lesser)
                } else if roll <= 90 {
                    self.create_regeneration_potion(world, position, 30.0, 2)
                } else {
                    self.create_cure_potion(world, position, StatusEffectType::Poison)
                }
            },
            ConsumableContext::Exploration => {
                let roll = rng.roll_dice(1, 100);
                if roll <= 40 {
                    self.create_food(world, position, FoodType::Rations)
                } else if roll <= 60 {
                    self.create_scroll(world, position, ScrollType::MagicMapping)
                } else if roll <= 80 {
                    self.create_scroll(world, position, ScrollType::Identify)
                } else {
                    self.create_health_potion(world, position, PotionPotency::Lesser)
                }
            },
            ConsumableContext::Treasure => {
                let roll = rng.roll_dice(1, 100);
                if roll <= 30 {
                    self.create_health_potion(world, position, PotionPotency::Greater)
                } else if roll <= 50 {
                    let attributes = vec!["Strength", "Dexterity", "Constitution", "Intelligence"];
                    let attr = &attributes[rng.roll_dice(1, attributes.len()) - 1];
                    self.create_stat_potion(world, position, attr.to_string(), 3, 300.0)
                } else if roll <= 70 {
                    self.create_scroll(world, position, ScrollType::Teleport)
                } else if roll <= 90 {
                    self.create_regeneration_potion(world, position, 60.0, 3)
                } else {
                    self.create_emergency_potion(world, position)
                }
            },
        }
    }
}

/// Potion potency levels
#[derive(Debug, Clone, Copy)]
pub enum PotionPotency {
    Minor,
    Lesser,
    Greater,
    Superior,
    Ultimate,
}

/// Food types
#[derive(Debug, Clone, Copy)]
pub enum FoodType {
    Bread,
    Cheese,
    Meat,
    Apple,
    Rations,
}

/// Scroll types
#[derive(Debug, Clone, Copy)]
pub enum ScrollType {
    Healing,
    Fireball,
    Teleport,
    Identify,
    MagicMapping,
}

/// Context for random consumable generation
#[derive(Debug, Clone, Copy)]
pub enum ConsumableContext {
    Combat,
    Exploration,
    Treasure,
}

#[cfg(test)]
mod tests {
    use super::*;
    use specs::{World, WorldExt};

    fn setup_world() -> World {
        let mut world = World::new();
        world.register::<Item>();
        world.register::<Name>();
        world.register::<ItemProperties>();
        world.register::<Consumable>();
        world.register::<ItemStack>();
        world.register::<Position>();
        world.register::<Renderable>();
        world
    }

    #[test]
    fn test_health_potion_creation() {
        let mut world = setup_world();
        let factory = ConsumableFactory::new();
        let position = Position { x: 0, y: 0 };

        let potion = factory.create_health_potion(&mut world, position, PotionPotency::Lesser);

        let names = world.read_storage::<Name>();
        let consumables = world.read_storage::<Consumable>();
        let properties = world.read_storage::<ItemProperties>();

        let name = names.get(potion).unwrap();
        let consumable = consumables.get(potion).unwrap();
        let props = properties.get(potion).unwrap();

        assert_eq!(name.name, "Health Potion");
        assert_eq!(consumable.consumable_type, ConsumableType::Potion);
        assert!(!consumable.effects.is_empty());
        assert_eq!(props.value, 50);
    }

    #[test]
    fn test_food_creation() {
        let mut world = setup_world();
        let factory = ConsumableFactory::new();
        let position = Position { x: 0, y: 0 };

        let food = factory.create_food(&mut world, position, FoodType::Bread);

        let names = world.read_storage::<Name>();
        let consumables = world.read_storage::<Consumable>();

        let name = names.get(food).unwrap();
        let consumable = consumables.get(food).unwrap();

        assert_eq!(name.name, "Bread");
        assert_eq!(consumable.consumable_type, ConsumableType::Food);
        assert_eq!(consumable.use_time, 3.0); // Food takes longer to consume
    }

    #[test]
    fn test_scroll_creation() {
        let mut world = setup_world();
        let factory = ConsumableFactory::new();
        let position = Position { x: 0, y: 0 };

        let scroll = factory.create_scroll(&mut world, position, ScrollType::Healing);

        let names = world.read_storage::<Name>();
        let consumables = world.read_storage::<Consumable>();

        let name = names.get(scroll).unwrap();
        let consumable = consumables.get(scroll).unwrap();

        assert_eq!(name.name, "Scroll of Healing");
        assert_eq!(consumable.consumable_type, ConsumableType::Scroll);
        assert_eq!(consumable.charges, Some(1));
    }

    #[test]
    fn test_emergency_potion_restrictions() {
        let mut world = setup_world();
        let factory = ConsumableFactory::new();
        let position = Position { x: 0, y: 0 };

        let potion = factory.create_emergency_potion(&mut world, position);

        let consumables = world.read_storage::<Consumable>();
        let consumable = consumables.get(potion).unwrap();

        assert!(!consumable.restrictions.is_empty());
        assert_eq!(consumable.cooldown, 300.0);
        assert_eq!(consumable.use_time, 0.5);
    }

    #[test]
    fn test_random_consumable_generation() {
        let mut world = setup_world();
        let factory = ConsumableFactory::new();
        let mut rng = RandomNumberGenerator::new();
        let position = Position { x: 0, y: 0 };

        // Test different contexts
        let combat_item = factory.create_random_consumable(&mut world, position, ConsumableContext::Combat, &mut rng);
        let exploration_item = factory.create_random_consumable(&mut world, position, ConsumableContext::Exploration, &mut rng);
        let treasure_item = factory.create_random_consumable(&mut world, position, ConsumableContext::Treasure, &mut rng);

        let names = world.read_storage::<Name>();
        
        // All items should have names
        assert!(names.get(combat_item).is_some());
        assert!(names.get(exploration_item).is_some());
        assert!(names.get(treasure_item).is_some());
    }
}