use crossterm::style::Color;
use std::time::{Duration, Instant};

/// A visual effect that can be displayed on the screen
#[derive(Clone)]
pub struct VisualEffect {
    pub effect_type: EffectType,
    pub position: (i32, i32),
    pub duration: Duration,
    pub start_time: Instant,
    pub completed: bool,
}

/// The type of visual effect
#[derive(Clone)]
pub enum EffectType {
    /// A particle effect that moves from one position to another
    Particle {
        glyph: char,
        color: Color,
        target: (i32, i32),
        speed: f32,
    },
    /// A flash effect that briefly changes the color of a tile
    Flash {
        glyph: char,
        colors: Vec<Color>,
        interval: Duration,
    },
    /// A text effect that displays text at a position
    Text {
        text: String,
        color: Color,
        offset_y: i32,
        fade: bool,
    },
    /// An explosion effect that radiates outward from a center point
    Explosion {
        radius: i32,
        color: Color,
        glyph: char,
    },
}

impl VisualEffect {
    /// Create a new particle effect
    pub fn particle(position: (i32, i32), target: (i32, i32), glyph: char, color: Color, duration: Duration) -> Self {
        VisualEffect {
            effect_type: EffectType::Particle {
                glyph,
                color,
                target,
                speed: 1.0,
            },
            position,
            duration,
            start_time: Instant::now(),
            completed: false,
        }
    }

    /// Create a new flash effect
    pub fn flash(position: (i32, i32), glyph: char, colors: Vec<Color>, duration: Duration) -> Self {
        VisualEffect {
            effect_type: EffectType::Flash {
                glyph,
                colors,
                interval: Duration::from_millis(100),
            },
            position,
            duration,
            start_time: Instant::now(),
            completed: false,
        }
    }

    /// Create a new text effect
    pub fn text(position: (i32, i32), text: String, color: Color, duration: Duration, fade: bool) -> Self {
        VisualEffect {
            effect_type: EffectType::Text {
                text,
                color,
                offset_y: 0,
                fade,
            },
            position,
            duration,
            start_time: Instant::now(),
            completed: false,
        }
    }

    /// Create a new explosion effect
    pub fn explosion(position: (i32, i32), radius: i32, color: Color, glyph: char, duration: Duration) -> Self {
        VisualEffect {
            effect_type: EffectType::Explosion {
                radius,
                color,
                glyph,
            },
            position,
            duration,
            start_time: Instant::now(),
            completed: false,
        }
    }

    /// Update the effect state
    pub fn update(&mut self) {
        // Check if the effect has completed
        if self.start_time.elapsed() >= self.duration {
            self.completed = true;
            return;
        }

        // Update based on effect type
        match &mut self.effect_type {
            EffectType::Particle { target, speed, .. } => {
                // Calculate the direction vector
                let dx = target.0 - self.position.0;
                let dy = target.1 - self.position.1;
                let distance = ((dx * dx + dy * dy) as f32).sqrt();

                // If we're close enough to the target, complete the effect
                if distance < 0.5 {
                    self.completed = true;
                    return;
                }

                // Normalize the direction vector and scale by speed
                let nx = dx as f32 / distance * *speed;
                let ny = dy as f32 / distance * *speed;

                // Update the position
                self.position.0 += nx as i32;
                self.position.1 += ny as i32;
            },
            EffectType::Text { offset_y, fade: true, .. } => {
                // Move the text upward over time
                *offset_y -= 1;
            },
            _ => {
                // Other effect types don't need updates
            }
        }
    }

    /// Get the current visual representation of the effect
    pub fn get_visual(&self) -> Option<(char, Color)> {
        // If the effect is completed, don't render it
        if self.completed {
            return None;
        }

        // Calculate the progress of the effect (0.0 to 1.0)
        let progress = self.start_time.elapsed().as_secs_f32() / self.duration.as_secs_f32();

        // Return the visual based on effect type
        match &self.effect_type {
            EffectType::Particle { glyph, color, .. } => {
                Some((*glyph, *color))
            },
            EffectType::Flash { glyph, colors, interval } => {
                // Calculate which color to use based on time
                let interval_secs = interval.as_secs_f32();
                let elapsed = self.start_time.elapsed().as_secs_f32();
                let index = (elapsed / interval_secs) as usize % colors.len();
                Some((*glyph, colors[index]))
            },
            EffectType::Text { text: _, color, .. } => {
                // For text effects, we don't return a visual here
                // Text is rendered separately
                None
            },
            EffectType::Explosion { glyph, color, radius } => {
                // Calculate the current radius based on progress
                let current_radius = (progress * *radius as f32) as i32;
                if current_radius <= 0 {
                    return None;
                }
                Some((*glyph, *color))
            },
        }
    }

    /// Get the text to display for text effects
    pub fn get_text(&self) -> Option<(&str, Color, i32)> {
        if self.completed {
            return None;
        }

        match &self.effect_type {
            EffectType::Text { text, color, offset_y, fade } => {
                // If fading, adjust the color alpha based on progress
                let color = if *fade {
                    let progress = self.start_time.elapsed().as_secs_f32() / self.duration.as_secs_f32();
                    // Crossterm doesn't support alpha, so we can't actually fade
                    // But we could use different colors or return None when progress is high
                    if progress > 0.8 {
                        return None;
                    }
                    *color
                } else {
                    *color
                };
                Some((text, color, *offset_y))
            },
            _ => None,
        }
    }

    /// Get the positions affected by an explosion effect
    pub fn get_explosion_positions(&self) -> Vec<(i32, i32)> {
        if self.completed {
            return Vec::new();
        }

        match &self.effect_type {
            EffectType::Explosion { radius, .. } => {
                let progress = self.start_time.elapsed().as_secs_f32() / self.duration.as_secs_f32();
                let current_radius = (progress * *radius as f32) as i32;
                
                let mut positions = Vec::new();
                for y in -current_radius..=current_radius {
                    for x in -current_radius..=current_radius {
                        let distance = ((x * x + y * y) as f32).sqrt();
                        if distance <= current_radius as f32 && distance > (current_radius - 1) as f32 {
                            positions.push((self.position.0 + x, self.position.1 + y));
                        }
                    }
                }
                positions
            },
            _ => Vec::new(),
        }
    }
}

/// A manager for visual effects
pub struct EffectManager {
    pub effects: Vec<VisualEffect>,
}

impl EffectManager {
    /// Create a new effect manager
    pub fn new() -> Self {
        EffectManager {
            effects: Vec::new(),
        }
    }

    /// Add a new effect
    pub fn add_effect(&mut self, effect: VisualEffect) {
        self.effects.push(effect);
    }

    /// Update all effects
    pub fn update(&mut self) {
        // Update each effect
        for effect in &mut self.effects {
            effect.update();
        }

        // Remove completed effects
        self.effects.retain(|effect| !effect.completed);
    }

    /// Clear all effects
    pub fn clear(&mut self) {
        self.effects.clear();
    }
}