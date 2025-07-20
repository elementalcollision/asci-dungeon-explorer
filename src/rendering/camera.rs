use crate::map::Map;

/// Camera struct for handling viewport calculations
#[derive(Clone)]
pub struct Camera {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub map_width: i32,
    pub map_height: i32,
}

impl Camera {
    /// Create a new camera with the given viewport dimensions
    pub fn new(width: i32, height: i32, map_width: i32, map_height: i32) -> Self {
        Camera {
            x: 0,
            y: 0,
            width,
            height,
            map_width,
            map_height,
        }
    }

    /// Center the camera on a specific position
    pub fn center_on(&mut self, x: i32, y: i32) {
        self.x = x - self.width / 2;
        self.y = y - self.height / 2;
        self.constrain();
    }

    /// Constrain the camera to the map boundaries
    pub fn constrain(&mut self) {
        if self.x < 0 {
            self.x = 0;
        } else if self.x + self.width > self.map_width {
            self.x = self.map_width - self.width;
        }

        if self.y < 0 {
            self.y = 0;
        } else if self.y + self.height > self.map_height {
            self.y = self.map_height - self.height;
        }
    }

    /// Move the camera by the given delta
    pub fn move_by(&mut self, dx: i32, dy: i32) {
        self.x += dx;
        self.y += dy;
        self.constrain();
    }

    /// Check if a world position is visible in the camera viewport
    pub fn is_visible(&self, x: i32, y: i32) -> bool {
        x >= self.x && x < self.x + self.width && y >= self.y && y < self.y + self.height
    }

    /// Convert a world position to a screen position
    pub fn world_to_screen(&self, x: i32, y: i32) -> (i32, i32) {
        (x - self.x, y - self.y)
    }

    /// Convert a screen position to a world position
    pub fn screen_to_world(&self, x: i32, y: i32) -> (i32, i32) {
        (x + self.x, y + self.y)
    }

    /// Update the camera dimensions
    pub fn resize(&mut self, width: i32, height: i32) {
        self.width = width;
        self.height = height;
        self.constrain();
    }

    /// Update the map dimensions
    pub fn update_map_size(&mut self, map_width: i32, map_height: i32) {
        self.map_width = map_width;
        self.map_height = map_height;
        self.constrain();
    }
}

/// A helper function to create a camera centered on a player position
pub fn create_camera_for_map(map: &Map, viewport_width: i32, viewport_height: i32, player_pos: (i32, i32)) -> Camera {
    let mut camera = Camera::new(viewport_width, viewport_height, map.width, map.height);
    camera.center_on(player_pos.0, player_pos.1);
    camera
}