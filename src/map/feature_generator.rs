use rand::Rng;
use crate::map::{Map, Rect, TileType, MapTheme};
use crate::resources::RandomNumberGenerator;

pub struct DungeonFeatureGenerator {
    pub rng: RandomNumberGenerator,
}

#[derive(Clone, Copy, Debug)]
pub enum SpecialRoomType {
    Treasury,
    Library,
    Armory,
    Shrine,
    Prison,
    Laboratory,
    Garden,
    Throne,
}

#[derive(Clone, Copy, Debug)]
pub enum EnvironmentalHazard {
    LavaPool,
    WaterPool,
    TrapCluster,
    PoisonGas,
    Chasm,
}

impl DungeonFeatureGenerator {
    pub fn new(rng: RandomNumberGenerator) -> Self {
        DungeonFeatureGenerator { rng }
    }
    
    /// Add special features to an existing map
    pub fn add_features(&mut self, map: &mut Map) {
        // Add special rooms
        self.add_special_rooms(map);
        
        // Add environmental hazards
        self.add_environmental_hazards(map);
        
        // Add decorative elements
        self.add_decorative_elements(map);
        
        // Add secret areas
        self.add_secret_areas(map);
    }
    
    fn add_special_rooms(&mut self, map: &mut Map) {
        if map.rooms.is_empty() {
            return;
        }
        
        // Convert 10-20% of rooms to special rooms
        let num_special = (map.rooms.len() as f32 * 0.15) as usize;
        let mut special_count = 0;
        
        // Skip the first and last rooms (entrance and exit)
        let available_rooms: Vec<Rect> = if map.rooms.len() > 2 {
            map.rooms[1..map.rooms.len()-1].to_vec()
        } else {
            map.rooms.clone()
        };
        
        for room in &available_rooms {
            if special_count >= num_special {
                break;
            }
            
            // 30% chance to make this room special
            if self.rng.range(0, 100) < 30 {
                let room_type = self.choose_special_room_type(map);
                self.create_special_room(map, room, room_type);
                special_count += 1;
            }
        }
    }
    
    fn choose_special_room_type(&mut self, map: &Map) -> SpecialRoomType {
        match map.theme {
            MapTheme::Dungeon => {
                let options = [
                    SpecialRoomType::Treasury,
                    SpecialRoomType::Armory,
                    SpecialRoomType::Prison,
                    SpecialRoomType::Shrine,
                ];
                options[self.rng.range(0, options.len() as i32) as usize]
            },
            MapTheme::Cave => {
                let options = [
                    SpecialRoomType::Treasury,
                    SpecialRoomType::Shrine,
                    SpecialRoomType::Laboratory,
                ];
                options[self.rng.range(0, options.len() as i32) as usize]
            },
            MapTheme::Forest => {
                let options = [
                    SpecialRoomType::Garden,
                    SpecialRoomType::Shrine,
                    SpecialRoomType::Library,
                ];
                options[self.rng.range(0, options.len() as i32) as usize]
            },
            _ => SpecialRoomType::Treasury, // Default
        }
    }
    
    fn create_special_room(&mut self, map: &mut Map, room: &Rect, room_type: SpecialRoomType) {
        match room_type {
            SpecialRoomType::Treasury => self.create_treasury(map, room),
            SpecialRoomType::Library => self.create_library(map, room),
            SpecialRoomType::Armory => self.create_armory(map, room),
            SpecialRoomType::Shrine => self.create_shrine(map, room),
            SpecialRoomType::Prison => self.create_prison(map, room),
            SpecialRoomType::Laboratory => self.create_laboratory(map, room),
            SpecialRoomType::Garden => self.create_garden(map, room),
            SpecialRoomType::Throne => self.create_throne_room(map, room),
        }
    }
    
    fn create_treasury(&mut self, map: &mut Map, room: &Rect) {
        // Add some water features (representing treasure pools)
        let center = room.center();
        
        // Small water pool in center
        if room.width() >= 5 && room.height() >= 5 {
            for dy in -1..=1 {
                for dx in -1..=1 {
                    let x = center.0 + dx;
                    let y = center.1 + dy;
                    if room.contains(x, y) {
                        map.set_tile(x, y, TileType::Water);
                    }
                }
            }
        }
        
        // Add some decorative walls around the edges
        for x in room.x1 + 1..room.x2 - 1 {
            if self.rng.range(0, 100) < 20 {
                map.set_tile(x, room.y1 + 1, TileType::Rock);
            }
            if self.rng.range(0, 100) < 20 {
                map.set_tile(x, room.y2 - 2, TileType::Rock);
            }
        }
    }
    
    fn create_library(&mut self, _map: &mut Map, _room: &Rect) {
        // Libraries would have bookshelves, but we'll keep it simple for ASCII
        // Could add special floor patterns or decorative elements
    }
    
    fn create_armory(&mut self, map: &mut Map, room: &Rect) {
        // Add some rock formations representing weapon racks
        for y in room.y1 + 1..room.y2 - 1 {
            if self.rng.range(0, 100) < 30 {
                map.set_tile(room.x1 + 1, y, TileType::Rock);
            }
            if self.rng.range(0, 100) < 30 {
                map.set_tile(room.x2 - 2, y, TileType::Rock);
            }
        }
    }
    
    fn create_shrine(&mut self, map: &mut Map, room: &Rect) {
        let center = room.center();
        
        // Create a small shrine in the center
        map.set_tile(center.0, center.1, TileType::Rock);
        
        // Add some decorative elements around it
        for &(dx, dy) in &[(0, -1), (1, 0), (0, 1), (-1, 0)] {
            let x = center.0 + dx;
            let y = center.1 + dy;
            if room.contains(x, y) && self.rng.range(0, 100) < 50 {
                map.set_tile(x, y, TileType::Sand); // Representing offerings
            }
        }
    }
    
    fn create_prison(&mut self, map: &mut Map, room: &Rect) {
        // Add prison bars (represented as walls) to create cells
        if room.width() >= 6 && room.height() >= 4 {
            let mid_x = (room.x1 + room.x2) / 2;
            
            // Vertical divider
            for y in room.y1 + 1..room.y2 - 1 {
                map.set_tile(mid_x, y, TileType::Wall);
            }
            
            // Add doors in the divider
            if room.height() >= 6 {
                map.set_tile(mid_x, room.y1 + 2, TileType::Door(false));
                map.set_tile(mid_x, room.y2 - 3, TileType::Door(false));
            }
        }
    }
    
    fn create_laboratory(&mut self, map: &mut Map, room: &Rect) {
        // Add some lava pools representing alchemical equipment
        let center = room.center();
        
        // Small lava pool
        map.set_tile(center.0, center.1, TileType::Lava);
        
        // Add some water pools for contrast
        if room.width() >= 5 {
            map.set_tile(center.0 - 2, center.1, TileType::Water);
            map.set_tile(center.0 + 2, center.1, TileType::Water);
        }
    }
    
    fn create_garden(&mut self, map: &mut Map, room: &Rect) {
        // Fill with grass and trees
        for y in room.y1 + 1..room.y2 - 1 {
            for x in room.x1 + 1..room.x2 - 1 {
                if self.rng.range(0, 100) < 60 {
                    map.set_tile(x, y, TileType::Grass);
                } else if self.rng.range(0, 100) < 20 {
                    map.set_tile(x, y, TileType::Tree);
                }
            }
        }
    }
    
    fn create_throne_room(&mut self, map: &mut Map, room: &Rect) {
        let center = room.center();
        
        // Throne (represented as a rock)
        map.set_tile(center.0, center.1, TileType::Rock);
        
        // Red carpet approach (using sand for now)
        for y in room.y1 + 1..center.1 {
            map.set_tile(center.0, y, TileType::Sand);
        }
    }
    
    fn add_environmental_hazards(&mut self, map: &mut Map) {
        let num_hazards = self.rng.range(2, 6);
        
        for _ in 0..num_hazards {
            let hazard_type = self.choose_hazard_type(map);
            self.place_hazard(map, hazard_type);
        }
    }
    
    fn choose_hazard_type(&mut self, map: &Map) -> EnvironmentalHazard {
        match map.theme {
            MapTheme::Dungeon => {
                let options = [
                    EnvironmentalHazard::TrapCluster,
                    EnvironmentalHazard::WaterPool,
                    EnvironmentalHazard::Chasm,
                ];
                options[self.rng.range(0, options.len() as i32) as usize]
            },
            MapTheme::Cave => {
                let options = [
                    EnvironmentalHazard::LavaPool,
                    EnvironmentalHazard::WaterPool,
                    EnvironmentalHazard::Chasm,
                ];
                options[self.rng.range(0, options.len() as i32) as usize]
            },
            MapTheme::Volcanic => {
                let options = [
                    EnvironmentalHazard::LavaPool,
                    EnvironmentalHazard::PoisonGas,
                ];
                options[self.rng.range(0, options.len() as i32) as usize]
            },
            _ => EnvironmentalHazard::WaterPool,
        }
    }
    
    fn place_hazard(&mut self, map: &mut Map, hazard_type: EnvironmentalHazard) {
        // Find a random floor tile that's not in a room
        let mut attempts = 0;
        while attempts < 50 {
            let x = self.rng.range(1, map.width - 1);
            let y = self.rng.range(1, map.height - 1);
            
            let idx = map.xy_idx(x, y);
            if map.tiles[idx] == TileType::Floor {
                // Check if it's not in a room
                let in_room = map.rooms.iter().any(|room| room.contains(x, y));
                
                if !in_room {
                    self.create_hazard(map, x, y, hazard_type);
                    break;
                }
            }
            
            attempts += 1;
        }
    }
    
    fn create_hazard(&mut self, map: &mut Map, x: i32, y: i32, hazard_type: EnvironmentalHazard) {
        match hazard_type {
            EnvironmentalHazard::LavaPool => {
                self.create_lava_pool(map, x, y);
            },
            EnvironmentalHazard::WaterPool => {
                self.create_water_pool(map, x, y);
            },
            EnvironmentalHazard::TrapCluster => {
                self.create_trap_cluster(map, x, y);
            },
            EnvironmentalHazard::PoisonGas => {
                // Represented as void for now
                map.set_tile(x, y, TileType::Void);
            },
            EnvironmentalHazard::Chasm => {
                self.create_chasm(map, x, y);
            },
        }
    }
    
    fn create_lava_pool(&mut self, map: &mut Map, center_x: i32, center_y: i32) {
        let size = self.rng.range(1, 4);
        
        for dy in -size..=size {
            for dx in -size..=size {
                let x = center_x + dx;
                let y = center_y + dy;
                
                if map.in_bounds(x, y) {
                    let distance = (dx * dx + dy * dy) as f32;
                    if distance <= (size * size) as f32 {
                        let idx = map.xy_idx(x, y);
                        if map.tiles[idx] == TileType::Floor {
                            map.set_tile(x, y, TileType::Lava);
                        }
                    }
                }
            }
        }
    }
    
    fn create_water_pool(&mut self, map: &mut Map, center_x: i32, center_y: i32) {
        let size = self.rng.range(1, 3);
        
        for dy in -size..=size {
            for dx in -size..=size {
                let x = center_x + dx;
                let y = center_y + dy;
                
                if map.in_bounds(x, y) {
                    let distance = (dx * dx + dy * dy) as f32;
                    if distance <= (size * size) as f32 {
                        let idx = map.xy_idx(x, y);
                        if map.tiles[idx] == TileType::Floor {
                            map.set_tile(x, y, TileType::Water);
                        }
                    }
                }
            }
        }
    }
    
    fn create_trap_cluster(&mut self, map: &mut Map, center_x: i32, center_y: i32) {
        let num_traps = self.rng.range(2, 6);
        
        for _ in 0..num_traps {
            let dx = self.rng.range(-2, 3);
            let dy = self.rng.range(-2, 3);
            let x = center_x + dx;
            let y = center_y + dy;
            
            if map.in_bounds(x, y) {
                let idx = map.xy_idx(x, y);
                if map.tiles[idx] == TileType::Floor {
                    // 50% chance for visible trap, 50% for hidden
                    let visible = self.rng.range(0, 2) == 0;
                    map.set_tile(x, y, TileType::Trap(visible));
                }
            }
        }
    }
    
    fn create_chasm(&mut self, map: &mut Map, center_x: i32, center_y: i32) {
        // Create a line of void tiles
        let direction = self.rng.range(0, 2); // 0 = horizontal, 1 = vertical
        let length = self.rng.range(3, 7);
        
        if direction == 0 {
            // Horizontal chasm
            for i in 0..length {
                let x = center_x - length/2 + i;
                let y = center_y;
                
                if map.in_bounds(x, y) {
                    let idx = map.xy_idx(x, y);
                    if map.tiles[idx] == TileType::Floor {
                        map.set_tile(x, y, TileType::Void);
                    }
                }
            }
        } else {
            // Vertical chasm
            for i in 0..length {
                let x = center_x;
                let y = center_y - length/2 + i;
                
                if map.in_bounds(x, y) {
                    let idx = map.xy_idx(x, y);
                    if map.tiles[idx] == TileType::Floor {
                        map.set_tile(x, y, TileType::Void);
                    }
                }
            }
        }
        
        // Add bridges across the chasm
        if length >= 5 {
            let bridge_pos = length / 2;
            if direction == 0 {
                let x = center_x - length/2 + bridge_pos;
                let y = center_y;
                if map.in_bounds(x, y) {
                    map.set_tile(x, y, TileType::Bridge);
                }
            } else {
                let x = center_x;
                let y = center_y - length/2 + bridge_pos;
                if map.in_bounds(x, y) {
                    map.set_tile(x, y, TileType::Bridge);
                }
            }
        }
    }
    
    fn add_decorative_elements(&mut self, map: &mut Map) {
        // Add random decorative elements throughout the map
        let num_decorations = self.rng.range(5, 15);
        
        for _ in 0..num_decorations {
            let x = self.rng.range(1, map.width - 1);
            let y = self.rng.range(1, map.height - 1);
            
            let idx = map.xy_idx(x, y);
            if map.tiles[idx] == TileType::Floor {
                // Don't place decorations in rooms
                let in_room = map.rooms.iter().any(|room| room.contains(x, y));
                
                if !in_room && self.rng.range(0, 100) < 30 {
                    self.place_decoration(map, x, y);
                }
            }
        }
    }
    
    fn place_decoration(&mut self, map: &mut Map, x: i32, y: i32) {
        let decoration_type = self.rng.range(0, 4);
        
        match decoration_type {
            0 => map.set_tile(x, y, TileType::Rock),
            1 => map.set_tile(x, y, TileType::Sand),
            2 => if map.theme == MapTheme::Forest || map.theme == MapTheme::Cave {
                map.set_tile(x, y, TileType::Grass);
            },
            3 => if map.theme == MapTheme::Ice {
                map.set_tile(x, y, TileType::Ice);
            },
            _ => {} // No decoration
        }
    }
    
    fn add_secret_areas(&mut self, map: &mut Map) {
        // Add 1-2 secret areas
        let num_secrets = self.rng.range(1, 3);
        
        for _ in 0..num_secrets {
            self.create_secret_area(map);
        }
    }
    
    fn create_secret_area(&mut self, map: &mut Map) {
        // Find a wall area that could be converted to a secret room
        let mut attempts = 0;
        
        while attempts < 100 {
            let x = self.rng.range(2, map.width - 4);
            let y = self.rng.range(2, map.height - 4);
            
            // Check if we can create a 3x3 secret room here
            let mut can_create = true;
            for dy in 0..3 {
                for dx in 0..3 {
                    let check_x = x + dx;
                    let check_y = y + dy;
                    let idx = map.xy_idx(check_x, check_y);
                    
                    if map.tiles[idx] != TileType::Wall {
                        can_create = false;
                        break;
                    }
                }
                if !can_create { break; }
            }
            
            if can_create {
                // Check if it's adjacent to a floor tile (for access)
                let mut has_access = false;
                for dy in -1..=3 {
                    for dx in -1..=3 {
                        if dx >= 0 && dx < 3 && dy >= 0 && dy < 3 {
                            continue; // Skip the room itself
                        }
                        
                        let check_x = x + dx;
                        let check_y = y + dy;
                        
                        if map.in_bounds(check_x, check_y) {
                            let idx = map.xy_idx(check_x, check_y);
                            if map.tiles[idx] == TileType::Floor {
                                has_access = true;
                                break;
                            }
                        }
                    }
                    if has_access { break; }
                }
                
                if has_access {
                    // Create the secret room
                    for dy in 0..3 {
                        for dx in 0..3 {
                            let room_x = x + dx;
                            let room_y = y + dy;
                            
                            if dx == 1 && dy == 1 {
                                // Center - special tile
                                map.set_tile(room_x, room_y, TileType::Water); // Treasure pool
                            } else if dx == 0 || dx == 2 || dy == 0 || dy == 2 {
                                // Walls
                                map.set_tile(room_x, room_y, TileType::Wall);
                            } else {
                                // Floor
                                map.set_tile(room_x, room_y, TileType::Floor);
                            }
                        }
                    }
                    
                    // Add a secret door
                    let door_side = self.rng.range(0, 4);
                    let (door_x, door_y) = match door_side {
                        0 => (x + 1, y),     // Top
                        1 => (x + 2, y + 1), // Right
                        2 => (x + 1, y + 2), // Bottom
                        _ => (x, y + 1),     // Left
                    };
                    
                    map.set_tile(door_x, door_y, TileType::Door(false)); // Secret door (closed)
                    break;
                }
            }
            
            attempts += 1;
        }
    }
}