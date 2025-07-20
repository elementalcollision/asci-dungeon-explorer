use specs::{System, WriteStorage, ReadStorage, Entities, Join, ReadExpect};
use crate::components::{Position, WantsToMove, BlocksTile};
use crate::map::Map;

pub struct MovementSystem;

impl<'a> System<'a> for MovementSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, Position>,
        ReadStorage<'a, WantsToMove>,
        ReadStorage<'a, BlocksTile>,
        ReadExpect<'a, Map>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, mut positions, wants_move, blockers, map) = data;

        // Process movement intents
        for (entity, pos, movement) in (&entities, &mut positions, &wants_move).join() {
            let (destination_x, destination_y) = movement.destination;
            
            // Check if the destination is valid
            if map.in_bounds(destination_x, destination_y) {
                let destination_idx = map.xy_idx(destination_x, destination_y);
                
                // Check if the destination is blocked by the map
                if !map.is_blocked(destination_x, destination_y) {
                    // Check if the destination is blocked by an entity
                    let mut blocked = false;
                    for (blocker_entity, blocker_pos, _) in (&entities, &positions, &blockers).join() {
                        if blocker_pos.x == destination_x && blocker_pos.y == destination_y {
                            blocked = true;
                            break;
                        }
                    }
                    
                    // If not blocked, move the entity
                    if !blocked {
                        pos.x = destination_x;
                        pos.y = destination_y;
                    }
                }
            }
        }
        
        // Clean up the WantsToMove components
        entities.join().for_each(|entity| {
            let _ = wants_move.remove(entity);
        });
    }
}