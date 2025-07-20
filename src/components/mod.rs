use specs::{Component, VecStorage, NullStorage, World, WorldExt};
use specs_derive::Component;
use serde::{Serialize, Deserialize};

// Position component
#[derive(Component, Debug, Serialize, Deserialize, Clone)]
#[storage(VecStorage)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

// Renderable component
#[derive(Component, Debug, Serialize, Deserialize, Clone)]
#[storage(VecStorage)]
pub struct Renderable {
    pub glyph: char,
    pub fg: crossterm::style::Color,
    pub bg: crossterm::style::Color,
    pub render_order: i32,
}

// Player marker component
#[derive(Component, Debug, Serialize, Deserialize, Clone, Default)]
#[storage(NullStorage)]
pub struct Player;

// Viewshed component for field of view
#[derive(Component, Debug, Serialize, Deserialize, Clone)]
#[storage(VecStorage)]
pub struct Viewshed {
    pub visible_tiles: Vec<(i32, i32)>,
    pub range: i32,
    pub dirty: bool,
}

// Name component
#[derive(Component, Debug, Serialize, Deserialize, Clone)]
#[storage(VecStorage)]
pub struct Name {
    pub name: String,
}

// BlocksTile component for entities that block movement
#[derive(Component, Debug, Serialize, Deserialize, Clone, Default)]
#[storage(NullStorage)]
pub struct BlocksTile;

// Combat stats component
#[derive(Component, Debug, Serialize, Deserialize, Clone)]
#[storage(VecStorage)]
pub struct CombatStats {
    pub max_hp: i32,
    pub hp: i32,
    pub defense: i32,
    pub power: i32,
}

// Monster marker component
#[derive(Component, Debug, Serialize, Deserialize, Clone, Default)]
#[storage(NullStorage)]
pub struct Monster;

// Item marker component
#[derive(Component, Debug, Serialize, Deserialize, Clone, Default)]
#[storage(NullStorage)]
pub struct Item;

// Hidden component for things that aren't immediately visible
#[derive(Component, Debug, Serialize, Deserialize, Clone)]
#[storage(VecStorage)]
pub struct Hidden {
    pub hidden: bool,
}

// Equipment slot enum
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum EquipmentSlot {
    Melee,
    Ranged,
    Shield,
    Armor,
    Helmet,
    Boots,
    Gloves,
    Ring,
    Amulet,
}

// Equippable component
#[derive(Component, Debug, Serialize, Deserialize, Clone)]
#[storage(VecStorage)]
pub struct Equippable {
    pub slot: EquipmentSlot,
}

// Provides healing component
#[derive(Component, Debug, Serialize, Deserialize, Clone)]
#[storage(VecStorage)]
pub struct ProvidesHealing {
    pub heal_amount: i32,
}

// Melee power bonus component
#[derive(Component, Debug, Serialize, Deserialize, Clone)]
#[storage(VecStorage)]
pub struct MeleePowerBonus {
    pub power: i32,
}

// Defense bonus component
#[derive(Component, Debug, Serialize, Deserialize, Clone)]
#[storage(VecStorage)]
pub struct DefenseBonus {
    pub defense: i32,
}

// Abilities component for tracking character abilities
#[derive(Component, Debug, Serialize, Deserialize, Clone)]
#[storage(VecStorage)]
pub struct Abilities {
    pub abilities: std::collections::HashSet<AbilityType>,
    pub ability_cooldowns: std::collections::HashMap<AbilityType, i32>,
}

impl Abilities {
    pub fn new() -> Self {
        Abilities {
            abilities: std::collections::HashSet::new(),
            ability_cooldowns: std::collections::HashMap::new(),
        }
    }
    
    pub fn has_ability(&self, ability_type: AbilityType) -> bool {
        self.abilities.contains(&ability_type)
    }
    
    pub fn add_ability(&mut self, ability_type: AbilityType) {
        self.abilities.insert(ability_type);
    }
    
    pub fn is_on_cooldown(&self, ability_type: AbilityType) -> bool {
        self.ability_cooldowns.get(&ability_type).map_or(false, |&cd| cd > 0)
    }
    
    pub fn get_cooldown(&self, ability_type: AbilityType) -> i32 {
        *self.ability_cooldowns.get(&ability_type).unwrap_or(&0)
    }
    
    pub fn set_cooldown(&mut self, ability_type: AbilityType, cooldown: i32) {
        self.ability_cooldowns.insert(ability_type, cooldown);
    }
    
    pub fn update_cooldowns(&mut self) {
        for cooldown in self.ability_cooldowns.values_mut() {
            if *cooldown > 0 {
                *cooldown -= 1;
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AbilityType {
    // Fighter abilities
    PowerAttack,
    Cleave,
    ShieldBash,
    SecondWind,
    
    // Rogue abilities
    Backstab,
    Evasion,
    ShadowStep,
    DisarmTrap,
    
    // Mage abilities
    Fireball,
    IceSpike,
    MagicMissile,
    Teleport,
    
    // Cleric abilities
    Heal,
    TurnUndead,
    BlessWeapon,
    DivineProtection,
    
    // Ranger abilities
    PreciseShot,
    AnimalCompanion,
    TrackEnemy,
    NaturalRemedy,
}

impl AbilityType {
    pub fn name(&self) -> &'static str {
        match self {
            // Fighter abilities
            AbilityType::PowerAttack => "Power Attack",
            AbilityType::Cleave => "Cleave",
            AbilityType::ShieldBash => "Shield Bash",
            AbilityType::SecondWind => "Second Wind",
            
            // Rogue abilities
            AbilityType::Backstab => "Backstab",
            AbilityType::Evasion => "Evasion",
            AbilityType::ShadowStep => "Shadow Step",
            AbilityType::DisarmTrap => "Disarm Trap",
            
            // Mage abilities
            AbilityType::Fireball => "Fireball",
            AbilityType::IceSpike => "Ice Spike",
            AbilityType::MagicMissile => "Magic Missile",
            AbilityType::Teleport => "Teleport",
            
            // Cleric abilities
            AbilityType::Heal => "Heal",
            AbilityType::TurnUndead => "Turn Undead",
            AbilityType::BlessWeapon => "Bless Weapon",
            AbilityType::DivineProtection => "Divine Protection",
            
            // Ranger abilities
            AbilityType::PreciseShot => "Precise Shot",
            AbilityType::AnimalCompanion => "Animal Companion",
            AbilityType::TrackEnemy => "Track Enemy",
            AbilityType::NaturalRemedy => "Natural Remedy",
        }
    }
    
    pub fn description(&self) -> &'static str {
        match self {
            // Fighter abilities
            AbilityType::PowerAttack => "A powerful attack that sacrifices accuracy for increased damage.",
            AbilityType::Cleave => "A sweeping attack that can hit multiple adjacent enemies.",
            AbilityType::ShieldBash => "Bash an enemy with your shield, stunning them briefly.",
            AbilityType::SecondWind => "Recover a portion of your health through sheer determination.",
            
            // Rogue abilities
            AbilityType::Backstab => "A deadly attack from stealth that deals massive damage.",
            AbilityType::Evasion => "Dodge incoming attacks with increased chance of success.",
            AbilityType::ShadowStep => "Teleport to a nearby shadow, becoming hidden.",
            AbilityType::DisarmTrap => "Safely disarm a detected trap.",
            
            // Mage abilities
            AbilityType::Fireball => "Launch a ball of fire that explodes on impact, damaging multiple enemies.",
            AbilityType::IceSpike => "Fire a spike of ice that slows and damages an enemy.",
            AbilityType::MagicMissile => "Fire magical bolts that never miss their target.",
            AbilityType::Teleport => "Instantly teleport to a visible location.",
            
            // Cleric abilities
            AbilityType::Heal => "Restore health to yourself or an ally.",
            AbilityType::TurnUndead => "Force undead creatures to flee in terror.",
            AbilityType::BlessWeapon => "Temporarily enhance a weapon with divine power.",
            AbilityType::DivineProtection => "Create a shield of divine energy that absorbs damage.",
            
            // Ranger abilities
            AbilityType::PreciseShot => "A carefully aimed shot that deals extra damage.",
            AbilityType::AnimalCompanion => "Summon an animal companion to fight alongside you.",
            AbilityType::TrackEnemy => "Reveal the location of nearby enemies.",
            AbilityType::NaturalRemedy => "Use natural ingredients to create a healing poultice.",
        }
    }
    
    pub fn cooldown(&self) -> i32 {
        match self {
            // Fighter abilities
            AbilityType::PowerAttack => 3,
            AbilityType::Cleave => 5,
            AbilityType::ShieldBash => 4,
            AbilityType::SecondWind => 10,
            
            // Rogue abilities
            AbilityType::Backstab => 5,
            AbilityType::Evasion => 8,
            AbilityType::ShadowStep => 6,
            AbilityType::DisarmTrap => 0, // Passive ability
            
            // Mage abilities
            AbilityType::Fireball => 6,
            AbilityType::IceSpike => 4,
            AbilityType::MagicMissile => 3,
            AbilityType::Teleport => 10,
            
            // Cleric abilities
            AbilityType::Heal => 5,
            AbilityType::TurnUndead => 8,
            AbilityType::BlessWeapon => 10,
            AbilityType::DivineProtection => 12,
            
            // Ranger abilities
            AbilityType::PreciseShot => 4,
            AbilityType::AnimalCompanion => 15,
            AbilityType::TrackEnemy => 8,
            AbilityType::NaturalRemedy => 6,
        }
    }
    
    pub fn required_level(&self) -> i32 {
        match self {
            // Fighter abilities
            AbilityType::PowerAttack => 1,
            AbilityType::Cleave => 3,
            AbilityType::ShieldBash => 5,
            AbilityType::SecondWind => 7,
            
            // Rogue abilities
            AbilityType::Backstab => 1,
            AbilityType::Evasion => 3,
            AbilityType::ShadowStep => 5,
            AbilityType::DisarmTrap => 2,
            
            // Mage abilities
            AbilityType::MagicMissile => 1,
            AbilityType::Fireball => 3,
            AbilityType::IceSpike => 5,
            AbilityType::Teleport => 7,
            
            // Cleric abilities
            AbilityType::Heal => 1,
            AbilityType::TurnUndead => 3,
            AbilityType::BlessWeapon => 5,
            AbilityType::DivineProtection => 7,
            
            // Ranger abilities
            AbilityType::PreciseShot => 1,
            AbilityType::TrackEnemy => 3,
            AbilityType::NaturalRemedy => 5,
            AbilityType::AnimalCompanion => 7,
        }
    }
    
    pub fn get_class_abilities(class_type: ClassType) -> Vec<AbilityType> {
        match class_type {
            ClassType::Fighter => vec![
                AbilityType::PowerAttack,
                AbilityType::Cleave,
                AbilityType::ShieldBash,
                AbilityType::SecondWind,
            ],
            ClassType::Rogue => vec![
                AbilityType::Backstab,
                AbilityType::Evasion,
                AbilityType::ShadowStep,
                AbilityType::DisarmTrap,
            ],
            ClassType::Mage => vec![
                AbilityType::MagicMissile,
                AbilityType::Fireball,
                AbilityType::IceSpike,
                AbilityType::Teleport,
            ],
            ClassType::Cleric => vec![
                AbilityType::Heal,
                AbilityType::TurnUndead,
                AbilityType::BlessWeapon,
                AbilityType::DivineProtection,
            ],
            ClassType::Ranger => vec![
                AbilityType::PreciseShot,
                AbilityType::TrackEnemy,
                AbilityType::NaturalRemedy,
                AbilityType::AnimalCompanion,
            ],
        }
    }
}

// Player input component for handling player actions
#[derive(Component, Debug, Serialize, Deserialize, Clone)]
#[storage(VecStorage)]
pub struct PlayerInput {
    pub move_intent: Option<(i32, i32)>,
    pub attack_intent: Option<usize>,
    pub use_item_intent: Option<usize>,
    pub pickup_intent: bool,
    pub drop_intent: Option<usize>,
    pub wait_intent: bool,
    pub examine_intent: Option<(i32, i32)>,
}

impl PlayerInput {
    pub fn new() -> Self {
        PlayerInput {
            move_intent: None,
            attack_intent: None,
            use_item_intent: None,
            pickup_intent: false,
            drop_intent: None,
            wait_intent: false,
            examine_intent: None,
        }
    }
    
    pub fn clear(&mut self) {
        self.move_intent = None;
        self.attack_intent = None;
        self.use_item_intent = None;
        self.pickup_intent = false;
        self.drop_intent = None;
        self.wait_intent = false;
        self.examine_intent = None;
    }
}

// WantsToMove component for movement intent
#[derive(Component, Debug, Serialize, Deserialize, Clone)]
#[storage(VecStorage)]
pub struct WantsToMove {
    pub destination: (i32, i32),
}

// WantsToAttack component for attack intent
#[derive(Component, Debug, Serialize, Deserialize, Clone)]
#[storage(VecStorage)]
pub struct WantsToAttack {
    pub target: specs::Entity,
}

// WantsToPickupItem component for item pickup intent
#[derive(Component, Debug, Serialize, Deserialize, Clone)]
#[storage(VecStorage)]
pub struct WantsToPickupItem {
    pub item: specs::Entity,
}

// WantsToUseItem component for item usage intent
#[derive(Component, Debug, Serialize, Deserialize, Clone)]
#[storage(VecStorage)]
pub struct WantsToUseItem {
    pub item: specs::Entity,
    pub target: Option<specs::Entity>,
}

// WantsToDropItem component for item drop intent
#[derive(Component, Debug, Serialize, Deserialize, Clone)]
#[storage(VecStorage)]
pub struct WantsToDropItem {
    pub item: specs::Entity,
}

// Death-related components
#[derive(Component, Debug, Serialize, Deserialize, Clone)]
#[storage(VecStorage)]
pub struct Dead {
    pub cause: DeathCause,
    pub time_of_death: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum DeathCause {
    Combat(specs::Entity), // Killed by another entity
    Environment,           // Environmental hazard
    Starvation,           // Died from hunger
    Poison,               // Died from poison
    Other(String),        // Other causes
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
#[storage(VecStorage)]
pub struct Corpse {
    pub original_entity: Option<specs::Entity>,
    pub decay_timer: i32,
    pub loot_generated: bool,
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
#[storage(VecStorage)]
pub struct DeathAnimation {
    pub animation_type: DeathAnimationType,
    pub duration: f32,
    pub elapsed: f32,
    pub original_glyph: char,
    pub original_color: crossterm::style::Color,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum DeathAnimationType {
    Fade,
    Dissolve,
    Explosion,
    Collapse,
}

// Inventory component for storing items
#[derive(Component, Debug, Serialize, Deserialize, Clone)]
#[storage(VecStorage)]
pub struct Inventory {
    pub items: Vec<specs::Entity>,
    pub capacity: usize,
}

impl Inventory {
    pub fn new(capacity: usize) -> Self {
        Inventory {
            items: Vec::new(),
            capacity,
        }
    }
    
    pub fn is_full(&self) -> bool {
        self.items.len() >= self.capacity
    }
}

// Equipped component for equipped items
#[derive(Component, Debug, Serialize, Deserialize, Clone)]
#[storage(VecStorage)]
pub struct Equipped {
    pub owner: specs::Entity,
    pub slot: EquipmentSlot,
}

// Experience component for player progression
#[derive(Component, Debug, Serialize, Deserialize, Clone)]
#[storage(VecStorage)]
pub struct Experience {
    pub current: i32,
    pub level: i32,
    pub level_up_target: i32,
    pub unspent_points: i32,
    pub total_exp_earned: i32,
}

impl Experience {
    pub fn new() -> Self {
        Experience {
            current: 0,
            level: 1,
            level_up_target: 100,
            unspent_points: 0,
            total_exp_earned: 0,
        }
    }
    
    pub fn gain_exp(&mut self, amount: i32) -> bool {
        self.current += amount;
        self.total_exp_earned += amount;
        
        if self.current >= self.level_up_target {
            self.level_up();
            return true;
        }
        false
    }
    
    pub fn level_up(&mut self) {
        self.level += 1;
        self.current -= self.level_up_target;
        self.level_up_target = (self.level_up_target as f32 * 1.5) as i32;
        self.unspent_points += 3; // Grant 3 points per level
    }
    
    pub fn exp_to_next_level(&self) -> i32 {
        self.level_up_target - self.current
    }
    
    pub fn progress_percentage(&self) -> f32 {
        (self.current as f32 / self.level_up_target as f32) * 100.0
    }
}

// Character attributes component
#[derive(Component, Debug, Serialize, Deserialize, Clone)]
#[storage(VecStorage)]
pub struct Attributes {
    pub strength: i32,
    pub dexterity: i32,
    pub constitution: i32,
    pub intelligence: i32,
    pub wisdom: i32,
    pub charisma: i32,
    pub unspent_points: i32,
}

impl Attributes {
    pub fn new() -> Self {
        Attributes {
            strength: 8,
            dexterity: 8,
            constitution: 8,
            intelligence: 8,
            wisdom: 8,
            charisma: 8,
            unspent_points: 8,
        }
    }
    
    pub fn get_modifier(&self, attribute: AttributeType) -> i32 {
        let value = match attribute {
            AttributeType::Strength => self.strength,
            AttributeType::Dexterity => self.dexterity,
            AttributeType::Constitution => self.constitution,
            AttributeType::Intelligence => self.intelligence,
            AttributeType::Wisdom => self.wisdom,
            AttributeType::Charisma => self.charisma,
        };
        
        // Calculate modifier: (value - 10) / 2, rounded down
        (value - 10) / 2
    }
    
    pub fn increase_attribute(&mut self, attribute: AttributeType) -> bool {
        if self.unspent_points <= 0 {
            return false;
        }
        
        match attribute {
            AttributeType::Strength => {
                if self.strength < 18 {
                    self.strength += 1;
                    self.unspent_points -= 1;
                    return true;
                }
            },
            AttributeType::Dexterity => {
                if self.dexterity < 18 {
                    self.dexterity += 1;
                    self.unspent_points -= 1;
                    return true;
                }
            },
            AttributeType::Constitution => {
                if self.constitution < 18 {
                    self.constitution += 1;
                    self.unspent_points -= 1;
                    return true;
                }
            },
            AttributeType::Intelligence => {
                if self.intelligence < 18 {
                    self.intelligence += 1;
                    self.unspent_points -= 1;
                    return true;
                }
            },
            AttributeType::Wisdom => {
                if self.wisdom < 18 {
                    self.wisdom += 1;
                    self.unspent_points -= 1;
                    return true;
                }
            },
            AttributeType::Charisma => {
                if self.charisma < 18 {
                    self.charisma += 1;
                    self.unspent_points -= 1;
                    return true;
                }
            },
        }
        
        false
    }
    
    pub fn decrease_attribute(&mut self, attribute: AttributeType) -> bool {
        match attribute {
            AttributeType::Strength => {
                if self.strength > 8 {
                    self.strength -= 1;
                    self.unspent_points += 1;
                    return true;
                }
            },
            AttributeType::Dexterity => {
                if self.dexterity > 8 {
                    self.dexterity -= 1;
                    self.unspent_points += 1;
                    return true;
                }
            },
            AttributeType::Constitution => {
                if self.constitution > 8 {
                    self.constitution -= 1;
                    self.unspent_points += 1;
                    return true;
                }
            },
            AttributeType::Intelligence => {
                if self.intelligence > 8 {
                    self.intelligence -= 1;
                    self.unspent_points += 1;
                    return true;
                }
            },
            AttributeType::Wisdom => {
                if self.wisdom > 8 {
                    self.wisdom -= 1;
                    self.unspent_points += 1;
                    return true;
                }
            },
            AttributeType::Charisma => {
                if self.charisma > 8 {
                    self.charisma -= 1;
                    self.unspent_points += 1;
                    return true;
                }
            },
        }
        
        false
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
pub enum AttributeType {
    Strength,
    Dexterity,
    Constitution,
    Intelligence,
    Wisdom,
    Charisma,
}

// Character class component
#[derive(Component, Debug, Serialize, Deserialize, Clone, PartialEq)]
#[storage(VecStorage)]
pub struct CharacterClass {
    pub class_type: ClassType,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
pub enum ClassType {
    Fighter,
    Rogue,
    Mage,
    Cleric,
    Ranger,
}

impl ClassType {
    pub fn name(&self) -> &'static str {
        match self {
            ClassType::Fighter => "Fighter",
            ClassType::Rogue => "Rogue",
            ClassType::Mage => "Mage",
            ClassType::Cleric => "Cleric",
            ClassType::Ranger => "Ranger",
        }
    }
    
    pub fn description(&self) -> &'static str {
        match self {
            ClassType::Fighter => "A skilled warrior with high strength and constitution. Specializes in melee combat and heavy armor.",
            ClassType::Rogue => "A nimble thief with high dexterity. Specializes in stealth, traps, and critical strikes.",
            ClassType::Mage => "A powerful spellcaster with high intelligence. Specializes in offensive magic and arcane knowledge.",
            ClassType::Cleric => "A divine spellcaster with high wisdom. Specializes in healing, support magic, and undead turning.",
            ClassType::Ranger => "A skilled hunter with high dexterity and wisdom. Specializes in ranged combat and survival skills.",
        }
    }
    
    pub fn primary_attribute(&self) -> AttributeType {
        match self {
            ClassType::Fighter => AttributeType::Strength,
            ClassType::Rogue => AttributeType::Dexterity,
            ClassType::Mage => AttributeType::Intelligence,
            ClassType::Cleric => AttributeType::Wisdom,
            ClassType::Ranger => AttributeType::Dexterity,
        }
    }
    
    pub fn secondary_attribute(&self) -> AttributeType {
        match self {
            ClassType::Fighter => AttributeType::Constitution,
            ClassType::Rogue => AttributeType::Charisma,
            ClassType::Mage => AttributeType::Wisdom,
            ClassType::Cleric => AttributeType::Charisma,
            ClassType::Ranger => AttributeType::Wisdom,
        }
    }
    
    pub fn starting_hp(&self) -> i32 {
        match self {
            ClassType::Fighter => 12,
            ClassType::Rogue => 8,
            ClassType::Mage => 6,
            ClassType::Cleric => 10,
            ClassType::Ranger => 10,
        }
    }
    
    pub fn hp_per_level(&self) -> i32 {
        match self {
            ClassType::Fighter => 10,
            ClassType::Rogue => 6,
            ClassType::Mage => 4,
            ClassType::Cleric => 8,
            ClassType::Ranger => 8,
        }
    }
}

// Character background component
#[derive(Component, Debug, Serialize, Deserialize, Clone, PartialEq)]
#[storage(VecStorage)]
pub struct Background {
    pub background_type: BackgroundType,
}

// Skills component for tracking character skills
#[derive(Component, Debug, Serialize, Deserialize, Clone)]
#[storage(VecStorage)]
pub struct Skills {
    pub skills: std::collections::HashMap<SkillType, i32>,
    pub unspent_skill_points: i32,
}

impl Skills {
    pub fn new() -> Self {
        let mut skills = std::collections::HashMap::new();
        
        // Initialize all skills at level 0
        for skill_type in SkillType::all() {
            skills.insert(skill_type, 0);
        }
        
        // Add starting skills based on class and background later
        
        Skills {
            skills,
            unspent_skill_points: 0,
        }
    }
    
    pub fn get_skill_level(&self, skill_type: SkillType) -> i32 {
        *self.skills.get(&skill_type).unwrap_or(&0)
    }
    
    pub fn increase_skill(&mut self, skill_type: SkillType) -> bool {
        if self.unspent_skill_points <= 0 {
            return false;
        }
        
        let current_level = self.get_skill_level(skill_type);
        if current_level < 5 { // Maximum skill level is 5
            self.skills.insert(skill_type, current_level + 1);
            self.unspent_skill_points -= 1;
            return true;
        }
        
        false
    }
    
    pub fn add_skill_points(&mut self, points: i32) {
        self.unspent_skill_points += points;
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SkillType {
    // Combat skills
    MeleeWeapons,
    RangedWeapons,
    Unarmed,
    Defense,
    
    // Magic skills
    Arcane,
    Divine,
    Elemental,
    
    // Utility skills
    Stealth,
    Lockpicking,
    Perception,
    Survival,
    Persuasion,
    
    // Crafting skills
    Alchemy,
    Enchanting,
}

impl SkillType {
    pub fn all() -> Vec<SkillType> {
        vec![
            SkillType::MeleeWeapons,
            SkillType::RangedWeapons,
            SkillType::Unarmed,
            SkillType::Defense,
            SkillType::Arcane,
            SkillType::Divine,
            SkillType::Elemental,
            SkillType::Stealth,
            SkillType::Lockpicking,
            SkillType::Perception,
            SkillType::Survival,
            SkillType::Persuasion,
            SkillType::Alchemy,
            SkillType::Enchanting,
        ]
    }
    
    pub fn name(&self) -> &'static str {
        match self {
            SkillType::MeleeWeapons => "Melee Weapons",
            SkillType::RangedWeapons => "Ranged Weapons",
            SkillType::Unarmed => "Unarmed Combat",
            SkillType::Defense => "Defense",
            SkillType::Arcane => "Arcane Magic",
            SkillType::Divine => "Divine Magic",
            SkillType::Elemental => "Elemental Magic",
            SkillType::Stealth => "Stealth",
            SkillType::Lockpicking => "Lockpicking",
            SkillType::Perception => "Perception",
            SkillType::Survival => "Survival",
            SkillType::Persuasion => "Persuasion",
            SkillType::Alchemy => "Alchemy",
            SkillType::Enchanting => "Enchanting",
        }
    }
    
    pub fn description(&self) -> &'static str {
        match self {
            SkillType::MeleeWeapons => "Proficiency with swords, axes, maces and other melee weapons.",
            SkillType::RangedWeapons => "Proficiency with bows, crossbows, and thrown weapons.",
            SkillType::Unarmed => "Skill in fighting without weapons.",
            SkillType::Defense => "Ability to avoid damage and reduce incoming attacks.",
            SkillType::Arcane => "Knowledge of arcane spells and magical theory.",
            SkillType::Divine => "Connection to divine powers and healing magic.",
            SkillType::Elemental => "Control over elemental forces like fire, ice, and lightning.",
            SkillType::Stealth => "Ability to move silently and remain undetected.",
            SkillType::Lockpicking => "Skill at opening locks without keys.",
            SkillType::Perception => "Awareness of surroundings and hidden objects.",
            SkillType::Survival => "Knowledge of wilderness survival and tracking.",
            SkillType::Persuasion => "Ability to influence others through speech.",
            SkillType::Alchemy => "Skill at creating potions and poisons.",
            SkillType::Enchanting => "Ability to imbue items with magical properties.",
        }
    }
    
    pub fn primary_attribute(&self) -> AttributeType {
        match self {
            SkillType::MeleeWeapons => AttributeType::Strength,
            SkillType::RangedWeapons => AttributeType::Dexterity,
            SkillType::Unarmed => AttributeType::Strength,
            SkillType::Defense => AttributeType::Constitution,
            SkillType::Arcane => AttributeType::Intelligence,
            SkillType::Divine => AttributeType::Wisdom,
            SkillType::Elemental => AttributeType::Intelligence,
            SkillType::Stealth => AttributeType::Dexterity,
            SkillType::Lockpicking => AttributeType::Dexterity,
            SkillType::Perception => AttributeType::Wisdom,
            SkillType::Survival => AttributeType::Wisdom,
            SkillType::Persuasion => AttributeType::Charisma,
            SkillType::Alchemy => AttributeType::Intelligence,
            SkillType::Enchanting => AttributeType::Intelligence,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
pub enum BackgroundType {
    Soldier,
    Scholar,
    Noble,
    Outlaw,
    Acolyte,
    Merchant,
}

impl BackgroundType {
    pub fn name(&self) -> &'static str {
        match self {
            BackgroundType::Soldier => "Soldier",
            BackgroundType::Scholar => "Scholar",
            BackgroundType::Noble => "Noble",
            BackgroundType::Outlaw => "Outlaw",
            BackgroundType::Acolyte => "Acolyte",
            BackgroundType::Merchant => "Merchant",
        }
    }
    
    pub fn description(&self) -> &'static str {
        match self {
            BackgroundType::Soldier => "You served in a military force, learning discipline and combat skills.",
            BackgroundType::Scholar => "You spent years studying in libraries and academies, gaining knowledge and arcane insights.",
            BackgroundType::Noble => "You were born into privilege, with education and resources but also responsibilities.",
            BackgroundType::Outlaw => "You lived outside the law, developing stealth and survival skills.",
            BackgroundType::Acolyte => "You served a temple or religious order, learning divine mysteries and rituals.",
            BackgroundType::Merchant => "You traveled as a trader, developing negotiation skills and worldly knowledge.",
        }
    }
    
    pub fn attribute_bonus(&self) -> AttributeType {
        match self {
            BackgroundType::Soldier => AttributeType::Strength,
            BackgroundType::Scholar => AttributeType::Intelligence,
            BackgroundType::Noble => AttributeType::Charisma,
            BackgroundType::Outlaw => AttributeType::Dexterity,
            BackgroundType::Acolyte => AttributeType::Wisdom,
            BackgroundType::Merchant => AttributeType::Charisma,
        }
    }
}

// Register all components with the world
pub fn register_components(world: &mut World) {
    world.register::<Position>();
    world.register::<Renderable>();
    world.register::<Player>();
    world.register::<Viewshed>();
    world.register::<Name>();
    world.register::<BlocksTile>();
    world.register::<CombatStats>();
    world.register::<Monster>();
    world.register::<Item>();
    world.register::<Hidden>();
    world.register::<Equippable>();
    world.register::<ProvidesHealing>();
    world.register::<MeleePowerBonus>();
    world.register::<DefenseBonus>();
    
    // Player-related components
    world.register::<PlayerInput>();
    world.register::<WantsToMove>();
    world.register::<WantsToAttack>();
    world.register::<WantsToPickupItem>();
    world.register::<WantsToUseItem>();
    world.register::<WantsToDropItem>();
    world.register::<Inventory>();
    world.register::<Equipped>();
    world.register::<Experience>();
    
    // Character creation components
    world.register::<Attributes>();
    world.register::<CharacterClass>();
    world.register::<Background>();
    
    // Character progression components
    world.register::<Skills>();
    world.register::<Abilities>();
    
    // Combat components
    world.register::<SufferDamage>();
    world.register::<Consumable>();
    
    // Resource management components
    world.register::<PlayerResources>();
    world.register::<StatusEffects>();
    world.register::<WantsToUseAbility>();
    
    // Death and revival components
    world.register::<DeathState>();
    world.register::<RevivalItem>();
    world.register::<DeathPenalty>();
    world.register::<GameSettings>();
    
    // Enhanced combat components
    world.register::<Attacker>();
    world.register::<Defender>();
    world.register::<DamageInfo>();
    world.register::<DamageResistances>();
    world.register::<CombatAction>();
    world.register::<Initiative>();
    world.register::<CombatFeedback>();
    world.register::<ParticleEffect>();
    
    // Combat rewards components
    world.register::<LootTable>();
    world.register::<UniqueEnemy>();
    world.register::<CombatReward>();
    world.register::<BossEnemy>();
    world.register::<Treasure>();
    world.register::<WantsToInteract>();
}

// Combat-related components
#[derive(Component, Debug, Serialize, Deserialize, Clone)]
#[storage(VecStorage)]
pub struct SufferDamage {
    pub amount: i32,
}

impl SufferDamage {
    pub fn new_damage(store: &mut WriteStorage<SufferDamage>, victim: Entity, amount: i32) {
        if let Some(suffering) = store.get_mut(victim) {
            suffering.amount += amount;
        } else {
            store.insert(victim, SufferDamage { amount })
                .expect("Unable to insert damage");
        }
    }
}

// Consumable marker component
#[derive(Component, Debug, Serialize, Deserialize, Clone, Default)]
#[storage(NullStorage)]
pub struct Consumable;

// Player resource management components
#[derive(Component, Debug, Serialize, Deserialize, Clone)]
#[storage(VecStorage)]
pub struct PlayerResources {
    pub mana: i32,
    pub max_mana: i32,
    pub stamina: i32,
    pub max_stamina: i32,
    pub mana_regen_rate: i32,
    pub stamina_regen_rate: i32,
    pub mana_regen_timer: i32,
    pub stamina_regen_timer: i32,
}

impl PlayerResources {
    pub fn new(max_mana: i32, max_stamina: i32) -> Self {
        PlayerResources {
            mana: max_mana,
            max_mana,
            stamina: max_stamina,
            max_stamina,
            mana_regen_rate: 1, // Regenerate 1 mana per 3 turns
            stamina_regen_rate: 2, // Regenerate 2 stamina per 2 turns
            mana_regen_timer: 0,
            stamina_regen_timer: 0,
        }
    }
    
    pub fn consume_mana(&mut self, amount: i32) -> bool {
        if self.mana >= amount {
            self.mana -= amount;
            true
        } else {
            false
        }
    }
    
    pub fn consume_stamina(&mut self, amount: i32) -> bool {
        if self.stamina >= amount {
            self.stamina -= amount;
            true
        } else {
            false
        }
    }
    
    pub fn restore_mana(&mut self, amount: i32) {
        self.mana = i32::min(self.mana + amount, self.max_mana);
    }
    
    pub fn restore_stamina(&mut self, amount: i32) {
        self.stamina = i32::min(self.stamina + amount, self.max_stamina);
    }
    
    pub fn mana_percentage(&self) -> f32 {
        if self.max_mana > 0 {
            (self.mana as f32 / self.max_mana as f32) * 100.0
        } else {
            0.0
        }
    }
    
    pub fn stamina_percentage(&self) -> f32 {
        if self.max_stamina > 0 {
            (self.stamina as f32 / self.max_stamina as f32) * 100.0
        } else {
            0.0
        }
    }
}

// Status effects that can affect resources
#[derive(Component, Debug, Serialize, Deserialize, Clone)]
#[storage(VecStorage)]
pub struct StatusEffects {
    pub effects: Vec<StatusEffect>,
}

impl StatusEffects {
    pub fn new() -> Self {
        StatusEffects {
            effects: Vec::new(),
        }
    }
    
    pub fn add_effect(&mut self, effect: StatusEffect) {
        // Check if effect already exists and update duration
        for existing_effect in &mut self.effects {
            if existing_effect.effect_type == effect.effect_type {
                existing_effect.duration = i32::max(existing_effect.duration, effect.duration);
                return;
            }
        }
        
        // Add new effect
        self.effects.push(effect);
    }
    
    pub fn remove_effect(&mut self, effect_type: StatusEffectType) {
        self.effects.retain(|effect| effect.effect_type != effect_type);
    }
    
    pub fn has_effect(&self, effect_type: StatusEffectType) -> bool {
        self.effects.iter().any(|effect| effect.effect_type == effect_type)
    }
    
    pub fn get_effect(&self, effect_type: StatusEffectType) -> Option<&StatusEffect> {
        self.effects.iter().find(|effect| effect.effect_type == effect_type)
    }
    
    pub fn update_effects(&mut self) {
        // Decrease duration and remove expired effects
        self.effects.retain_mut(|effect| {
            effect.duration -= 1;
            effect.duration > 0
        });
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StatusEffect {
    pub effect_type: StatusEffectType,
    pub duration: i32,
    pub magnitude: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
pub enum StatusEffectType {
    // Resource effects
    ManaRegenBoost,
    ManaRegenPenalty,
    StaminaRegenBoost,
    StaminaRegenPenalty,
    
    // Combat effects
    Poisoned,
    Blessed,
    Cursed,
    Haste,
    Slow,
    
    // Stat effects
    StrengthBoost,
    StrengthPenalty,
    DefenseBoost,
    DefensePenalty,
}

impl StatusEffectType {
    pub fn name(&self) -> &'static str {
        match self {
            StatusEffectType::ManaRegenBoost => "Mana Regeneration Boost",
            StatusEffectType::ManaRegenPenalty => "Mana Regeneration Penalty",
            StatusEffectType::StaminaRegenBoost => "Stamina Regeneration Boost",
            StatusEffectType::StaminaRegenPenalty => "Stamina Regeneration Penalty",
            StatusEffectType::Poisoned => "Poisoned",
            StatusEffectType::Blessed => "Blessed",
            StatusEffectType::Cursed => "Cursed",
            StatusEffectType::Haste => "Haste",
            StatusEffectType::Slow => "Slow",
            StatusEffectType::StrengthBoost => "Strength Boost",
            StatusEffectType::StrengthPenalty => "Strength Penalty",
            StatusEffectType::DefenseBoost => "Defense Boost",
            StatusEffectType::DefensePenalty => "Defense Penalty",
        }
    }
    
    pub fn is_beneficial(&self) -> bool {
        match self {
            StatusEffectType::ManaRegenBoost |
            StatusEffectType::StaminaRegenBoost |
            StatusEffectType::Blessed |
            StatusEffectType::Haste |
            StatusEffectType::StrengthBoost |
            StatusEffectType::DefenseBoost => true,
            _ => false,
        }
    }
}

// Component for actions that consume resources
#[derive(Component, Debug, Serialize, Deserialize, Clone)]
#[storage(VecStorage)]
pub struct WantsToUseAbility {
    pub ability: AbilityType,
    pub target: Option<Entity>,
    pub mana_cost: i32,
    pub stamina_cost: i32,
}

// Player death and revival components
#[derive(Component, Debug, Serialize, Deserialize, Clone)]
#[storage(VecStorage)]
pub struct DeathState {
    pub is_dead: bool,
    pub death_cause: String,
    pub death_turn: i32,
    pub revival_attempts: i32,
    pub max_revival_attempts: i32,
    pub death_penalty_applied: bool,
}

impl DeathState {
    pub fn new() -> Self {
        DeathState {
            is_dead: false,
            death_cause: String::new(),
            death_turn: 0,
            revival_attempts: 0,
            max_revival_attempts: 3,
            death_penalty_applied: false,
        }
    }
    
    pub fn kill(&mut self, cause: String, turn: i32) {
        self.is_dead = true;
        self.death_cause = cause;
        self.death_turn = turn;
        self.death_penalty_applied = false;
    }
    
    pub fn revive(&mut self) -> bool {
        if self.revival_attempts < self.max_revival_attempts {
            self.is_dead = false;
            self.revival_attempts += 1;
            true
        } else {
            false
        }
    }
    
    pub fn can_revive(&self) -> bool {
        self.revival_attempts < self.max_revival_attempts
    }
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
#[storage(VecStorage)]
pub struct RevivalItem {
    pub revival_power: i32,
    pub auto_use: bool,
    pub consumed_on_use: bool,
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
#[storage(VecStorage)]
pub struct DeathPenalty {
    pub experience_loss_percentage: f32,
    pub attribute_penalty: i32,
    pub equipment_durability_loss: i32,
    pub temporary_stat_reduction: i32,
    pub penalty_duration: i32,
}

impl DeathPenalty {
    pub fn new() -> Self {
        DeathPenalty {
            experience_loss_percentage: 10.0, // Lose 10% of current XP
            attribute_penalty: 1, // Temporary -1 to all attributes
            equipment_durability_loss: 5, // Equipment loses durability
            temporary_stat_reduction: 2, // -2 to combat stats temporarily
            penalty_duration: 50, // Penalty lasts 50 turns
        }
    }
    
    pub fn permadeath() -> Self {
        DeathPenalty {
            experience_loss_percentage: 100.0, // Lose all progress
            attribute_penalty: 0,
            equipment_durability_loss: 0,
            temporary_stat_reduction: 0,
            penalty_duration: 0,
        }
    }
}

// Game mode settings
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum GameMode {
    Normal,      // Standard death with penalties and revival
    Hardcore,    // Limited revivals with harsh penalties
    Permadeath,  // Single life, game over on death
    Casual,      // Minimal penalties, unlimited revivals
}

impl GameMode {
    pub fn name(&self) -> &'static str {
        match self {
            GameMode::Normal => "Normal",
            GameMode::Hardcore => "Hardcore",
            GameMode::Permadeath => "Permadeath",
            GameMode::Casual => "Casual",
        }
    }
    
    pub fn description(&self) -> &'static str {
        match self {
            GameMode::Normal => "Standard difficulty with death penalties and limited revivals",
            GameMode::Hardcore => "High difficulty with harsh death penalties and very limited revivals",
            GameMode::Permadeath => "Ultimate challenge - one life only, game over on death",
            GameMode::Casual => "Relaxed mode with minimal penalties and unlimited revivals",
        }
    }
    
    pub fn max_revivals(&self) -> i32 {
        match self {
            GameMode::Normal => 3,
            GameMode::Hardcore => 1,
            GameMode::Permadeath => 0,
            GameMode::Casual => -1, // Unlimited
        }
    }
    
    pub fn death_penalty(&self) -> DeathPenalty {
        match self {
            GameMode::Normal => DeathPenalty::new(),
            GameMode::Hardcore => DeathPenalty {
                experience_loss_percentage: 25.0,
                attribute_penalty: 2,
                equipment_durability_loss: 10,
                temporary_stat_reduction: 5,
                penalty_duration: 100,
            },
            GameMode::Permadeath => DeathPenalty::permadeath(),
            GameMode::Casual => DeathPenalty {
                experience_loss_percentage: 5.0,
                attribute_penalty: 0,
                equipment_durability_loss: 2,
                temporary_stat_reduction: 1,
                penalty_duration: 20,
            },
        }
    }
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
#[storage(VecStorage)]
pub struct GameSettings {
    pub game_mode: GameMode,
    pub permadeath_enabled: bool,
    pub auto_save_on_death: bool,
    pub death_screen_enabled: bool,
}

impl GameSettings {
    pub fn new(mode: GameMode) -> Self {
        GameSettings {
            permadeath_enabled: mode == GameMode::Permadeath,
            game_mode: mode,
            auto_save_on_death: true,
            death_screen_enabled: true,
        }
    }
}

// Enhanced combat components
#[derive(Component, Debug, Serialize, Deserialize, Clone)]
#[storage(VecStorage)]
pub struct Attacker {
    pub attack_bonus: i32,
    pub critical_chance: f32,
    pub critical_multiplier: f32,
    pub attack_speed: i32,
    pub last_attack_turn: i32,
}

impl Attacker {
    pub fn new() -> Self {
        Attacker {
            attack_bonus: 0,
            critical_chance: 0.05, // 5% base crit chance
            critical_multiplier: 2.0, // 2x damage on crit
            attack_speed: 100, // Base attack speed (lower is faster)
            last_attack_turn: 0,
        }
    }
    
    pub fn can_attack(&self, current_turn: i32) -> bool {
        current_turn - self.last_attack_turn >= self.attack_speed / 100
    }
    
    pub fn is_critical_hit(&self, rng: &mut crate::resources::RandomNumberGenerator) -> bool {
        rng.roll_dice(1, 100) as f32 <= self.critical_chance * 100.0
    }
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
#[storage(VecStorage)]
pub struct Defender {
    pub armor_class: i32,
    pub damage_reduction: i32,
    pub evasion_chance: f32,
    pub block_chance: f32,
    pub parry_chance: f32,
}

impl Defender {
    pub fn new() -> Self {
        Defender {
            armor_class: 10, // Base AC
            damage_reduction: 0,
            evasion_chance: 0.05, // 5% base evasion
            block_chance: 0.0, // Requires shield
            parry_chance: 0.0, // Requires weapon
        }
    }
    
    pub fn calculate_defense(&self, rng: &mut crate::resources::RandomNumberGenerator) -> DefenseResult {
        let roll = rng.roll_dice(1, 100) as f32;
        
        if roll <= self.evasion_chance * 100.0 {
            DefenseResult::Evaded
        } else if roll <= (self.evasion_chance + self.block_chance) * 100.0 {
            DefenseResult::Blocked
        } else if roll <= (self.evasion_chance + self.block_chance + self.parry_chance) * 100.0 {
            DefenseResult::Parried
        } else {
            DefenseResult::Hit
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum DefenseResult {
    Hit,
    Evaded,
    Blocked,
    Parried,
}

// Enhanced damage system
#[derive(Component, Debug, Serialize, Deserialize, Clone)]
#[storage(VecStorage)]
pub struct DamageInfo {
    pub base_damage: i32,
    pub damage_type: DamageType,
    pub source: Entity,
    pub is_critical: bool,
    pub penetration: i32, // Armor penetration
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
pub enum DamageType {
    Physical,
    Fire,
    Ice,
    Lightning,
    Poison,
    Holy,
    Dark,
    Psychic,
}

impl DamageType {
    pub fn name(&self) -> &'static str {
        match self {
            DamageType::Physical => "Physical",
            DamageType::Fire => "Fire",
            DamageType::Ice => "Ice",
            DamageType::Lightning => "Lightning",
            DamageType::Poison => "Poison",
            DamageType::Holy => "Holy",
            DamageType::Dark => "Dark",
            DamageType::Psychic => "Psychic",
        }
    }
}

// Damage resistance system
#[derive(Component, Debug, Serialize, Deserialize, Clone)]
#[storage(VecStorage)]
pub struct DamageResistances {
    pub resistances: std::collections::HashMap<DamageType, f32>,
}

impl DamageResistances {
    pub fn new() -> Self {
        DamageResistances {
            resistances: std::collections::HashMap::new(),
        }
    }
    
    pub fn add_resistance(&mut self, damage_type: DamageType, resistance: f32) {
        self.resistances.insert(damage_type, resistance.clamp(0.0, 1.0));
    }
    
    pub fn get_resistance(&self, damage_type: DamageType) -> f32 {
        *self.resistances.get(&damage_type).unwrap_or(&0.0)
    }
    
    pub fn calculate_damage(&self, base_damage: i32, damage_type: DamageType) -> i32 {
        let resistance = self.get_resistance(damage_type);
        ((base_damage as f32) * (1.0 - resistance)) as i32
    }
}

// Combat action queue
#[derive(Component, Debug, Serialize, Deserialize, Clone)]
#[storage(VecStorage)]
pub struct CombatAction {
    pub action_type: CombatActionType,
    pub actor: Entity,
    pub target: Option<Entity>,
    pub priority: i32,
    pub delay: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum CombatActionType {
    Attack,
    Defend,
    UseItem,
    UseAbility(AbilityType),
    Move,
    Wait,
}

// Combat turn order system
#[derive(Component, Debug, Serialize, Deserialize, Clone)]
#[storage(VecStorage)]
pub struct Initiative {
    pub base_initiative: i32,
    pub current_initiative: i32,
    pub has_acted: bool,
}

impl Initiative {
    pub fn new(base: i32) -> Self {
        Initiative {
            base_initiative: base,
            current_initiative: base,
            has_acted: false,
        }
    }
    
    pub fn roll_initiative(&mut self, rng: &mut crate::resources::RandomNumberGenerator) {
        self.current_initiative = self.base_initiative + rng.roll_dice(1, 20);
        self.has_acted = false;
    }
}

// Combat feedback components
#[derive(Component, Debug, Clone)]
#[storage(VecStorage)]
pub struct CombatFeedback {
    pub feedback_type: CombatFeedbackType,
    pub position: FloatingPosition,
    pub duration: f32,
    pub max_duration: f32,
    pub color: crossterm::style::Color,
    pub animation_type: AnimationType,
}

#[derive(Debug, Clone)]
pub enum CombatFeedbackType {
    DamageText {
        damage: i32,
        damage_type: DamageType,
        is_critical: bool,
    },
    HealingText {
        healing: i32,
    },
    StatusText {
        text: String,
    },
    ScreenShake {
        intensity: ShakeIntensity,
    },
    SoundEffect {
        sound_type: SoundEffectType,
    },
}

#[derive(Debug, Clone)]
pub struct FloatingPosition {
    pub x: f32,
    pub y: f32,
    pub offset_x: f32,
    pub offset_y: f32,
}

impl Default for FloatingPosition {
    fn default() -> Self {
        FloatingPosition {
            x: 0.0,
            y: 0.0,
            offset_x: 0.0,
            offset_y: 0.0,
        }
    }
}

#[derive(Debug, Clone)]
pub enum AnimationType {
    FloatUp,
    CriticalBounce,
    Shake,
    Flash,
    Pulse,
}

#[derive(Debug, Clone)]
pub enum ShakeIntensity {
    Light,
    Medium,
    Heavy,
}

impl ShakeIntensity {
    pub fn get_offset(&self) -> f32 {
        match self {
            ShakeIntensity::Light => 1.0,
            ShakeIntensity::Medium => 2.0,
            ShakeIntensity::Heavy => 4.0,
        }
    }
}

#[derive(Debug, Clone)]
pub enum SoundEffectType {
    Hit,
    CriticalHit,
    Block,
    Evade,
    Death,
    Heal,
    StatusEffect,
}

impl SoundEffectType {
    pub fn get_sound_file(&self) -> &'static str {
        match self {
            SoundEffectType::Hit => "sounds/hit.wav",
            SoundEffectType::CriticalHit => "sounds/critical.wav",
            SoundEffectType::Block => "sounds/block.wav",
            SoundEffectType::Evade => "sounds/evade.wav",
            SoundEffectType::Death => "sounds/death.wav",
            SoundEffectType::Heal => "sounds/heal.wav",
            SoundEffectType::StatusEffect => "sounds/status.wav",
        }
    }
}

// Visual effects components
#[derive(Component, Debug, Clone)]
#[storage(VecStorage)]
pub struct ParticleEffect {
    pub position: FloatingPosition,
    pub velocity: ParticleVelocity,
    pub color: crossterm::style::Color,
    pub character: char,
    pub lifetime: f32,
    pub max_lifetime: f32,
}

#[derive(Debug, Clone)]
pub struct ParticleVelocity {
    pub x: f32,
    pub y: f32,
}

// Combat rewards components
#[derive(Component, Debug, Serialize, Deserialize, Clone)]
#[storage(VecStorage)]
pub struct LootTable {
    pub entries: Vec<LootTableEntry>,
}

impl LootTable {
    pub fn new() -> Self {
        LootTable {
            entries: Vec::new(),
        }
    }
    
    pub fn add_entry(&mut self, loot_drop: LootDrop, chance: i32) {
        self.entries.push(LootTableEntry {
            loot_drop,
            chance,
        });
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LootTableEntry {
    pub loot_drop: LootDrop,
    pub chance: i32, // Percentage chance (1-100)
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum LootDrop {
    Equipment {
        name: String,
        slot: EquipmentSlot,
        power_bonus: i32,
        defense_bonus: i32,
    },
    Consumable {
        name: String,
        healing: i32,
    },
    Currency {
        amount: i32,
    },
}

#[derive(Component, Debug, Serialize, Deserialize, Clone, Default)]
#[storage(NullStorage)]
pub struct UniqueEnemy;

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
#[storage(VecStorage)]
pub struct CombatReward {
    pub source_entity: Entity,
    pub source_name: String,
    pub experience_gained: i32,
    pub loot_generated: bool,
    pub special_drops: bool,
}

// Boss enemy component for special rewards
#[derive(Component, Debug, Serialize, Deserialize, Clone)]
#[storage(VecStorage)]
pub struct BossEnemy {
    pub boss_type: BossType,
    pub difficulty_multiplier: f32,
    pub guaranteed_drops: Vec<LootDrop>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum BossType {
    MiniBoss,
    AreaBoss,
    FinalBoss,
}

impl BossType {
    pub fn experience_multiplier(&self) -> f32 {
        match self {
            BossType::MiniBoss => 2.0,
            BossType::AreaBoss => 3.5,
            BossType::FinalBoss => 5.0,
        }
    }
    
    pub fn loot_multiplier(&self) -> f32 {
        match self {
            BossType::MiniBoss => 1.5,
            BossType::AreaBoss => 2.0,
            BossType::FinalBoss => 3.0,
        }
    }
}

// Treasure component for special loot containers
#[derive(Component, Debug, Serialize, Deserialize, Clone)]
#[storage(VecStorage)]
pub struct Treasure {
    pub treasure_type: TreasureType,
    pub loot_table: LootTable,
    pub is_opened: bool,
    pub requires_key: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum TreasureType {
    Chest,
    Barrel,
    Urn,
    Corpse,
    SecretCache,
}

impl TreasureType {
    pub fn get_glyph(&self) -> char {
        match self {
            TreasureType::Chest => '&',
            TreasureType::Barrel => 'O',
            TreasureType::Urn => 'U',
            TreasureType::Corpse => '%',
            TreasureType::SecretCache => '*',
        }
    }
    
    pub fn get_color(&self) -> crossterm::style::Color {
        match self {
            TreasureType::Chest => crossterm::style::Color::Yellow,
            TreasureType::Barrel => crossterm::style::Color::DarkYellow,
            TreasureType::Urn => crossterm::style::Color::Grey,
            TreasureType::Corpse => crossterm::style::Color::DarkRed,
            TreasureType::SecretCache => crossterm::style::Color::Magenta,
        }
    }
}

// Interaction component
#[derive(Component, Debug, Clone)]
#[storage(VecStorage)]
pub struct WantsToInteract {
    pub target: Entity,
}