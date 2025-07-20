use specs::{System, WriteStorage, ReadStorage, Entities, Entity, Join, Write};
use crate::components::{
    CombatFeedback, CombatFeedbackType, Position, Renderable, AnimationType
};
use crate::rendering::terminal::with_terminal;
use crossterm::style::Color;

pub struct VisualEffectsSystem {}

impl<'a> System<'a> for VisualEffectsSystem {
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, CombatFeedback>,
        ReadStorage<'a, Position>,
        WriteStorage<'a, Renderable>,
        Write<'a, crate::systems::sound_effect_system::ScreenShakeState>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, combat_feedback, positions, mut renderables, mut screen_shake) = data;

        // Render floating combat text and effects
        let _ = with_terminal(|terminal| {
            // Apply screen shake offset
            let shake_offset_x = screen_shake.offset_x as i16;
            let shake_offset_y = screen_shake.offset_y as i16;
            
            // Render combat feedback effects
            for (entity, feedback) in (&entities, &combat_feedback).join() {
                match &feedback.feedback_type {
                    CombatFeedbackType::DamageText { damage, damage_type, is_critical } => {
                        self.render_damage_text(
                            terminal,
                            feedback,
                            *damage,
                            *is_critical,
                            shake_offset_x,
                            shake_offset_y
                        )?;
                    },
                    CombatFeedbackType::HealingText { healing } => {
                        self.render_healing_text(
                            terminal,
                            feedback,
                            *healing,
                            shake_offset_x,
                            shake_offset_y
                        )?;
                    },
                    CombatFeedbackType::StatusText { text } => {
                        self.render_status_text(
                            terminal,
                            feedback,
                            text,
                            shake_offset_x,
                            shake_offset_y
                        )?;
                    },
                    _ => {} // Other feedback types handled elsewhere
                }
            }
            
            Ok(())
        });
        
        // Apply flash effects to renderables
        for (entity, feedback) in (&entities, &combat_feedback).join() {
            if matches!(feedback.animation_type, AnimationType::Flash) {
                if let Some(renderable) = renderables.get_mut(entity) {
                    self.apply_flash_effect(renderable, feedback);
                }
            }
        }
    }
}

impl VisualEffectsSystem {
    fn render_damage_text(
        &self,
        terminal: &mut crate::rendering::terminal::Terminal,
        feedback: &CombatFeedback,
        damage: i32,
        is_critical: bool,
        shake_x: i16,
        shake_y: i16,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let x = (feedback.position.x + feedback.position.offset_x) as u16;
        let y = (feedback.position.y + feedback.position.offset_y) as u16;
        
        // Apply screen shake
        let final_x = (x as i16 + shake_x).max(0) as u16;
        let final_y = (y as i16 + shake_y).max(0) as u16;
        
        // Format damage text
        let damage_text = if is_critical {
            format!("{}!", damage)
        } else {
            format!("{}", damage)
        };
        
        // Calculate alpha based on remaining duration
        let alpha = feedback.duration / feedback.max_duration;
        let color = if alpha > 0.5 {
            feedback.color
        } else {
            // Fade to darker color
            match feedback.color {
                Color::Yellow => Color::DarkYellow,
                Color::Red => Color::DarkRed,
                Color::Green => Color::DarkGreen,
                Color::Blue => Color::DarkBlue,
                Color::Cyan => Color::DarkCyan,
                Color::Magenta => Color::DarkMagenta,
                _ => Color::DarkGrey,
            }
        };
        
        // Render the damage text
        terminal.draw_text(final_x, final_y, &damage_text, color, Color::Black)?;
        
        // Add extra visual flair for critical hits
        if is_critical && feedback.duration > feedback.max_duration * 0.7 {
            terminal.draw_text(final_x.saturating_sub(1), final_y, "*", Color::Yellow, Color::Black)?;
            terminal.draw_text(final_x + damage_text.len() as u16, final_y, "*", Color::Yellow, Color::Black)?;
        }
        
        Ok(())
    }
    
    fn render_healing_text(
        &self,
        terminal: &mut crate::rendering::terminal::Terminal,
        feedback: &CombatFeedback,
        healing: i32,
        shake_x: i16,
        shake_y: i16,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let x = (feedback.position.x + feedback.position.offset_x) as u16;
        let y = (feedback.position.y + feedback.position.offset_y) as u16;
        
        let final_x = (x as i16 + shake_x).max(0) as u16;
        let final_y = (y as i16 + shake_y).max(0) as u16;
        
        let healing_text = format!("+{}", healing);
        terminal.draw_text(final_x, final_y, &healing_text, Color::Green, Color::Black)?;
        
        Ok(())
    }
    
    fn render_status_text(
        &self,
        terminal: &mut crate::rendering::terminal::Terminal,
        feedback: &CombatFeedback,
        text: &str,
        shake_x: i16,
        shake_y: i16,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let x = (feedback.position.x + feedback.position.offset_x) as u16;
        let y = (feedback.position.y + feedback.position.offset_y) as u16;
        
        let final_x = (x as i16 + shake_x).max(0) as u16;
        let final_y = (y as i16 + shake_y).max(0) as u16;
        
        terminal.draw_text(final_x, final_y, text, feedback.color, Color::Black)?;
        
        Ok(())
    }
    
    fn apply_flash_effect(&self, renderable: &mut Renderable, feedback: &CombatFeedback) {
        // Calculate flash intensity based on remaining duration
        let flash_intensity = feedback.duration / feedback.max_duration;
        
        if flash_intensity > 0.5 {
            // Full flash effect
            renderable.bg = feedback.color;
        } else if flash_intensity > 0.0 {
            // Fading flash effect
            renderable.bg = Color::DarkGrey;
        } else {
            // Restore original background
            renderable.bg = Color::Black;
        }
    }
}

// System for creating particle effects
pub struct ParticleEffectSystem {}

impl<'a> System<'a> for ParticleEffectSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, ParticleEffect>,
        ReadStorage<'a, CombatFeedback>,
        ReadStorage<'a, Position>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, mut particles, combat_feedback, positions) = data;

        // Create particle effects based on combat feedback
        for (entity, feedback) in (&entities, &combat_feedback).join() {
            if let Some(pos) = positions.get(entity) {
                match &feedback.feedback_type {
                    CombatFeedbackType::DamageText { damage_type, is_critical, .. } => {
                        if *is_critical {
                            // Create particle burst for critical hits
                            self.create_particle_burst(
                                entity,
                                pos,
                                *damage_type,
                                &mut particles
                            );
                        }
                    },
                    _ => {}
                }
            }
        }
        
        // Update existing particles
        self.update_particles(&mut particles);
    }
}

impl ParticleEffectSystem {
    fn create_particle_burst(
        &self,
        entity: Entity,
        position: &Position,
        damage_type: crate::components::DamageType,
        particles: &mut WriteStorage<ParticleEffect>,
    ) {
        let particle_color = match damage_type {
            crate::components::DamageType::Fire => Color::Red,
            crate::components::DamageType::Ice => Color::Cyan,
            crate::components::DamageType::Lightning => Color::Yellow,
            crate::components::DamageType::Poison => Color::Green,
            crate::components::DamageType::Holy => Color::White,
            crate::components::DamageType::Dark => Color::Magenta,
            _ => Color::White,
        };
        
        let particle = ParticleEffect {
            position: crate::components::FloatingPosition {
                x: position.x as f32,
                y: position.y as f32,
                offset_x: 0.0,
                offset_y: 0.0,
            },
            velocity: ParticleVelocity {
                x: 0.0,
                y: -1.0, // Move upward
            },
            color: particle_color,
            character: '*',
            lifetime: 1.0,
            max_lifetime: 1.0,
        };
        
        particles.insert(entity, particle)
            .expect("Failed to insert particle effect");
    }
    
    fn update_particles(&self, particles: &mut WriteStorage<ParticleEffect>) {
        let mut expired_particles = Vec::new();
        
        for (entity, mut particle) in (&entities, particles).join() {
            // Update particle position
            particle.position.offset_x += particle.velocity.x * 0.016;
            particle.position.offset_y += particle.velocity.y * 0.016;
            
            // Update lifetime
            particle.lifetime -= 0.016;
            
            // Apply gravity/physics
            particle.velocity.y += 0.5; // Gravity
            particle.velocity.x *= 0.98; // Air resistance
            
            // Mark expired particles
            if particle.lifetime <= 0.0 {
                expired_particles.push(entity);
            }
        }
        
        // Remove expired particles
        for entity in expired_particles {
            particles.remove(entity);
        }
    }
}

// Particle effect component
#[derive(Component, Debug, Clone)]
#[storage(VecStorage)]
pub struct ParticleEffect {
    pub position: crate::components::FloatingPosition,
    pub velocity: ParticleVelocity,
    pub color: Color,
    pub character: char,
    pub lifetime: f32,
    pub max_lifetime: f32,
}

#[derive(Debug, Clone)]
pub struct ParticleVelocity {
    pub x: f32,
    pub y: f32,
}