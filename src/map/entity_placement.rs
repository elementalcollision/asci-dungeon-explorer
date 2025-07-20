use rand::Rng;
use crate::map::{Map, TileType, MapTheme};
use crate::resources::RandomNumberGenerator;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum EnemyType {
    Goblin,
    Orc,
    Troll,
    Skeleton,
    Zombie,
    Ghost,
    Demon,
    Dragon,
    Spider,
    Bat,
    Rat,
    Snake,
    Slime,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ItemType {
    HealthPotion,
    ManaPotion,
    Scroll,
    Weapon,
    Armor,
    Shield,
    Ring,
    Amulet,
    Gold,
    Key,
    Gem,
}

pub struct EntityPlacementSystem {
    pub rng: RandomNumberGenerator,
}

impl EntityPlacementSystem {
    pub fn new(rng: RandomNumberGenerator) -> Self {
        EntityPlacementSystem { rng }
    }
    
    /// Place entities in the map based on difficulty and theme
    pub fn populate_map(&mut self, map: &Map, difficulty: i32) -> Vec<EntitySpawn> {
        let mut spawns = Vec::new();
        
        // Place enemies
        self.place_enemies(&mut spawns, map, difficulty);
        
        // Place items
        self.place_items(&mut spawns, map, difficulty);
        
        // Place special features
        self.place_special_features(&mut spawns, map, difficulty);
        
        spawns
    }
    
    fn place_enemies(&mut self, spawns: &mut Vec<EntitySpawn>, map: &Map, difficulty: i32) {
        // Calculate number of enemies based on map size and difficulty
        let map_area = map.width * map.height;
        let base_enemies = (map_area as f32 * 0.01) as i32; // 1% of map area
        let enemy_count = base_enemies + (difficulty / 2);
        
        // Place enemies
        for _ in 0..enemy_count {
            if let Some(pos) = self.find_valid_spawn_position(map) {
                let enemy_type = self.choose_enemy_type(map, difficulty, pos);
                spawns.push(EntitySpawn {
                    entity_type: SpawnType::Enemy(enemy_type),
                    x: pos.0,
                    y: pos.1,
                });
            }
        }
    }
    
    fn place_items(&mut self, spawns: &mut Vec<EntitySpawn>, map: &Map, difficulty: i32) {
        // Calculate number of items based on map size and difficulty
        let map_area = map.width * map.height;
        let base_items = (map_area as f32 * 0.005) as i32; // 0.5% of map area
        let item_count = base_items + (difficulty / 3);
        
        // Place items
        for _ in 0..item_count {
            if let Some(pos) = self.find_valid_spawn_position(map) {
                let item_type = self.choose_item_type(map, difficulty);
                spawns.push(EntitySpawn {
                    entity_type: SpawnType::Item(item_type),
                    x: pos.0,
                    y: pos.1,
                });
            }
        }
    }
    
    fn place_special_features(&mut self, spawns: &mut Vec<EntitySpawn>, map: &Map, difficulty: i32) {
        // Place special features like chests, shrines, etc.
        let special_count = 1 + (difficulty / 5);
        
        for _ in 0..special_count {
            if let Some(pos) = self.find_valid_spawn_position(map) {
                let feature_type = self.choose_special_feature(map, difficulty);
                spawns.push(EntitySpawn {
                    entity_type: SpawnType::Special(feature_type),
                    x: pos.0,
                    y: pos.1,
                });
            }
        }
    }
    
    fn find_valid_spawn_position(&mut self, map: &Map) -> Option<(i32, i32)> {
        // Try to find a valid position for spawning
        let mut attempts = 0;
        while attempts < 100 {
            let x = self.rng.range(1, map.width - 1);
            let y = self.rng.range(1, map.height - 1);
            
            let idx = map.xy_idx(x, y);
            
            // Check if the position is valid for spawning
            if map.tiles[idx] == TileType::Floor && !self.is_near_stairs(map, x, y) {
                return Some((x, y));
            }
            
            attempts += 1;
        }
        
        None
    }
    
    fn is_near_stairs(&self, map: &Map, x: i32, y: i32) -> bool {
        // Check if the position is near stairs (to avoid blocking exits)
        let stairs_distance = 3;
        
        let dx_up = (x - map.entrance.0).abs();
        let dy_up = (y - map.entrance.1).abs();
        let dx_down = (x - map.exit.0).abs();
        let dy_down = (y - map.exit.1).abs();
        
        (dx_up <= stairs_distance && dy_up <= stairs_distance) || 
        (dx_down <= stairs_distance && dy_down <= stairs_distance)
    }
    
    fn choose_enemy_type(&mut self, map: &Map, difficulty: i32, pos: (i32, i32)) -> EnemyType {
        // Choose enemy type based on map theme, difficulty, and position
        let theme_enemies = self.get_theme_appropriate_enemies(map.theme);
        
        // Divide enemies into tiers based on difficulty
        let mut tier1: Vec<EnemyType> = Vec::new();
        let mut tier2: Vec<EnemyType> = Vec::new();
        let mut tier3: Vec<EnemyType> = Vec::new();
        
        for &enemy in &theme_enemies {
            match enemy {
                EnemyType::Rat | EnemyType::Bat | EnemyType::Spider | EnemyType::Goblin => {
                    tier1.push(enemy);
                },
                EnemyType::Skeleton | EnemyType::Zombie | EnemyType::Orc | EnemyType::Snake | EnemyType::Slime => {
                    tier2.push(enemy);
                },
                EnemyType::Troll | EnemyType::Ghost | EnemyType::Demon | EnemyType::Dragon => {
                    tier3.push(enemy);
                },
            }
        }
        
        // Choose tier based on difficulty and randomness
        let tier_roll = self.rng.range(0, 100) + difficulty;
        
        if tier_roll > 80 && !tier3.is_empty() {
            // High tier enemy
            tier3[self.rng.range(0, tier3.len() as i32) as usize]
        } else if tier_roll > 50 && !tier2.is_empty() {
            // Medium tier enemy
            tier2[self.rng.range(0, tier2.len() as i32) as usize]
        } else if !tier1.is_empty() {
            // Low tier enemy
            tier1[self.rng.range(0, tier1.len() as i32) as usize]
        } else {
            // Fallback
            EnemyType::Goblin
        }
    }
    
    fn get_theme_appropriate_enemies(&self, theme: MapTheme) -> Vec<EnemyType> {
        match theme {
            MapTheme::Dungeon => vec![
                EnemyType::Goblin, EnemyType::Orc, EnemyType::Skeleton, 
                EnemyType::Zombie, EnemyType::Rat, EnemyType::Troll
            ],
            MapTheme::Cave => vec![
                EnemyType::Bat, EnemyType::Spider, EnemyType::Slime,
                EnemyType::Troll, EnemyType::Rat, EnemyType::Snake
            ],
            MapTheme::Forest => vec![
                EnemyType::Spider, EnemyType::Snake, EnemyType::Goblin,
                EnemyType::Bat, EnemyType::Slime
            ],
            MapTheme::Desert => vec![
                EnemyType::Snake, EnemyType::Skeleton, EnemyType::Zombie,
                EnemyType::Demon
            ],
            MapTheme::Ice => vec![
                EnemyType::Troll, EnemyType::Ghost, EnemyType::Zombie,
                EnemyType::Dragon
            ],
            MapTheme::Volcanic => vec![
                EnemyType::Demon, EnemyType::Dragon, EnemyType::Slime,
                EnemyType::Troll
            ],
            MapTheme::Underwater => vec![
                EnemyType::Slime, EnemyType::Snake, EnemyType::Ghost
            ],
        }
    }
    
    fn choose_item_type(&mut self, map: &Map, difficulty: i32) -> ItemType {
        // Choose item type based on map theme and difficulty
        let roll = self.rng.range(0, 100);
        
        // Common items (50%)
        if roll < 50 {
            let common_items = vec![
                ItemType::HealthPotion,
                ItemType::Gold,
                ItemType::ManaPotion
            ];
            return common_items[self.rng.range(0, common_items.len() as i32) as usize];
        }
        
        // Uncommon items (30%)
        if roll < 80 {
            let uncommon_items = vec![
                ItemType::Scroll,
                ItemType::Weapon,
                ItemType::Armor,
                ItemType::Shield,
                ItemType::Key
            ];
            return uncommon_items[self.rng.range(0, uncommon_items.len() as i32) as usize];
        }
        
        // Rare items (20%)
        let rare_items = vec![
            ItemType::Ring,
            ItemType::Amulet,
            ItemType::Gem
        ];
        rare_items[self.rng.range(0, rare_items.len() as i32) as usize]
    }
    
    fn choose_special_feature(&mut self, map: &Map, difficulty: i32) -> SpecialFeatureType {
        // Choose special feature based on map theme and difficulty
        let roll = self.rng.range(0, 100);
        
        if roll < 60 {
            SpecialFeatureType::Chest
        } else if roll < 80 {
            SpecialFeatureType::Shrine
        } else if roll < 90 {
            SpecialFeatureType::Altar
        } else {
            SpecialFeatureType::Statue
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum SpawnType {
    Enemy(EnemyType),
    Item(ItemType),
    Special(SpecialFeatureType),
}

#[derive(Clone, Copy, Debug)]
pub enum SpecialFeatureType {
    Chest,
    Shrine,
    Altar,
    Statue,
}

#[derive(Clone, Debug)]
pub struct EntitySpawn {
    pub entity_type: SpawnType,
    pub x: i32,
    pub y: i32,
}