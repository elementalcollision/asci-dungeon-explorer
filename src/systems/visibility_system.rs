use specs::{System, ReadStorage, WriteStorage, ReadExpect, WriteExpect, Join};
use crate::components::{Position, Viewshed, Player};
use crate::map::Map;

pub struct VisibilitySystem {}

impl<'a> System<'a> for VisibilitySystem {
    type SystemData = (
        WriteStorage<'a, Viewshed>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, Player>,
        WriteExpect<'a, Map>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut viewshed, pos, player, mut map) = data;

        // Reset all visible tiles
        for tile in map.visible_tiles.iter_mut() {
            *tile = false;
        }

        // Process each entity with a viewshed and position
        for (viewshed, pos, _player) in (&mut viewshed, &pos, &player).join() {
            if viewshed.dirty {
                viewshed.dirty = false;
                viewshed.visible_tiles.clear();
                
                // Simple field of view algorithm (will be replaced with a more sophisticated one)
                // For now, just make a square around the player visible
                for y in -viewshed.range..=viewshed.range {
                    for x in -viewshed.range..=viewshed.range {
                        let target_x = pos.x + x;
                        let target_y = pos.y + y;
                        
                        if map.in_bounds(target_x, target_y) {
                            let idx = map.xy_idx(target_x, target_y);
                            viewshed.visible_tiles.push((target_x, target_y));
                            map.visible_tiles[idx] = true;
                            map.revealed_tiles[idx] = true;
                        }
                    }
                }
            }
        }
    }
}