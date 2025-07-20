use specs::{System, WriteStorage, ReadStorage, Entities, Entity, Join, ReadExpect};
use crate::components::{
    Position, Player, PlayerInput, WantsToMove, WantsToAttack, WantsToPickupItem,
    WantsToUseItem, WantsToDropItem, Viewshed
};
use crate::map::Map;

pub struct PlayerController;

impl<'a> System<'a> for PlayerController {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, WantsToMove>,
        WriteStorage<'a, WantsToAttack>,
        WriteStorage<'a, WantsToPickupItem>,
        WriteStorage<'a, WantsToUseItem>,
        WriteStorage<'a, WantsToDropItem>,
        ReadStorage<'a, Player>,
        ReadStorage<'a, Position>,
        WriteStorage<'a, PlayerInput>,
        WriteStorage<'a, Viewshed>,
        ReadExpect<'a, Map>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            entities, 
            mut wants_move, 
            mut wants_attack, 
            mut wants_pickup, 
            mut wants_use, 
            mut wants_drop,
            player, 
            positions, 
            mut player_input, 
            mut viewsheds,
            map
        ) = data;

        // Process player input
        for (entity, _player, pos, input, viewshed) in (&entities, &player, &positions, &mut player_input, &mut viewsheds).join() {
            // Handle movement intent
            if let Some(movement) = input.move_intent {
                let destination_x = pos.x + movement.0;
                let destination_y = pos.y + movement.1;
                
                // Check if the destination is valid
                if map.in_bounds(destination_x, destination_y) {
                    let destination_idx = map.xy_idx(destination_x, destination_y);
                    
                    // Check if there's an entity to attack at the destination
                    let mut attack_target = None;
                    for (target_entity, target_pos) in (&entities, &positions).join() {
                        if target_pos.x == destination_x && target_pos.y == destination_y {
                            attack_target = Some(target_entity);
                            break;
                        }
                    }
                    
                    if let Some(target) = attack_target {
                        // Create attack intent
                        wants_attack.insert(entity, WantsToAttack { target }).expect("Failed to insert attack intent");
                    } else if !map.is_blocked(destination_x, destination_y) {
                        // Create movement intent
                        wants_move.insert(entity, WantsToMove { destination: (destination_x, destination_y) }).expect("Failed to insert move intent");
                        
                        // Mark viewshed as dirty since we're moving
                        viewshed.dirty = true;
                    }
                }
            }
            
            // Handle pickup intent
            if input.pickup_intent {
                // Find items at the player's position
                let mut items_at_pos = Vec::new();
                for (item_entity, item_pos) in (&entities, &positions).join() {
                    if item_pos.x == pos.x && item_pos.y == pos.y && entity != item_entity {
                        items_at_pos.push(item_entity);
                    }
                }
                
                // If there's an item, pick up the first one
                if let Some(item) = items_at_pos.first() {
                    wants_pickup.insert(entity, WantsToPickupItem { item: *item }).expect("Failed to insert pickup intent");
                }
            }
            
            // Handle use item intent
            if let Some(item_idx) = input.use_item_intent {
                // The actual item entity will be resolved in the inventory system
                wants_use.insert(entity, WantsToUseItem { item: Entity::from_bits(item_idx as u64), target: None }).expect("Failed to insert use item intent");
            }
            
            // Handle drop item intent
            if let Some(item_idx) = input.drop_intent {
                // The actual item entity will be resolved in the inventory system
                wants_drop.insert(entity, WantsToDropItem { item: Entity::from_bits(item_idx as u64) }).expect("Failed to insert drop item intent");
            }
            
            // Clear input after processing
            input.clear();
        }
    }
}