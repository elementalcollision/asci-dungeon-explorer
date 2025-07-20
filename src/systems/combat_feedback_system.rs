use specs::{System, WriteStorage, ReadStorage, Entities, Entity, Join, Write};
use crate::components::{
    DamageInfo, CombatStats, Name, Player, Monster, Position, Renderable, StatusEffects
};
use crate::resources::{GameLog, RandomNumberGenerator};
use crossterm::style::Color;

pub struct CombatFeedbackSystem {}

impl<'a> System<'a> for CombatFeedbackSystem {
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, DamageInfo>,
        ReadStorage<'a, CombatStats>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, Player>,
        ReadStorage<'a, Monster>,
        ReadStorage<'a, Position>,
        WriteStorage<'a, Renderable>,
        WriteStorage<'a, CombatFeedback>,
        ReadStorage<'a, StatusEffects>,
        Write<'a, GameLog>,
        Write<'a, RandomNumberGenerator>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            entities, 
            damage_info, 
            combat_stats, 
            names, 
            players, 
            monsters,
            positions,
            mut renderables,
            mut combat_feedback,
            status_effects,
            mut gamelog, 
            mut rng
        ) = data;

        // Process damage for visual feedback
        for (entity, damage, stats, name) in (&entities, &damage_info, &combat_stats, &names).join() {
            // Create visual feedback for damage
            self.create_damage_feedback(
                entity,
                damage,
                stats,
                name,
                &positions,
                &mut combat_feedback,
                &mut rng
            );
            
            // Create screen shake effect for significant damage
            if damage.base_damage > 10 || damage.is_critical {
                self.create_screen_shake_feedback(
                    entity,
                    damage,
                    &mut combat_feedback
                );
            }
            
            // Create color flash effect on hit
            if let Some(renderable) = renderables.get_mut(entity) {
                self.apply_hit_flash_effect(renderable, damage);
            }
            
            // Enhanced combat logging
            self.create_enhanced_combat_log(
                entity,
                damage,
                stats,
                name,
                &players,
                &monsters,
                &status_effects,
                &mut gamelog
            );
        }
        
        // Update existing feedback effects
        self.update_feedback_effects(&mut combat_feedback, &mut renderables);
    }
}

impl CombatFeedbackSystem {
    fn create_damage_feedback(
        &self,
        target: Entity,
        damage: &DamageInfo,
        stats: &CombatStats,
        name: &Name,
        positions: &ReadStorage<Position>,
        combat_feedback: &mut WriteStorage<CombatFeedback>,
        rng: &mut RandomNumberGenerator,
    ) {
        if let Some(pos) = positions.get(target) {
            // Create floating damage text
            let feedback = CombatFeedback {
                feedback_type: CombatFeedbackType::DamageText {
                    damage: damage.base_damage,
                    damage_type: damage.damage_type,
                    is_critical: damage.is_critical,
                },
                position: FloatingPosition {
                    x: pos.x as f32,
                    y: pos.y as f32,
                    offset_x: (rng.roll_dice(1, 6) - 3) as f32 * 0.2, // Random offset
                    offset_y: -0.5, // Start slightly above
                },
                duration: if damage.is_critical { 2.0 } else { 1.5 },
                max_duration: if damage.is_critical { 2.0 } else { 1.5 },
                color: self.get_damage_color(damage),
                animation_type: if damage.is_critical {
                    AnimationType::CriticalBounce
                } else {
                    AnimationType::FloatUp
                },
            };
            
            combat_feedback.insert(target, feedback)
                .expect("Failed to insert combat feedback");
        }
    }
    
    fn create_screen_shake_feedback(
        &self,
        target: Entity,
        damage: &DamageInfo,
        combat_feedback: &mut WriteStorage<CombatFeedback>,
    ) {
        let shake_intensity = if damage.is_critical {
            ShakeIntensity::Heavy
        } else if damage.base_damage > 15 {
            ShakeIntensity::Medium
        } else {
            ShakeIntensity::Light
        };
        
        let feedback = CombatFeedback {
            feedback_type: CombatFeedbackType::ScreenShake {
                intensity: shake_intensity,
            },
            position: FloatingPosition::default(),
            duration: 0.3,
            max_duration: 0.3,
            color: Color::White,
            animation_type: AnimationType::Shake,
        };
        
        // Use a special entity ID for screen effects
        let screen_effect_entity = Entity::from_raw(u32::MAX);
        combat_feedback.insert(screen_effect_entity, feedback)
            .expect("Failed to insert screen shake feedback");
    }
    
    fn apply_hit_flash_effect(&self, renderable: &mut Renderable, damage: &DamageInfo) {
        // Flash the entity with damage type color
        let flash_color = match damage.damage_type {
            crate::components::DamageType::Fire => Color::Red,
            crate::components::DamageType::Ice => Color::Cyan,
            crate::components::DamageType::Lightning => Color::Yellow,
            crate::components::DamageType::Poison => Color::Green,
            crate::components::DamageType::Holy => Color::White,
            crate::components::DamageType::Dark => Color::Magenta,
            crate::components::DamageType::Psychic => Color::Blue,
            _ => Color::Red, // Default for physical
        };
        
        // Temporarily change the background color for flash effect
        renderable.bg = flash_color;
    }
    
    fn create_enhanced_combat_log(
        &self,
        target: Entity,
        damage: &DamageInfo,
        stats: &CombatStats,
        name: &Name,
        players: &ReadStorage<Player>,
        monsters: &ReadStorage<Monster>,
        status_effects: &ReadStorage<StatusEffects>,
        gamelog: &mut GameLog,
    ) {
        let target_name = &name.name;
        let is_player = players.contains(target);
        let is_monster = monsters.contains(target);
        
        // Create detailed damage description
        let damage_desc = if damage.is_critical {
            format!("CRITICAL {} damage", damage.damage_type.name().to_uppercase())
        } else {
            format!("{} damage", damage.damage_type.name())
        };
        
        // Add damage amount with visual emphasis
        let damage_text = if damage.is_critical {
            format!("*** {} takes {} {}! ***", target_name, damage.base_damage, damage_desc)
        } else {
            format!("{} takes {} {}!", target_name, damage.base_damage, damage_desc)
        };
        
        gamelog.add_entry(damage_text);
        
        // Add health status information
        let health_percentage = (stats.hp as f32 / stats.max_hp as f32) * 100.0;
        let health_status = match health_percentage as i32 {
            0..=10 => "critically wounded",
            11..=25 => "severely wounded", 
            26..=50 => "badly wounded",
            51..=75 => "wounded",
            76..=90 => "lightly wounded",
            _ => "healthy",
        };
        
        if health_percentage <= 25.0 {
            gamelog.add_entry(format!("{} is {}! ({} HP remaining)", 
                target_name, health_status, stats.hp));
        }
        
        // Add status effect notifications
        if let Some(effects) = status_effects.get(target) {
            for effect in &effects.effects {
                if effect.duration == effect.magnitude { // Just applied
                    gamelog.add_entry(format!("{} is affected by {}!", 
                        target_name, effect.effect_type.name()));
                }
            }
        }
        
        // Add special messages for player
        if is_player {
            if stats.hp <= 0 {
                gamelog.add_entry("*** YOU HAVE BEEN DEFEATED! ***".to_string());
            } else if health_percentage <= 10.0 {
                gamelog.add_entry("*** WARNING: You are near death! ***".to_string());
            }
        }
    }
    
    fn update_feedback_effects(
        &self,
        combat_feedback: &mut WriteStorage<CombatFeedback>,
        renderables: &mut WriteStorage<Renderable>,
    ) {
        let mut expired_feedback = Vec::new();
        
        for (entity, mut feedback) in (&entities, &mut combat_feedback).join() {
            feedback.duration -= 0.016; // Assuming ~60 FPS
            
            // Update animation
            match feedback.animation_type {
                AnimationType::FloatUp => {
                    feedback.position.offset_y -= 0.02; // Float upward
                    feedback.position.offset_x *= 0.98; // Slight horizontal dampening
                },
                AnimationType::CriticalBounce => {
                    let progress = 1.0 - (feedback.duration / feedback.max_duration);
                    feedback.position.offset_y = -0.5 - (progress * 0.5).sin() * 0.3;
                },
                AnimationType::Shake => {
                    // Screen shake would be handled by the rendering system
                },
                AnimationType::Flash => {
                    // Flash effect fading
                    if let Some(renderable) = renderables.get_mut(entity) {
                        // Gradually restore original background color
                        if feedback.duration <= 0.1 {
                            renderable.bg = Color::Black; // Restore original
                        }
                    }
                },
            }
            
            // Mark expired feedback for removal
            if feedback.duration <= 0.0 {
                expired_feedback.push(entity);
                
                // Restore original colors for flash effects
                if let Some(renderable) = renderables.get_mut(entity) {
                    if matches!(feedback.animation_type, AnimationType::Flash) {
                        renderable.bg = Color::Black;
                    }
                }
            }
        }
        
        // Remove expired feedback
        for entity in expired_feedback {
            combat_feedback.remove(entity);
        }
    }
    
    fn get_damage_color(&self, damage: &DamageInfo) -> Color {
        if damage.is_critical {
            Color::Yellow // Critical hits are always yellow
        } else {
            match damage.damage_type {
                crate::components::DamageType::Physical => Color::White,
                crate::components::DamageType::Fire => Color::Red,
                crate::components::DamageType::Ice => Color::Cyan,
                crate::components::DamageType::Lightning => Color::Yellow,
                crate::components::DamageType::Poison => Color::Green,
                crate::components::DamageType::Holy => Color::White,
                crate::components::DamageType::Dark => Color::Magenta,
                crate::components::DamageType::Psychic => Color::Blue,
            }
        }
    }
}

// Combat feedback components
#[derive(Component, Debug, Clone)]
#[storage(VecStorage)]
pub struct CombatFeedback {
    pub feedback_type: CombatFeedbackType,
    pub position: FloatingPosition,
    pub duration: f32,
    pub max_duration: f32,
    pub color: Color,
    pub animation_type: AnimationType,
}

#[derive(Debug, Clone)]
pub enum CombatFeedbackType {
    DamageText {
        damage: i32,
        damage_type: crate::components::DamageType,
        is_critical: bool,
    },
    HealingText {
        healing: i32,
    },
    StatusText {
        text: String,
    },
    ScreenShake {
        intensity: ShakeIntensity,
    },
    SoundEffect {
        sound_type: SoundEffectType,
    },
}

#[derive(Debug, Clone)]
pub struct FloatingPosition {
    pub x: f32,
    pub y: f32,
    pub offset_x: f32,
    pub offset_y: f32,
}

impl Default for FloatingPosition {
    fn default() -> Self {
        FloatingPosition {
            x: 0.0,
            y: 0.0,
            offset_x: 0.0,
            offset_y: 0.0,
        }
    }
}

#[derive(Debug, Clone)]
pub enum AnimationType {
    FloatUp,
    CriticalBounce,
    Shake,
    Flash,
    Pulse,
}

#[derive(Debug, Clone)]
pub enum ShakeIntensity {
    Light,
    Medium,
    Heavy,
}

#[derive(Debug, Clone)]
pub enum SoundEffectType {
    Hit,
    CriticalHit,
    Block,
    Evade,
    Death,
    Heal,
    StatusEffect,
}