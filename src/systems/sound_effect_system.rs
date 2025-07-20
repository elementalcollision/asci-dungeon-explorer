use specs::{System, WriteStorage, ReadStorage, Entities, Entity, Join, Write};
use crate::components::{
    CombatFeedback, CombatFeedbackType, SoundEffectType, DamageInfo, 
    CombatStats, Name, Player, DefenseResult
};
use crate::resources::GameLog;

pub struct SoundEffectSystem {}

impl<'a> System<'a> for SoundEffectSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, CombatFeedback>,
        ReadStorage<'a, DamageInfo>,
        ReadStorage<'a, CombatStats>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, Player>,
        Write<'a, GameLog>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, mut combat_feedback, damage_info, combat_stats, names, players, mut gamelog) = data;

        // Process damage events for sound effects
        for (entity, damage, stats, name) in (&entities, &damage_info, &combat_stats, &names).join() {
            let sound_type = if damage.is_critical {
                SoundEffectType::CriticalHit
            } else {
                SoundEffectType::Hit
            };
            
            // Create sound effect feedback
            let sound_feedback = CombatFeedback {
                feedback_type: CombatFeedbackType::SoundEffect { sound_type: sound_type.clone() },
                position: crate::components::FloatingPosition::default(),
                duration: 0.1, // Short duration for sound triggers
                max_duration: 0.1,
                color: crossterm::style::Color::White,
                animation_type: crate::components::AnimationType::Flash,
            };
            
            // Insert sound feedback (using a unique entity for sound effects)
            let sound_entity = Entity::from_raw(entity.id() + 1000000); // Offset to avoid conflicts
            combat_feedback.insert(sound_entity, sound_feedback)
                .expect("Failed to insert sound feedback");
            
            // Log sound effect (in a real implementation, this would trigger actual audio)
            self.play_sound_effect(&sound_type, &mut gamelog);
            
            // Check for death sound
            if stats.hp <= 0 {
                let death_feedback = CombatFeedback {
                    feedback_type: CombatFeedbackType::SoundEffect { 
                        sound_type: SoundEffectType::Death 
                    },
                    position: crate::components::FloatingPosition::default(),
                    duration: 0.1,
                    max_duration: 0.1,
                    color: crossterm::style::Color::White,
                    animation_type: crate::components::AnimationType::Flash,
                };
                
                let death_sound_entity = Entity::from_raw(entity.id() + 2000000);
                combat_feedback.insert(death_sound_entity, death_feedback)
                    .expect("Failed to insert death sound feedback");
                
                self.play_sound_effect(&SoundEffectType::Death, &mut gamelog);
            }
        }
    }
}

impl SoundEffectSystem {
    fn play_sound_effect(&self, sound_type: &SoundEffectType, gamelog: &mut GameLog) {
        // In a real implementation, this would interface with an audio library
        // For now, we'll just log the sound effect
        let sound_description = match sound_type {
            SoundEffectType::Hit => "♪ *THWACK*",
            SoundEffectType::CriticalHit => "♪ *CRITICAL HIT*",
            SoundEffectType::Block => "♪ *CLANG*",
            SoundEffectType::Evade => "♪ *WHOOSH*",
            SoundEffectType::Death => "♪ *DEATH SOUND*",
            SoundEffectType::Heal => "♪ *HEALING CHIME*",
            SoundEffectType::StatusEffect => "♪ *MAGIC SOUND*",
        };
        
        // Add to game log with special formatting
        gamelog.add_entry(format!("{}", sound_description));
    }
}

// System for managing screen shake effects
pub struct ScreenShakeSystem {}

impl<'a> System<'a> for ScreenShakeSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, CombatFeedback>,
        Write<'a, ScreenShakeState>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, mut combat_feedback, mut screen_shake) = data;

        // Process screen shake feedback
        for (entity, feedback) in (&entities, &combat_feedback).join() {
            if let CombatFeedbackType::ScreenShake { intensity } = &feedback.feedback_type {
                // Update screen shake state
                screen_shake.add_shake(intensity.clone(), feedback.duration);
            }
        }
        
        // Update screen shake state
        screen_shake.update();
    }
}

// Resource for managing screen shake state
#[derive(Debug)]
pub struct ScreenShakeState {
    pub current_intensity: f32,
    pub duration: f32,
    pub offset_x: f32,
    pub offset_y: f32,
}

impl ScreenShakeState {
    pub fn new() -> Self {
        ScreenShakeState {
            current_intensity: 0.0,
            duration: 0.0,
            offset_x: 0.0,
            offset_y: 0.0,
        }
    }
    
    pub fn add_shake(&mut self, intensity: crate::components::ShakeIntensity, duration: f32) {
        let intensity_value = intensity.get_offset();
        
        // Add to existing shake or replace if stronger
        if intensity_value > self.current_intensity {
            self.current_intensity = intensity_value;
            self.duration = duration.max(self.duration);
        }
    }
    
    pub fn update(&mut self) {
        if self.duration > 0.0 {
            self.duration -= 0.016; // Assuming ~60 FPS
            
            // Generate random shake offsets
            use crate::resources::RandomNumberGenerator;
            let mut rng = RandomNumberGenerator::new_with_random_seed();
            
            let shake_range = self.current_intensity * (self.duration / 0.3); // Fade out over time
            self.offset_x = (rng.roll_dice(1, 100) as f32 - 50.0) / 50.0 * shake_range;
            self.offset_y = (rng.roll_dice(1, 100) as f32 - 50.0) / 50.0 * shake_range;
            
            // Reduce intensity over time
            self.current_intensity *= 0.95;
        } else {
            self.current_intensity = 0.0;
            self.offset_x = 0.0;
            self.offset_y = 0.0;
        }
    }
    
    pub fn is_shaking(&self) -> bool {
        self.current_intensity > 0.1
    }
}