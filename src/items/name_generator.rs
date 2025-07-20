use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use crate::items::{ItemType, ItemRarity, WeaponType, ArmorType, ConsumableType, MaterialType};
use crate::resources::RandomNumberGenerator;

/// Procedural name generator for items
pub struct ItemNameGenerator {
    pub weapon_bases: HashMap<WeaponType, Vec<String>>,
    pub armor_bases: HashMap<ArmorType, Vec<String>>,
    pub consumable_bases: HashMap<ConsumableType, Vec<String>>,
    pub material_bases: HashMap<MaterialType, Vec<String>>,
    pub prefixes: Vec<NameAffix>,
    pub suffixes: Vec<NameAffix>,
    pub legendary_names: Vec<String>,
    pub artifact_names: Vec<String>,
}

impl ItemNameGenerator {
    pub fn new() -> Self {
        let mut generator = ItemNameGenerator {
            weapon_bases: HashMap::new(),
            armor_bases: HashMap::new(),
            consumable_bases: HashMap::new(),
            material_bases: HashMap::new(),
            prefixes: Vec::new(),
            suffixes: Vec::new(),
            legendary_names: Vec::new(),
            artifact_names: Vec::new(),
        };
        
        generator.initialize_base_names();
        generator.initialize_affixes();
        generator.initialize_unique_names();
        
        generator
    }

    /// Generate a name for an item based on its properties
    pub fn generate_name(
        &self,
        item_type: &ItemType,
        rarity: &ItemRarity,
        has_enchantments: bool,
        rng: &mut RandomNumberGenerator,
    ) -> String {
        match rarity {
            ItemRarity::Artifact => {
                self.generate_artifact_name(rng)
            },
            ItemRarity::Legendary => {
                self.generate_legendary_name(item_type, rng)
            },
            ItemRarity::Epic | ItemRarity::Rare => {
                self.generate_magical_name(item_type, has_enchantments, rng)
            },
            ItemRarity::Uncommon => {
                if has_enchantments || rng.roll_dice(1, 3) == 1 {
                    self.generate_magical_name(item_type, has_enchantments, rng)
                } else {
                    self.generate_quality_name(item_type, rng)
                }
            },
            _ => {
                self.generate_basic_name(item_type, rng)
            }
        }
    }

    fn generate_basic_name(&self, item_type: &ItemType, rng: &mut RandomNumberGenerator) -> String {
        let base_name = self.get_base_name(item_type, rng);
        
        // Sometimes add a simple quality descriptor
        if rng.roll_dice(1, 4) == 1 {
            let qualities = vec!["Old", "Worn", "Simple", "Basic", "Common"];
            let quality = &qualities[rng.roll_dice(1, qualities.len()) - 1];
            format!("{} {}", quality, base_name)
        } else {
            base_name
        }
    }

    fn generate_quality_name(&self, item_type: &ItemType, rng: &mut RandomNumberGenerator) -> String {
        let base_name = self.get_base_name(item_type, rng);
        let qualities = vec![
            "Fine", "Well-made", "Sturdy", "Reliable", "Quality",
            "Masterwork", "Superior", "Excellent", "Refined"
        ];
        let quality = &qualities[rng.roll_dice(1, qualities.len()) - 1];
        format!("{} {}", quality, base_name)
    }

    fn generate_magical_name(&self, item_type: &ItemType, has_enchantments: bool, rng: &mut RandomNumberGenerator) -> String {
        let base_name = self.get_base_name(item_type, rng);
        let mut name = base_name;
        
        // Add prefix
        if rng.roll_dice(1, 2) == 1 {
            if let Some(prefix) = self.get_random_prefix(item_type, rng) {
                name = format!("{} {}", prefix.name, name);
            }
        }
        
        // Add suffix
        if rng.roll_dice(1, 2) == 1 {
            if let Some(suffix) = self.get_random_suffix(item_type, rng) {
                name = format!("{} {}", name, suffix.name);
            }
        }
        
        // If no affixes were added and it has enchantments, force one
        if name == base_name && has_enchantments {
            if rng.roll_dice(1, 2) == 1 {
                if let Some(prefix) = self.get_random_prefix(item_type, rng) {
                    name = format!("{} {}", prefix.name, name);
                }
            } else {
                if let Some(suffix) = self.get_random_suffix(item_type, rng) {
                    name = format!("{} {}", name, suffix.name);
                }
            }
        }
        
        name
    }

    fn generate_legendary_name(&self, item_type: &ItemType, rng: &mut RandomNumberGenerator) -> String {
        // Use predefined legendary names or generate epic ones
        if rng.roll_dice(1, 3) == 1 && !self.legendary_names.is_empty() {
            let legendary_name = &self.legendary_names[rng.roll_dice(1, self.legendary_names.len()) - 1];
            legendary_name.clone()
        } else {
            let base_name = self.get_base_name(item_type, rng);
            let legendary_prefixes = vec![
                "Legendary", "Fabled", "Mythical", "Ancient", "Divine",
                "Celestial", "Infernal", "Eternal", "Sacred", "Cursed"
            ];
            let prefix = &legendary_prefixes[rng.roll_dice(1, legendary_prefixes.len()) - 1];
            format!("{} {}", prefix, base_name)
        }
    }

    fn generate_artifact_name(&self, rng: &mut RandomNumberGenerator) -> String {
        if !self.artifact_names.is_empty() {
            let artifact_name = &self.artifact_names[rng.roll_dice(1, self.artifact_names.len()) - 1];
            artifact_name.clone()
        } else {
            // Fallback to generated name
            let titles = vec![
                "The Worldbreaker", "The Eternal Flame", "The Void Walker",
                "The Star Forge", "The Time Render", "The Soul Keeper",
                "The Dream Weaver", "The Reality Shaper", "The Fate Binder"
            ];
            let title = &titles[rng.roll_dice(1, titles.len()) - 1];
            title.clone()
        }
    }

    fn get_base_name(&self, item_type: &ItemType, rng: &mut RandomNumberGenerator) -> String {
        match item_type {
            ItemType::Weapon(weapon_type) => {
                if let Some(names) = self.weapon_bases.get(weapon_type) {
                    names[rng.roll_dice(1, names.len()) - 1].clone()
                } else {
                    format!("{:?}", weapon_type)
                }
            },
            ItemType::Armor(armor_type) => {
                if let Some(names) = self.armor_bases.get(armor_type) {
                    names[rng.roll_dice(1, names.len()) - 1].clone()
                } else {
                    format!("{:?}", armor_type)
                }
            },
            ItemType::Consumable(consumable_type) => {
                if let Some(names) = self.consumable_bases.get(consumable_type) {
                    names[rng.roll_dice(1, names.len()) - 1].clone()
                } else {
                    format!("{:?}", consumable_type)
                }
            },
            ItemType::Material(material_type) => {
                if let Some(names) = self.material_bases.get(material_type) {
                    names[rng.roll_dice(1, names.len()) - 1].clone()
                } else {
                    format!("{:?}", material_type)
                }
            },
            _ => "Unknown Item".to_string(),
        }
    }

    fn get_random_prefix(&self, item_type: &ItemType, rng: &mut RandomNumberGenerator) -> Option<&NameAffix> {
        let applicable_prefixes: Vec<&NameAffix> = self.prefixes.iter()
            .filter(|affix| affix.applies_to_type(item_type))
            .collect();
        
        if applicable_prefixes.is_empty() {
            None
        } else {
            Some(applicable_prefixes[rng.roll_dice(1, applicable_prefixes.len()) - 1])
        }
    }

    fn get_random_suffix(&self, item_type: &ItemType, rng: &mut RandomNumberGenerator) -> Option<&NameAffix> {
        let applicable_suffixes: Vec<&NameAffix> = self.suffixes.iter()
            .filter(|affix| affix.applies_to_type(item_type))
            .collect();
        
        if applicable_suffixes.is_empty() {
            None
        } else {
            Some(applicable_suffixes[rng.roll_dice(1, applicable_suffixes.len()) - 1])
        }
    }

    fn initialize_base_names(&mut self) {
        // Weapon base names
        self.weapon_bases.insert(WeaponType::Sword, vec![
            "Sword".to_string(), "Blade".to_string(), "Saber".to_string(),
            "Rapier".to_string(), "Scimitar".to_string(), "Longsword".to_string(),
            "Broadsword".to_string(), "Claymore".to_string()
        ]);
        
        self.weapon_bases.insert(WeaponType::Axe, vec![
            "Axe".to_string(), "Hatchet".to_string(), "Battleaxe".to_string(),
            "War Axe".to_string(), "Cleaver".to_string(), "Tomahawk".to_string()
        ]);
        
        self.weapon_bases.insert(WeaponType::Mace, vec![
            "Mace".to_string(), "Club".to_string(), "Hammer".to_string(),
            "War Hammer".to_string(), "Flail".to_string(), "Morningstar".to_string()
        ]);
        
        self.weapon_bases.insert(WeaponType::Dagger, vec![
            "Dagger".to_string(), "Knife".to_string(), "Stiletto".to_string(),
            "Dirk".to_string(), "Shiv".to_string(), "Blade".to_string()
        ]);
        
        self.weapon_bases.insert(WeaponType::Spear, vec![
            "Spear".to_string(), "Lance".to_string(), "Pike".to_string(),
            "Javelin".to_string(), "Halberd".to_string(), "Trident".to_string()
        ]);
        
        self.weapon_bases.insert(WeaponType::Bow, vec![
            "Bow".to_string(), "Longbow".to_string(), "Shortbow".to_string(),
            "Composite Bow".to_string(), "Recurve Bow".to_string()
        ]);
        
        self.weapon_bases.insert(WeaponType::Staff, vec![
            "Staff".to_string(), "Rod".to_string(), "Scepter".to_string(),
            "Quarterstaff".to_string(), "Walking Stick".to_string()
        ]);
        
        self.weapon_bases.insert(WeaponType::Wand, vec![
            "Wand".to_string(), "Rod".to_string(), "Stick".to_string(),
            "Branch".to_string(), "Twig".to_string()
        ]);

        // Armor base names
        self.armor_bases.insert(ArmorType::Helmet, vec![
            "Helmet".to_string(), "Helm".to_string(), "Cap".to_string(),
            "Coif".to_string(), "Crown".to_string(), "Circlet".to_string()
        ]);
        
        self.armor_bases.insert(ArmorType::Chest, vec![
            "Armor".to_string(), "Chestplate".to_string(), "Breastplate".to_string(),
            "Mail".to_string(), "Vest".to_string(), "Tunic".to_string(),
            "Robe".to_string(), "Jacket".to_string()
        ]);
        
        self.armor_bases.insert(ArmorType::Legs, vec![
            "Leggings".to_string(), "Greaves".to_string(), "Pants".to_string(),
            "Trousers".to_string(), "Chaps".to_string()
        ]);
        
        self.armor_bases.insert(ArmorType::Boots, vec![
            "Boots".to_string(), "Shoes".to_string(), "Sandals".to_string(),
            "Slippers".to_string(), "Sabatons".to_string()
        ]);
        
        self.armor_bases.insert(ArmorType::Gloves, vec![
            "Gloves".to_string(), "Gauntlets".to_string(), "Mittens".to_string(),
            "Bracers".to_string(), "Vambraces".to_string()
        ]);
        
        self.armor_bases.insert(ArmorType::Shield, vec![
            "Shield".to_string(), "Buckler".to_string(), "Targe".to_string(),
            "Kite Shield".to_string(), "Tower Shield".to_string()
        ]);
        
        self.armor_bases.insert(ArmorType::Ring, vec![
            "Ring".to_string(), "Band".to_string(), "Circle".to_string(),
            "Loop".to_string(), "Signet".to_string()
        ]);
        
        self.armor_bases.insert(ArmorType::Amulet, vec![
            "Amulet".to_string(), "Pendant".to_string(), "Necklace".to_string(),
            "Charm".to_string(), "Talisman".to_string(), "Medallion".to_string()
        ]);

        // Consumable base names
        self.consumable_bases.insert(ConsumableType::Potion, vec![
            "Potion".to_string(), "Elixir".to_string(), "Draught".to_string(),
            "Brew".to_string(), "Tonic".to_string(), "Philter".to_string()
        ]);
        
        self.consumable_bases.insert(ConsumableType::Food, vec![
            "Bread".to_string(), "Rations".to_string(), "Jerky".to_string(),
            "Cheese".to_string(), "Apple".to_string(), "Meat".to_string()
        ]);
        
        self.consumable_bases.insert(ConsumableType::Scroll, vec![
            "Scroll".to_string(), "Parchment".to_string(), "Tome".to_string(),
            "Manuscript".to_string(), "Document".to_string()
        ]);

        // Material base names
        self.material_bases.insert(MaterialType::Metal, vec![
            "Iron Ore".to_string(), "Steel Ingot".to_string(), "Copper".to_string(),
            "Silver".to_string(), "Gold".to_string(), "Mithril".to_string()
        ]);
        
        self.material_bases.insert(MaterialType::Gem, vec![
            "Ruby".to_string(), "Sapphire".to_string(), "Emerald".to_string(),
            "Diamond".to_string(), "Amethyst".to_string(), "Topaz".to_string()
        ]);
        
        self.material_bases.insert(MaterialType::Herb, vec![
            "Herb".to_string(), "Flower".to_string(), "Root".to_string(),
            "Leaf".to_string(), "Mushroom".to_string(), "Moss".to_string()
        ]);
    }

    fn initialize_affixes(&mut self) {
        // Prefixes
        self.prefixes.extend(vec![
            NameAffix {
                name: "Flaming".to_string(),
                applicable_types: vec![AffixApplicability::Weapons],
                weight: 20,
            },
            NameAffix {
                name: "Frozen".to_string(),
                applicable_types: vec![AffixApplicability::Weapons],
                weight: 20,
            },
            NameAffix {
                name: "Shocking".to_string(),
                applicable_types: vec![AffixApplicability::Weapons],
                weight: 20,
            },
            NameAffix {
                name: "Venomous".to_string(),
                applicable_types: vec![AffixApplicability::Weapons],
                weight: 15,
            },
            NameAffix {
                name: "Blessed".to_string(),
                applicable_types: vec![AffixApplicability::All],
                weight: 10,
            },
            NameAffix {
                name: "Cursed".to_string(),
                applicable_types: vec![AffixApplicability::All],
                weight: 5,
            },
            NameAffix {
                name: "Enchanted".to_string(),
                applicable_types: vec![AffixApplicability::All],
                weight: 15,
            },
            NameAffix {
                name: "Glowing".to_string(),
                applicable_types: vec![AffixApplicability::All],
                weight: 12,
            },
            NameAffix {
                name: "Ancient".to_string(),
                applicable_types: vec![AffixApplicability::All],
                weight: 8,
            },
            NameAffix {
                name: "Runic".to_string(),
                applicable_types: vec![AffixApplicability::All],
                weight: 10,
            },
        ]);

        // Suffixes
        self.suffixes.extend(vec![
            NameAffix {
                name: "of Power".to_string(),
                applicable_types: vec![AffixApplicability::Weapons],
                weight: 20,
            },
            NameAffix {
                name: "of Strength".to_string(),
                applicable_types: vec![AffixApplicability::All],
                weight: 18,
            },
            NameAffix {
                name: "of Agility".to_string(),
                applicable_types: vec![AffixApplicability::All],
                weight: 18,
            },
            NameAffix {
                name: "of Protection".to_string(),
                applicable_types: vec![AffixApplicability::Armor],
                weight: 20,
            },
            NameAffix {
                name: "of the Eagle".to_string(),
                applicable_types: vec![AffixApplicability::All],
                weight: 12,
            },
            NameAffix {
                name: "of the Bear".to_string(),
                applicable_types: vec![AffixApplicability::All],
                weight: 12,
            },
            NameAffix {
                name: "of the Wolf".to_string(),
                applicable_types: vec![AffixApplicability::All],
                weight: 12,
            },
            NameAffix {
                name: "of Slaying".to_string(),
                applicable_types: vec![AffixApplicability::Weapons],
                weight: 15,
            },
            NameAffix {
                name: "of Warding".to_string(),
                applicable_types: vec![AffixApplicability::Armor],
                weight: 15,
            },
            NameAffix {
                name: "of the Ancients".to_string(),
                applicable_types: vec![AffixApplicability::All],
                weight: 8,
            },
        ]);
    }

    fn initialize_unique_names(&mut self) {
        self.legendary_names.extend(vec![
            "Excalibur".to_string(),
            "Mjolnir".to_string(),
            "Durandal".to_string(),
            "Gram".to_string(),
            "Balmung".to_string(),
            "Tyrfing".to_string(),
            "Curtana".to_string(),
            "Joyeuse".to_string(),
            "Caladbolg".to_string(),
            "Galatine".to_string(),
        ]);

        self.artifact_names.extend(vec![
            "The Worldrender".to_string(),
            "Eternity's Edge".to_string(),
            "Voidcaller".to_string(),
            "Starfall".to_string(),
            "The Dreambane".to_string(),
            "Soulreaper".to_string(),
            "The Timeless Crown".to_string(),
            "Heart of the Mountain".to_string(),
            "The Infinite Codex".to_string(),
            "Whisper of the Void".to_string(),
        ]);
    }

    /// Save name generator data to file
    pub fn save_to_file(&self, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
        let data = NameGeneratorData {
            weapon_bases: self.weapon_bases.clone(),
            armor_bases: self.armor_bases.clone(),
            consumable_bases: self.consumable_bases.clone(),
            material_bases: self.material_bases.clone(),
            prefixes: self.prefixes.clone(),
            suffixes: self.suffixes.clone(),
            legendary_names: self.legendary_names.clone(),
            artifact_names: self.artifact_names.clone(),
        };
        
        let json = serde_json::to_string_pretty(&data)?;
        std::fs::write(filename, json)?;
        Ok(())
    }

    /// Load name generator data from file
    pub fn load_from_file(filename: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let json = std::fs::read_to_string(filename)?;
        let data: NameGeneratorData = serde_json::from_str(&json)?;
        
        Ok(ItemNameGenerator {
            weapon_bases: data.weapon_bases,
            armor_bases: data.armor_bases,
            consumable_bases: data.consumable_bases,
            material_bases: data.material_bases,
            prefixes: data.prefixes,
            suffixes: data.suffixes,
            legendary_names: data.legendary_names,
            artifact_names: data.artifact_names,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NameAffix {
    pub name: String,
    pub applicable_types: Vec<AffixApplicability>,
    pub weight: i32,
}

impl NameAffix {
    pub fn applies_to_type(&self, item_type: &ItemType) -> bool {
        for applicability in &self.applicable_types {
            match applicability {
                AffixApplicability::All => return true,
                AffixApplicability::Weapons => {
                    if matches!(item_type, ItemType::Weapon(_)) {
                        return true;
                    }
                },
                AffixApplicability::Armor => {
                    if matches!(item_type, ItemType::Armor(_)) {
                        return true;
                    }
                },
                AffixApplicability::Consumables => {
                    if matches!(item_type, ItemType::Consumable(_)) {
                        return true;
                    }
                },
            }
        }
        false
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AffixApplicability {
    All,
    Weapons,
    Armor,
    Consumables,
}

#[derive(Serialize, Deserialize)]
struct NameGeneratorData {
    weapon_bases: HashMap<WeaponType, Vec<String>>,
    armor_bases: HashMap<ArmorType, Vec<String>>,
    consumable_bases: HashMap<ConsumableType, Vec<String>>,
    material_bases: HashMap<MaterialType, Vec<String>>,
    prefixes: Vec<NameAffix>,
    suffixes: Vec<NameAffix>,
    legendary_names: Vec<String>,
    artifact_names: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_name_generator_creation() {
        let generator = ItemNameGenerator::new();
        assert!(!generator.weapon_bases.is_empty());
        assert!(!generator.armor_bases.is_empty());
        assert!(!generator.prefixes.is_empty());
        assert!(!generator.suffixes.is_empty());
    }

    #[test]
    fn test_basic_name_generation() {
        let generator = ItemNameGenerator::new();
        let mut rng = RandomNumberGenerator::new();
        
        let name = generator.generate_name(
            &ItemType::Weapon(WeaponType::Sword),
            &ItemRarity::Common,
            false,
            &mut rng
        );
        
        assert!(!name.is_empty());
    }

    #[test]
    fn test_magical_name_generation() {
        let generator = ItemNameGenerator::new();
        let mut rng = RandomNumberGenerator::new();
        
        let name = generator.generate_name(
            &ItemType::Weapon(WeaponType::Sword),
            &ItemRarity::Rare,
            true,
            &mut rng
        );
        
        assert!(!name.is_empty());
        // Magical items should have more complex names
        assert!(name.split_whitespace().count() >= 2);
    }

    #[test]
    fn test_legendary_name_generation() {
        let generator = ItemNameGenerator::new();
        let mut rng = RandomNumberGenerator::new();
        
        let name = generator.generate_name(
            &ItemType::Weapon(WeaponType::Sword),
            &ItemRarity::Legendary,
            true,
            &mut rng
        );
        
        assert!(!name.is_empty());
    }

    #[test]
    fn test_artifact_name_generation() {
        let generator = ItemNameGenerator::new();
        let mut rng = RandomNumberGenerator::new();
        
        let name = generator.generate_name(
            &ItemType::Weapon(WeaponType::Sword),
            &ItemRarity::Artifact,
            true,
            &mut rng
        );
        
        assert!(!name.is_empty());
    }

    #[test]
    fn test_affix_applicability() {
        let weapon_affix = NameAffix {
            name: "Test".to_string(),
            applicable_types: vec![AffixApplicability::Weapons],
            weight: 10,
        };
        
        assert!(weapon_affix.applies_to_type(&ItemType::Weapon(WeaponType::Sword)));
        assert!(!weapon_affix.applies_to_type(&ItemType::Armor(ArmorType::Helmet)));
        
        let all_affix = NameAffix {
            name: "Test".to_string(),
            applicable_types: vec![AffixApplicability::All],
            weight: 10,
        };
        
        assert!(all_affix.applies_to_type(&ItemType::Weapon(WeaponType::Sword)));
        assert!(all_affix.applies_to_type(&ItemType::Armor(ArmorType::Helmet)));
    }
}