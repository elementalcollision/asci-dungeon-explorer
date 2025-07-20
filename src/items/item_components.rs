use specs::{Component, VecStorage, NullStorage};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

// Basic item component - already exists in components/mod.rs but we'll extend it
#[derive(Component, Debug, Serialize, Deserialize, Clone)]
#[storage(VecStorage)]
pub struct ItemProperties {
    pub name: String,
    pub description: String,
    pub item_type: ItemType,
    pub rarity: ItemRarity,
    pub value: i32,
    pub weight: f32,
    pub stack_size: i32,
    pub durability: Option<ItemDurability>,
    pub requirements: ItemRequirements,
    pub tags: Vec<ItemTag>,
}

impl ItemProperties {
    pub fn new(name: String, item_type: ItemType) -> Self {
        ItemProperties {
            name,
            description: String::new(),
            item_type,
            rarity: ItemRarity::Common,
            value: 1,
            weight: 1.0,
            stack_size: 1,
            durability: None,
            requirements: ItemRequirements::default(),
            tags: Vec::new(),
        }
    }

    pub fn with_description(mut self, description: String) -> Self {
        self.description = description;
        self
    }

    pub fn with_rarity(mut self, rarity: ItemRarity) -> Self {
        self.rarity = rarity;
        self
    }

    pub fn with_value(mut self, value: i32) -> Self {
        self.value = value;
        self
    }

    pub fn with_weight(mut self, weight: f32) -> Self {
        self.weight = weight;
        self
    }

    pub fn with_stack_size(mut self, stack_size: i32) -> Self {
        self.stack_size = stack_size;
        self
    }

    pub fn with_durability(mut self, max_durability: i32) -> Self {
        self.durability = Some(ItemDurability {
            current: max_durability,
            max: max_durability,
        });
        self
    }

    pub fn with_requirements(mut self, requirements: ItemRequirements) -> Self {
        self.requirements = requirements;
        self
    }

    pub fn add_tag(mut self, tag: ItemTag) -> Self {
        self.tags.push(tag);
        self
    }

    pub fn has_tag(&self, tag: &ItemTag) -> bool {
        self.tags.contains(tag)
    }

    pub fn is_broken(&self) -> bool {
        if let Some(durability) = &self.durability {
            durability.current <= 0
        } else {
            false
        }
    }

    pub fn repair(&mut self, amount: i32) {
        if let Some(durability) = &mut self.durability {
            durability.current = (durability.current + amount).min(durability.max);
        }
    }

    pub fn damage(&mut self, amount: i32) {
        if let Some(durability) = &mut self.durability {
            durability.current = (durability.current - amount).max(0);
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum ItemType {
    // Weapons
    Weapon(WeaponType),
    // Armor
    Armor(ArmorType),
    // Consumables
    Consumable(ConsumableType),
    // Tools and utilities
    Tool(ToolType),
    // Quest items
    Quest,
    // Crafting materials
    Material(MaterialType),
    // Miscellaneous
    Miscellaneous,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum WeaponType {
    Sword,
    Axe,
    Mace,
    Dagger,
    Spear,
    Bow,
    Crossbow,
    Staff,
    Wand,
    Thrown,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum ArmorType {
    Helmet,
    Chest,
    Legs,
    Boots,
    Gloves,
    Shield,
    Cloak,
    Ring,
    Amulet,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum ConsumableType {
    Potion,
    Food,
    Scroll,
    Ammunition,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum ToolType {
    Lockpick,
    Torch,
    Rope,
    Key,
    Container,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum MaterialType {
    Metal,
    Wood,
    Leather,
    Cloth,
    Gem,
    Herb,
    Bone,
    Stone,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ItemRarity {
    Trash,
    Common,
    Uncommon,
    Rare,
    Epic,
    Legendary,
    Artifact,
}

impl ItemRarity {
    pub fn color(&self) -> (u8, u8, u8) {
        match self {
            ItemRarity::Trash => (128, 128, 128),     // Gray
            ItemRarity::Common => (255, 255, 255),    // White
            ItemRarity::Uncommon => (0, 255, 0),      // Green
            ItemRarity::Rare => (0, 100, 255),        // Blue
            ItemRarity::Epic => (128, 0, 128),        // Purple
            ItemRarity::Legendary => (255, 165, 0),   // Orange
            ItemRarity::Artifact => (255, 215, 0),    // Gold
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            ItemRarity::Trash => "Trash",
            ItemRarity::Common => "Common",
            ItemRarity::Uncommon => "Uncommon",
            ItemRarity::Rare => "Rare",
            ItemRarity::Epic => "Epic",
            ItemRarity::Legendary => "Legendary",
            ItemRarity::Artifact => "Artifact",
        }
    }

    pub fn value_multiplier(&self) -> f32 {
        match self {
            ItemRarity::Trash => 0.1,
            ItemRarity::Common => 1.0,
            ItemRarity::Uncommon => 2.0,
            ItemRarity::Rare => 5.0,
            ItemRarity::Epic => 10.0,
            ItemRarity::Legendary => 25.0,
            ItemRarity::Artifact => 100.0,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ItemDurability {
    pub current: i32,
    pub max: i32,
}

impl ItemDurability {
    pub fn percentage(&self) -> f32 {
        if self.max == 0 {
            1.0
        } else {
            self.current as f32 / self.max as f32
        }
    }

    pub fn is_broken(&self) -> bool {
        self.current <= 0
    }

    pub fn condition_name(&self) -> &'static str {
        let percentage = self.percentage();
        if percentage >= 0.9 {
            "Excellent"
        } else if percentage >= 0.7 {
            "Good"
        } else if percentage >= 0.5 {
            "Fair"
        } else if percentage >= 0.3 {
            "Poor"
        } else if percentage > 0.0 {
            "Broken"
        } else {
            "Destroyed"
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ItemRequirements {
    pub level: i32,
    pub strength: i32,
    pub dexterity: i32,
    pub constitution: i32,
    pub intelligence: i32,
    pub wisdom: i32,
    pub charisma: i32,
    pub skills: HashMap<String, i32>,
}

impl ItemRequirements {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_level(mut self, level: i32) -> Self {
        self.level = level;
        self
    }

    pub fn with_strength(mut self, strength: i32) -> Self {
        self.strength = strength;
        self
    }

    pub fn with_dexterity(mut self, dexterity: i32) -> Self {
        self.dexterity = dexterity;
        self
    }

    pub fn with_skill(mut self, skill: String, level: i32) -> Self {
        self.skills.insert(skill, level);
        self
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub enum ItemTag {
    // General tags
    Magical,
    Cursed,
    Blessed,
    Unique,
    Set,
    
    // Weapon tags
    TwoHanded,
    Ranged,
    Thrown,
    Finesse,
    Heavy,
    Light,
    
    // Armor tags
    Heavy,
    Medium,
    Light,
    
    // Consumable tags
    Healing,
    Mana,
    Buff,
    Debuff,
    Poison,
    
    // Utility tags
    Lockpicking,
    Illumination,
    Container,
    Key,
    
    // Material tags
    Flammable,
    Fragile,
    Valuable,
    Rare,
    
    // Quest tags
    QuestItem,
    KeyItem,
    
    // Custom tags
    Custom(String),
}

// Component for stackable items
#[derive(Component, Debug, Serialize, Deserialize, Clone)]
#[storage(VecStorage)]
pub struct ItemStack {
    pub quantity: i32,
    pub max_stack: i32,
}

impl ItemStack {
    pub fn new(quantity: i32, max_stack: i32) -> Self {
        ItemStack {
            quantity: quantity.min(max_stack),
            max_stack,
        }
    }

    pub fn is_full(&self) -> bool {
        self.quantity >= self.max_stack
    }

    pub fn can_add(&self, amount: i32) -> bool {
        self.quantity + amount <= self.max_stack
    }

    pub fn add(&mut self, amount: i32) -> i32 {
        let can_add = (self.max_stack - self.quantity).min(amount);
        self.quantity += can_add;
        amount - can_add // Return overflow
    }

    pub fn remove(&mut self, amount: i32) -> i32 {
        let can_remove = self.quantity.min(amount);
        self.quantity -= can_remove;
        can_remove
    }

    pub fn is_empty(&self) -> bool {
        self.quantity <= 0
    }
}

// Component for items that can be identified
#[derive(Component, Debug, Serialize, Deserialize, Clone)]
#[storage(VecStorage)]
pub struct ItemIdentification {
    pub identified: bool,
    pub unidentified_name: String,
    pub unidentified_description: String,
}

impl ItemIdentification {
    pub fn new(unidentified_name: String) -> Self {
        ItemIdentification {
            identified: false,
            unidentified_name,
            unidentified_description: "An unidentified item.".to_string(),
        }
    }

    pub fn with_description(mut self, description: String) -> Self {
        self.unidentified_description = description;
        self
    }

    pub fn identify(&mut self) {
        self.identified = true;
    }
}

// Component for items with magical properties
#[derive(Component, Debug, Serialize, Deserialize, Clone)]
#[storage(VecStorage)]
pub struct MagicalItem {
    pub enchantments: Vec<Enchantment>,
    pub curse: Option<Curse>,
    pub magic_level: i32,
}

impl MagicalItem {
    pub fn new(magic_level: i32) -> Self {
        MagicalItem {
            enchantments: Vec::new(),
            curse: None,
            magic_level,
        }
    }

    pub fn add_enchantment(&mut self, enchantment: Enchantment) {
        self.enchantments.push(enchantment);
    }

    pub fn add_curse(&mut self, curse: Curse) {
        self.curse = Some(curse);
    }

    pub fn is_cursed(&self) -> bool {
        self.curse.is_some()
    }

    pub fn total_enchantment_power(&self) -> i32 {
        self.enchantments.iter().map(|e| e.power).sum()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Enchantment {
    pub name: String,
    pub description: String,
    pub enchantment_type: EnchantmentType,
    pub power: i32,
    pub duration: Option<i32>, // For temporary enchantments
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum EnchantmentType {
    // Weapon enchantments
    Sharpness,
    Fire,
    Ice,
    Lightning,
    Poison,
    Vampiric,
    
    // Armor enchantments
    Protection,
    Resistance(String), // Resistance to specific damage type
    Regeneration,
    Stealth,
    
    // Utility enchantments
    Light,
    Teleportation,
    Detection,
    
    // Stat bonuses
    AttributeBonus(String, i32),
    SkillBonus(String, i32),
    
    // Custom enchantments
    Custom(String),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Curse {
    pub name: String,
    pub description: String,
    pub curse_type: CurseType,
    pub power: i32,
    pub removable: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum CurseType {
    // Equipment curses
    Binding, // Cannot be unequipped
    Fragility, // Breaks more easily
    Weight, // Becomes heavier
    
    // Combat curses
    Weakness, // Reduces damage
    Vulnerability, // Takes more damage
    Accuracy, // Reduces hit chance
    
    // Stat curses
    AttributePenalty(String, i32),
    SkillPenalty(String, i32),
    
    // Special curses
    Hunger, // Increases food consumption
    Thirst, // Increases water consumption
    Madness, // Random actions
    
    // Custom curses
    Custom(String),
}

// Component for items that provide stat bonuses
#[derive(Component, Debug, Serialize, Deserialize, Clone)]
#[storage(VecStorage)]
pub struct ItemBonuses {
    pub attribute_bonuses: HashMap<String, i32>,
    pub skill_bonuses: HashMap<String, i32>,
    pub combat_bonuses: CombatBonuses,
    pub special_bonuses: Vec<SpecialBonus>,
}

impl ItemBonuses {
    pub fn new() -> Self {
        ItemBonuses {
            attribute_bonuses: HashMap::new(),
            skill_bonuses: HashMap::new(),
            combat_bonuses: CombatBonuses::default(),
            special_bonuses: Vec::new(),
        }
    }

    pub fn add_attribute_bonus(&mut self, attribute: String, bonus: i32) {
        *self.attribute_bonuses.entry(attribute).or_insert(0) += bonus;
    }

    pub fn add_skill_bonus(&mut self, skill: String, bonus: i32) {
        *self.skill_bonuses.entry(skill).or_insert(0) += bonus;
    }

    pub fn add_special_bonus(&mut self, bonus: SpecialBonus) {
        self.special_bonuses.push(bonus);
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct CombatBonuses {
    pub attack_bonus: i32,
    pub damage_bonus: i32,
    pub defense_bonus: i32,
    pub critical_chance_bonus: i32,
    pub critical_damage_bonus: i32,
    pub speed_bonus: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SpecialBonus {
    pub name: String,
    pub description: String,
    pub bonus_type: SpecialBonusType,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum SpecialBonusType {
    // Resistance bonuses
    DamageResistance(String, i32), // Damage type, percentage
    StatusResistance(String, i32), // Status effect, percentage
    
    // Regeneration bonuses
    HealthRegeneration(i32),
    ManaRegeneration(i32),
    StaminaRegeneration(i32),
    
    // Vision bonuses
    NightVision,
    TrueSeeing,
    DetectMagic,
    
    // Movement bonuses
    WaterWalking,
    Levitation,
    SpeedBoost(i32),
    
    // Utility bonuses
    ExtraInventorySlots(i32),
    ReducedWeight(i32), // Percentage
    IncreasedCarryCapacity(i32),
    
    // Custom bonuses
    Custom(String),
}