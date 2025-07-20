use rand::Rng;
use std::cmp::{max, min};
use crate::map::{Map, Rect, TileType, MapTheme, Direction};
use crate::resources::RandomNumberGenerator;

pub trait MapGenerator {
    fn generate_map(&mut self, width: i32, height: i32, depth: i32) -> Map;
}

pub struct RoomBasedDungeonGenerator {
    pub rng: RandomNumberGenerator,
    pub max_rooms: i32,
    pub min_room_size: i32,
    pub max_room_size: i32,
    pub min_corridor_length: i32,
    pub max_corridor_length: i32,
}

impl RoomBasedDungeonGenerator {
    pub fn new(rng: RandomNumberGenerator) -> Self {
        RoomBasedDungeonGenerator {
            rng,
            max_rooms: 30,
            min_room_size: 6,
            max_room_size: 10,
            min_corridor_length: 3,
            max_corridor_length: 10,
        }
    }
    
    fn add_room(&mut self, map: &mut Map, room: &Rect) {
        // Set all tiles in the room to floor
        for y in room.y1..room.y2 {
            for x in room.x1..room.x2 {
                if map.in_bounds(x, y) {
                    map.set_tile(x, y, TileType::Floor);
                }
            }
        }
    }
    
    fn add_corridor(&mut self, map: &mut Map, x1: i32, y1: i32, x2: i32, y2: i32) {
        // Store corridor points for later use
        let mut corridor = Vec::new();
        
        // Decide whether to go horizontally or vertically first
        let horizontal_first = self.rng.range(0, 2) == 0;
        
        if horizontal_first {
            // Go horizontally first, then vertically
            self.apply_horizontal_corridor(map, x1, x2, y1, &mut corridor);
            self.apply_vertical_corridor(map, y1, y2, x2, &mut corridor);
        } else {
            // Go vertically first, then horizontally
            self.apply_vertical_corridor(map, y1, y2, x1, &mut corridor);
            self.apply_horizontal_corridor(map, x1, x2, y2, &mut corridor);
        }
        
        // Add the corridor to the map's corridors list
        map.corridors.push(corridor);
    }
    
    fn apply_horizontal_corridor(&mut self, map: &mut Map, x1: i32, x2: i32, y: i32, corridor: &mut Vec<(i32, i32)>) {
        for x in min(x1, x2)..=max(x1, x2) {
            if map.in_bounds(x, y) {
                map.set_tile(x, y, TileType::Floor);
                corridor.push((x, y));
            }
        }
    }
    
    fn apply_vertical_corridor(&mut self, map: &mut Map, y1: i32, y2: i32, x: i32, corridor: &mut Vec<(i32, i32)>) {
        for y in min(y1, y2)..=max(y1, y2) {
            if map.in_bounds(x, y) {
                map.set_tile(x, y, TileType::Floor);
                corridor.push((x, y));
            }
        }
    }
    
    fn add_doors(&mut self, map: &mut Map) {
        // Find potential door locations (corridor tiles adjacent to exactly one room)
        let mut door_positions = Vec::new();
        
        // Check each corridor
        for corridor in &map.corridors {
            for &(x, y) in corridor {
                // Check if this position could be a door (has exactly one adjacent room tile)
                let mut adjacent_room_count = 0;
                
                for dir in Direction::cardinals() {
                    let (dx, dy) = dir.delta();
                    let nx = x + dx;
                    let ny = y + dy;
                    
                    if map.in_bounds(nx, ny) {
                        if let Some(tile) = map.get_tile(nx, ny) {
                            if tile == TileType::Floor {
                                // Check if this floor tile is part of a room (not a corridor)
                                let is_room_tile = map.rooms.iter().any(|room| {
                                    nx >= room.x1 && nx < room.x2 && ny >= room.y1 && ny < room.y2
                                });
                                
                                if is_room_tile {
                                    adjacent_room_count += 1;
                                }
                            }
                        }
                    }
                }
                
                // If this corridor tile connects to exactly one room, it's a door candidate
                if adjacent_room_count == 1 {
                    door_positions.push((x, y));
                }
            }
        }
        
        // Place doors at some of the identified positions
        for &(x, y) in door_positions.iter() {
            // 30% chance to place a door
            if self.rng.range(0, 100) < 30 {
                map.set_tile(x, y, TileType::Door(false)); // Closed door
            }
        }
    }
    
    fn place_stairs(&mut self, map: &mut Map) {
        // Place stairs down at the center of the last room
        if let Some(last_room) = map.rooms.last() {
            let (x, y) = last_room.center();
            map.set_tile(x, y, TileType::DownStairs);
            map.exit = (x, y);
        }
        
        // Place stairs up at the center of the first room
        if let Some(first_room) = map.rooms.first() {
            let (x, y) = first_room.center();
            map.set_tile(x, y, TileType::UpStairs);
            map.entrance = (x, y);
        }
    }
}

impl MapGenerator for RoomBasedDungeonGenerator {
    fn generate_map(&mut self, width: i32, height: i32, depth: i32) -> Map {
        let mut map = Map::new_with_theme(width, height, depth, MapTheme::Dungeon, 0);
        
        // Create rooms
        let mut rooms = Vec::new();
        
        for _ in 0..self.max_rooms {
            // Random room size
            let w = self.rng.range(self.min_room_size, self.max_room_size + 1);
            let h = self.rng.range(self.min_room_size, self.max_room_size + 1);
            
            // Random room position
            let x = self.rng.range(1, width - w - 1);
            let y = self.rng.range(1, height - h - 1);
            
            let new_room = Rect::new(x, y, w, h);
            
            // Check if the room overlaps with any existing room
            let mut overlaps = false;
            for other_room in &rooms {
                if new_room.intersect(other_room) {
                    overlaps = true;
                    break;
                }
            }
            
            if !overlaps {
                // Add the room to the map
                self.add_room(&mut map, &new_room);
                
                // Connect to previous room if this isn't the first room
                if !rooms.is_empty() {
                    let (new_x, new_y) = new_room.center();
                    let (prev_x, prev_y) = rooms[rooms.len() - 1].center();
                    self.add_corridor(&mut map, prev_x, prev_y, new_x, new_y);
                }
                
                // Add the room to our list
                rooms.push(new_room);
            }
        }
        
        // Store the rooms in the map
        map.rooms = rooms;
        
        // Add doors to the map
        self.add_doors(&mut map);
        
        // Place stairs
        self.place_stairs(&mut map);
        
        // Update the blocked array
        map.populate_blocked();
        
        map
    }
}