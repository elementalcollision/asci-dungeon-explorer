use crossterm::style::Color;
use specs::{World, Entity};
use std::collections::VecDeque;
use std::time::{Duration, Instant};
use crate::ui::ui_components::{UIRenderCommand, UIPanel};

/// Types of feedback that can be displayed
#[derive(Debug, Clone, PartialEq)]
pub enum FeedbackType {
    Success,
    Warning,
    Error,
    Info,
    Combat,
    Experience,
    ItemPickup,
    LevelUp,
    Achievement,
    Custom(String),
}

impl FeedbackType {
    pub fn color(&self) -> Color {
        match self {
            FeedbackType::Success => Color::Green,
            FeedbackType::Warning => Color::Yellow,
            FeedbackType::Error => Color::Red,
            FeedbackType::Info => Color::Cyan,
            FeedbackType::Combat => Color::Red,
            FeedbackType::Experience => Color::Blue,
            FeedbackType::ItemPickup => Color::Yellow,
            FeedbackType::LevelUp => Color::Magenta,
            FeedbackType::Achievement => Color::Gold,
            FeedbackType::Custom(_) => Color::White,
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            FeedbackType::Success => "✓",
            FeedbackType::Warning => "⚠",
            FeedbackType::Error => "✗",
            FeedbackType::Info => "ℹ",
            FeedbackType::Combat => "⚔",
            FeedbackType::Experience => "★",
            FeedbackType::ItemPickup => "📦",
            FeedbackType::LevelUp => "↑",
            FeedbackType::Achievement => "🏆",
            FeedbackType::Custom(_) => "•",
        }
    }
}

/// Visual feedback effects
#[derive(Debug, Clone)]
pub enum VisualEffect {
    Flash {
        color: Color,
        duration: Duration,
        intensity: f32,
    },
    Shake {
        intensity: f32,
        duration: Duration,
    },
    Pulse {
        color: Color,
        duration: Duration,
        frequency: f32,
    },
    Slide {
        from_x: i32,
        from_y: i32,
        to_x: i32,
        to_y: i32,
        duration: Duration,
    },
    FadeIn {
        duration: Duration,
    },
    FadeOut {
        duration: Duration,
    },
    Bounce {
        height: i32,
        duration: Duration,
    },
}

/// Notification message
#[derive(Debug, Clone)]
pub struct Notification {
    pub id: u32,
    pub message: String,
    pub feedback_type: FeedbackType,
    pub created_at: Instant,
    pub duration: Duration,
    pub position: NotificationPosition,
    pub priority: NotificationPriority,
    pub visual_effect: Option<VisualEffect>,
    pub sound_cue: Option<SoundCue>,
    pub persistent: bool,
    pub dismissible: bool,
}

/// Notification positioning
#[derive(Debug, Clone, PartialEq)]
pub enum NotificationPosition {
    TopLeft,
    TopCenter,
    TopRight,
    MiddleLeft,
    MiddleCenter,
    MiddleRight,
    BottomLeft,
    BottomCenter,
    BottomRight,
    FloatingText(i32, i32), // x, y coordinates
}

/// Notification priority levels
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum NotificationPriority {
    Low,
    Normal,
    High,
    Critical,
}

/// Sound cue types
#[derive(Debug, Clone, PartialEq)]
pub enum SoundCue {
    Beep,
    Chime,
    Alert,
    Success,
    Error,
    Combat,
    Pickup,
    LevelUp,
    Achievement,
    Custom(String),
}

/// Floating text animation
#[derive(Debug, Clone)]
pub struct FloatingText {
    pub text: String,
    pub x: f32,
    pub y: f32,
    pub velocity_x: f32,
    pub velocity_y: f32,
    pub color: Color,
    pub created_at: Instant,
    pub duration: Duration,
    pub fade_out: bool,
}

/// Screen shake effect
#[derive(Debug, Clone)]
pub struct ScreenShake {
    pub intensity: f32,
    pub duration: Duration,
    pub started_at: Instant,
    pub offset_x: i32,
    pub offset_y: i32,
}

/// UI feedback system
pub struct UIFeedbackSystem {
    pub notifications: VecDeque<Notification>,
    pub floating_texts: Vec<FloatingText>,
    pub screen_shake: Option<ScreenShake>,
    pub visual_effects: Vec<(Entity, VisualEffect, Instant)>,
    pub next_notification_id: u32,
    pub max_notifications: usize,
    pub sound_enabled: bool,
    pub visual_effects_enabled: bool,
    pub accessibility_mode: bool,
    pub high_contrast: bool,
    pub reduced_motion: bool,
    pub screen_reader_support: bool,
}

impl UIFeedbackSystem {
    pub fn new() -> Self {
        UIFeedbackSystem {
            notifications: VecDeque::new(),
            floating_texts: Vec::new(),
            screen_shake: None,
            visual_effects: Vec::new(),
            next_notification_id: 1,
            max_notifications: 10,
            sound_enabled: true,
            visual_effects_enabled: true,
            accessibility_mode: false,
            high_contrast: false,
            reduced_motion: false,
            screen_reader_support: false,
        }
    }

    /// Add a notification
    pub fn add_notification(&mut self, message: String, feedback_type: FeedbackType) -> u32 {
        let notification = Notification {
            id: self.next_notification_id,
            message,
            feedback_type,
            created_at: Instant::now(),
            duration: Duration::from_secs(3),
            position: NotificationPosition::TopRight,
            priority: NotificationPriority::Normal,
            visual_effect: None,
            sound_cue: None,
            persistent: false,
            dismissible: true,
        };

        self.next_notification_id += 1;
        self.notifications.push_back(notification);

        // Remove old notifications if we exceed the limit
        while self.notifications.len() > self.max_notifications {
            self.notifications.pop_front();
        }

        notification.id
    }

    /// Add a notification with custom settings
    pub fn add_custom_notification(&mut self, mut notification: Notification) -> u32 {
        notification.id = self.next_notification_id;
        notification.created_at = Instant::now();
        
        self.next_notification_id += 1;

        // Insert based on priority
        let insert_pos = self.notifications
            .iter()
            .position(|n| n.priority < notification.priority)
            .unwrap_or(self.notifications.len());

        self.notifications.insert(insert_pos, notification);

        // Remove old notifications if we exceed the limit
        while self.notifications.len() > self.max_notifications {
            if let Some(removed) = self.notifications.pop_front() {
                if removed.priority >= NotificationPriority::High {
                    // Don't remove high priority notifications, remove from back instead
                    self.notifications.push_front(removed);
                    self.notifications.pop_back();
                }
            }
        }

        notification.id
    }

    /// Remove a notification by ID
    pub fn remove_notification(&mut self, id: u32) {
        self.notifications.retain(|n| n.id != id);
    }

    /// Add floating text
    pub fn add_floating_text(&mut self, text: String, x: i32, y: i32, color: Color) {
        if self.reduced_motion {
            // In reduced motion mode, show as static notification instead
            self.add_notification(text, FeedbackType::Info);
            return;
        }

        let floating_text = FloatingText {
            text,
            x: x as f32,
            y: y as f32,
            velocity_x: 0.0,
            velocity_y: -20.0, // Float upward
            color,
            created_at: Instant::now(),
            duration: Duration::from_secs(2),
            fade_out: true,
        };

        self.floating_texts.push(floating_text);
    }

    /// Add screen shake effect
    pub fn add_screen_shake(&mut self, intensity: f32, duration: Duration) {
        if self.reduced_motion {
            return; // Skip screen shake in reduced motion mode
        }

        self.screen_shake = Some(ScreenShake {
            intensity,
            duration,
            started_at: Instant::now(),
            offset_x: 0,
            offset_y: 0,
        });
    }

    /// Add visual effect to entity
    pub fn add_visual_effect(&mut self, entity: Entity, effect: VisualEffect) {
        if !self.visual_effects_enabled || self.reduced_motion {
            return;
        }

        self.visual_effects.push((entity, effect, Instant::now()));
    }

    /// Play sound cue
    pub fn play_sound(&self, sound_cue: SoundCue) {
        if !self.sound_enabled {
            return;
        }

        // In a real implementation, this would interface with an audio system
        // For now, we'll just log the sound that would be played
        match sound_cue {
            SoundCue::Beep => println!("🔊 Beep"),
            SoundCue::Chime => println!("🔊 Chime"),
            SoundCue::Alert => println!("🔊 Alert"),
            SoundCue::Success => println!("🔊 Success"),
            SoundCue::Error => println!("🔊 Error"),
            SoundCue::Combat => println!("🔊 Combat"),
            SoundCue::Pickup => println!("🔊 Pickup"),
            SoundCue::LevelUp => println!("🔊 Level Up"),
            SoundCue::Achievement => println!("🔊 Achievement"),
            SoundCue::Custom(name) => println!("🔊 {}", name),
        }
    }

    /// Update the feedback system
    pub fn update(&mut self, delta_time: f32) {
        let now = Instant::now();

        // Update notifications
        self.notifications.retain(|notification| {
            if notification.persistent {
                true
            } else {
                now.duration_since(notification.created_at) < notification.duration
            }
        });

        // Update floating texts
        for floating_text in &mut self.floating_texts {
            floating_text.x += floating_text.velocity_x * delta_time;
            floating_text.y += floating_text.velocity_y * delta_time;
        }

        self.floating_texts.retain(|floating_text| {
            now.duration_since(floating_text.created_at) < floating_text.duration
        });

        // Update screen shake
        if let Some(ref mut shake) = self.screen_shake {
            let elapsed = now.duration_since(shake.started_at);
            if elapsed >= shake.duration {
                self.screen_shake = None;
            } else {
                let progress = elapsed.as_secs_f32() / shake.duration.as_secs_f32();
                let intensity = shake.intensity * (1.0 - progress);
                
                // Generate random shake offset
                use std::collections::hash_map::DefaultHasher;
                use std::hash::{Hash, Hasher};
                
                let mut hasher = DefaultHasher::new();
                elapsed.as_nanos().hash(&mut hasher);
                let hash = hasher.finish();
                
                shake.offset_x = (((hash % 100) as f32 - 50.0) * intensity * 0.1) as i32;
                shake.offset_y = ((((hash >> 8) % 100) as f32 - 50.0) * intensity * 0.1) as i32;
            }
        }

        // Update visual effects
        self.visual_effects.retain(|(_, effect, started_at)| {
            let duration = match effect {
                VisualEffect::Flash { duration, .. } => *duration,
                VisualEffect::Shake { duration, .. } => *duration,
                VisualEffect::Pulse { duration, .. } => *duration,
                VisualEffect::Slide { duration, .. } => *duration,
                VisualEffect::FadeIn { duration } => *duration,
                VisualEffect::FadeOut { duration } => *duration,
                VisualEffect::Bounce { duration, .. } => *duration,
            };

            now.duration_since(*started_at) < duration
        });
    }

    /// Render notifications
    pub fn render_notifications(&self, screen_width: i32, screen_height: i32) -> Vec<UIRenderCommand> {
        let mut commands = Vec::new();

        // Group notifications by position
        let mut position_groups: std::collections::HashMap<NotificationPosition, Vec<&Notification>> = 
            std::collections::HashMap::new();

        for notification in &self.notifications {
            position_groups.entry(notification.position.clone())
                .or_insert_with(Vec::new)
                .push(notification);
        }

        // Render each position group
        for (position, notifications) in position_groups {
            commands.extend(self.render_notification_group(&notifications, &position, screen_width, screen_height));
        }

        commands
    }

    fn render_notification_group(
        &self,
        notifications: &[&Notification],
        position: &NotificationPosition,
        screen_width: i32,
        screen_height: i32,
    ) -> Vec<UIRenderCommand> {
        let mut commands = Vec::new();
        let notification_height = 3;
        let notification_width = 40;

        let (base_x, base_y) = self.calculate_position(position, notification_width, notification_height, screen_width, screen_height);

        for (i, notification) in notifications.iter().enumerate() {
            let y = base_y + (i as i32 * notification_height);
            
            // Skip if notification would be off-screen
            if y + notification_height > screen_height {
                break;
            }

            commands.extend(self.render_single_notification(notification, base_x, y, notification_width));
        }

        commands
    }

    fn render_single_notification(&self, notification: &Notification, x: i32, y: i32, width: i32) -> Vec<UIRenderCommand> {
        let mut commands = Vec::new();

        // Calculate colors based on accessibility settings
        let (fg_color, bg_color) = if self.high_contrast {
            match notification.feedback_type {
                FeedbackType::Error => (Color::White, Color::Red),
                FeedbackType::Success => (Color::Black, Color::Green),
                FeedbackType::Warning => (Color::Black, Color::Yellow),
                _ => (Color::White, Color::Black),
            }
        } else {
            (notification.feedback_type.color(), Color::DarkGrey)
        };

        // Notification panel
        let panel = UIPanel::new(
            "".to_string(),
            x,
            y,
            width,
            3,
        ).with_colors(fg_color, bg_color, fg_color);

        commands.extend(panel.render());

        // Icon and message
        let icon = if self.accessibility_mode {
            format!("[{}] ", notification.feedback_type.icon())
        } else {
            format!("{} ", notification.feedback_type.icon())
        };

        let message_text = format!("{}{}", icon, notification.message);
        let truncated_message = if message_text.len() > (width - 2) as usize {
            format!("{}...", &message_text[..(width - 5) as usize])
        } else {
            message_text
        };

        commands.push(UIRenderCommand::DrawText {
            x: x + 1,
            y: y + 1,
            text: truncated_message,
            fg: fg_color,
            bg: bg_color,
        });

        // Progress bar for timed notifications
        if !notification.persistent {
            let elapsed = notification.created_at.elapsed();
            let progress = elapsed.as_secs_f32() / notification.duration.as_secs_f32();
            let progress_width = ((width - 2) as f32 * (1.0 - progress)) as usize;
            
            if progress_width > 0 {
                let progress_bar = "█".repeat(progress_width);
                commands.push(UIRenderCommand::DrawText {
                    x: x + 1,
                    y: y + 2,
                    text: progress_bar,
                    fg: fg_color,
                    bg: bg_color,
                });
            }
        }

        commands
    }

    /// Render floating texts
    pub fn render_floating_texts(&self) -> Vec<UIRenderCommand> {
        let mut commands = Vec::new();

        for floating_text in &self.floating_texts {
            let elapsed = floating_text.created_at.elapsed().as_secs_f32();
            let progress = elapsed / floating_text.duration.as_secs_f32();
            
            let alpha = if floating_text.fade_out {
                1.0 - progress
            } else {
                1.0
            };

            // Adjust color based on alpha (simplified)
            let color = if alpha < 0.5 {
                Color::DarkGrey
            } else {
                floating_text.color
            };

            commands.push(UIRenderCommand::DrawText {
                x: floating_text.x as i32,
                y: floating_text.y as i32,
                text: floating_text.text.clone(),
                fg: color,
                bg: Color::Black,
            });
        }

        commands
    }

    /// Get screen shake offset
    pub fn get_screen_shake_offset(&self) -> (i32, i32) {
        if let Some(ref shake) = self.screen_shake {
            (shake.offset_x, shake.offset_y)
        } else {
            (0, 0)
        }
    }

    fn calculate_position(
        &self,
        position: &NotificationPosition,
        width: i32,
        height: i32,
        screen_width: i32,
        screen_height: i32,
    ) -> (i32, i32) {
        match position {
            NotificationPosition::TopLeft => (2, 2),
            NotificationPosition::TopCenter => ((screen_width - width) / 2, 2),
            NotificationPosition::TopRight => (screen_width - width - 2, 2),
            NotificationPosition::MiddleLeft => (2, (screen_height - height) / 2),
            NotificationPosition::MiddleCenter => ((screen_width - width) / 2, (screen_height - height) / 2),
            NotificationPosition::MiddleRight => (screen_width - width - 2, (screen_height - height) / 2),
            NotificationPosition::BottomLeft => (2, screen_height - height - 2),
            NotificationPosition::BottomCenter => ((screen_width - width) / 2, screen_height - height - 2),
            NotificationPosition::BottomRight => (screen_width - width - 2, screen_height - height - 2),
            NotificationPosition::FloatingText(x, y) => (*x, *y),
        }
    }

    /// Enable accessibility features
    pub fn set_accessibility_mode(&mut self, enabled: bool) {
        self.accessibility_mode = enabled;
        if enabled {
            self.high_contrast = true;
            self.reduced_motion = true;
            self.screen_reader_support = true;
        }
    }

    /// Set high contrast mode
    pub fn set_high_contrast(&mut self, enabled: bool) {
        self.high_contrast = enabled;
    }

    /// Set reduced motion mode
    pub fn set_reduced_motion(&mut self, enabled: bool) {
        self.reduced_motion = enabled;
        if enabled {
            self.visual_effects_enabled = false;
            self.screen_shake = None;
            self.floating_texts.clear();
        } else {
            self.visual_effects_enabled = true;
        }
    }

    /// Set screen reader support
    pub fn set_screen_reader_support(&mut self, enabled: bool) {
        self.screen_reader_support = enabled;
    }

    /// Get accessibility description for screen readers
    pub fn get_accessibility_description(&self) -> String {
        if !self.screen_reader_support {
            return String::new();
        }

        let mut descriptions = Vec::new();

        // Describe active notifications
        for notification in &self.notifications {
            let priority_text = match notification.priority {
                NotificationPriority::Critical => "Critical: ",
                NotificationPriority::High => "Important: ",
                _ => "",
            };

            let type_text = match notification.feedback_type {
                FeedbackType::Error => "Error: ",
                FeedbackType::Warning => "Warning: ",
                FeedbackType::Success => "Success: ",
                _ => "",
            };

            descriptions.push(format!("{}{}{}", priority_text, type_text, notification.message));
        }

        descriptions.join(". ")
    }

    /// Quick feedback methods for common actions
    pub fn show_success(&mut self, message: String) {
        self.add_notification(message, FeedbackType::Success);
        self.play_sound(SoundCue::Success);
    }

    pub fn show_error(&mut self, message: String) {
        self.add_notification(message, FeedbackType::Error);
        self.play_sound(SoundCue::Error);
    }

    pub fn show_warning(&mut self, message: String) {
        self.add_notification(message, FeedbackType::Warning);
        self.play_sound(SoundCue::Alert);
    }

    pub fn show_info(&mut self, message: String) {
        self.add_notification(message, FeedbackType::Info);
    }

    pub fn show_combat_damage(&mut self, damage: i32, x: i32, y: i32) {
        self.add_floating_text(format!("-{}", damage), x, y, Color::Red);
        self.play_sound(SoundCue::Combat);
    }

    pub fn show_healing(&mut self, amount: i32, x: i32, y: i32) {
        self.add_floating_text(format!("+{}", amount), x, y, Color::Green);
    }

    pub fn show_experience_gain(&mut self, exp: i32) {
        self.add_notification(format!("Gained {} experience", exp), FeedbackType::Experience);
        self.play_sound(SoundCue::Success);
    }

    pub fn show_level_up(&mut self, new_level: i32) {
        let mut notification = Notification {
            id: 0, // Will be set by add_custom_notification
            message: format!("Level Up! You are now level {}", new_level),
            feedback_type: FeedbackType::LevelUp,
            created_at: Instant::now(),
            duration: Duration::from_secs(5),
            position: NotificationPosition::MiddleCenter,
            priority: NotificationPriority::High,
            visual_effect: Some(VisualEffect::Pulse {
                color: Color::Magenta,
                duration: Duration::from_secs(2),
                frequency: 2.0,
            }),
            sound_cue: Some(SoundCue::LevelUp),
            persistent: false,
            dismissible: true,
        };

        self.add_custom_notification(notification);
        self.add_screen_shake(5.0, Duration::from_millis(500));
    }

    pub fn show_item_pickup(&mut self, item_name: String) {
        self.add_notification(format!("Picked up: {}", item_name), FeedbackType::ItemPickup);
        self.play_sound(SoundCue::Pickup);
    }

    pub fn show_achievement(&mut self, achievement_name: String, description: String) {
        let mut notification = Notification {
            id: 0,
            message: format!("Achievement Unlocked: {} - {}", achievement_name, description),
            feedback_type: FeedbackType::Achievement,
            created_at: Instant::now(),
            duration: Duration::from_secs(8),
            position: NotificationPosition::TopCenter,
            priority: NotificationPriority::High,
            visual_effect: Some(VisualEffect::Bounce {
                height: 10,
                duration: Duration::from_secs(1),
            }),
            sound_cue: Some(SoundCue::Achievement),
            persistent: false,
            dismissible: true,
        };

        self.add_custom_notification(notification);
        self.play_sound(SoundCue::Achievement);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feedback_system_creation() {
        let feedback_system = UIFeedbackSystem::new();
        
        assert!(feedback_system.notifications.is_empty());
        assert!(feedback_system.floating_texts.is_empty());
        assert!(feedback_system.screen_shake.is_none());
        assert_eq!(feedback_system.next_notification_id, 1);
        assert!(feedback_system.sound_enabled);
        assert!(feedback_system.visual_effects_enabled);
    }

    #[test]
    fn test_add_notification() {
        let mut feedback_system = UIFeedbackSystem::new();
        
        let id = feedback_system.add_notification("Test message".to_string(), FeedbackType::Info);
        
        assert_eq!(id, 1);
        assert_eq!(feedback_system.notifications.len(), 1);
        assert_eq!(feedback_system.next_notification_id, 2);
        
        let notification = &feedback_system.notifications[0];
        assert_eq!(notification.message, "Test message");
        assert_eq!(notification.feedback_type, FeedbackType::Info);
    }

    #[test]
    fn test_notification_priority() {
        let mut feedback_system = UIFeedbackSystem::new();
        
        // Add normal priority notification
        feedback_system.add_notification("Normal".to_string(), FeedbackType::Info);
        
        // Add high priority notification
        let mut high_priority = Notification {
            id: 0,
            message: "High Priority".to_string(),
            feedback_type: FeedbackType::Error,
            created_at: Instant::now(),
            duration: Duration::from_secs(3),
            position: NotificationPosition::TopRight,
            priority: NotificationPriority::High,
            visual_effect: None,
            sound_cue: None,
            persistent: false,
            dismissible: true,
        };
        
        feedback_system.add_custom_notification(high_priority);
        
        // High priority should be first
        assert_eq!(feedback_system.notifications[0].message, "High Priority");
        assert_eq!(feedback_system.notifications[1].message, "Normal");
    }

    #[test]
    fn test_max_notifications() {
        let mut feedback_system = UIFeedbackSystem::new();
        feedback_system.max_notifications = 3;
        
        // Add more notifications than the limit
        for i in 1..=5 {
            feedback_system.add_notification(format!("Message {}", i), FeedbackType::Info);
        }
        
        // Should only keep the last 3
        assert_eq!(feedback_system.notifications.len(), 3);
        assert_eq!(feedback_system.notifications[0].message, "Message 3");
        assert_eq!(feedback_system.notifications[2].message, "Message 5");
    }

    #[test]
    fn test_remove_notification() {
        let mut feedback_system = UIFeedbackSystem::new();
        
        let id1 = feedback_system.add_notification("Message 1".to_string(), FeedbackType::Info);
        let id2 = feedback_system.add_notification("Message 2".to_string(), FeedbackType::Info);
        
        assert_eq!(feedback_system.notifications.len(), 2);
        
        feedback_system.remove_notification(id1);
        
        assert_eq!(feedback_system.notifications.len(), 1);
        assert_eq!(feedback_system.notifications[0].message, "Message 2");
    }

    #[test]
    fn test_floating_text() {
        let mut feedback_system = UIFeedbackSystem::new();
        
        feedback_system.add_floating_text("Damage!".to_string(), 10, 20, Color::Red);
        
        assert_eq!(feedback_system.floating_texts.len(), 1);
        
        let floating_text = &feedback_system.floating_texts[0];
        assert_eq!(floating_text.text, "Damage!");
        assert_eq!(floating_text.x, 10.0);
        assert_eq!(floating_text.y, 20.0);
        assert_eq!(floating_text.color, Color::Red);
    }

    #[test]
    fn test_screen_shake() {
        let mut feedback_system = UIFeedbackSystem::new();
        
        feedback_system.add_screen_shake(5.0, Duration::from_millis(500));
        
        assert!(feedback_system.screen_shake.is_some());
        
        let shake = feedback_system.screen_shake.as_ref().unwrap();
        assert_eq!(shake.intensity, 5.0);
        assert_eq!(shake.duration, Duration::from_millis(500));
    }

    #[test]
    fn test_accessibility_mode() {
        let mut feedback_system = UIFeedbackSystem::new();
        
        assert!(!feedback_system.accessibility_mode);
        assert!(!feedback_system.high_contrast);
        assert!(!feedback_system.reduced_motion);
        
        feedback_system.set_accessibility_mode(true);
        
        assert!(feedback_system.accessibility_mode);
        assert!(feedback_system.high_contrast);
        assert!(feedback_system.reduced_motion);
        assert!(feedback_system.screen_reader_support);
    }

    #[test]
    fn test_reduced_motion() {
        let mut feedback_system = UIFeedbackSystem::new();
        
        // Add some effects
        feedback_system.add_screen_shake(5.0, Duration::from_millis(500));
        feedback_system.add_floating_text("Test".to_string(), 0, 0, Color::White);
        
        assert!(feedback_system.screen_shake.is_some());
        assert_eq!(feedback_system.floating_texts.len(), 1);
        
        // Enable reduced motion
        feedback_system.set_reduced_motion(true);
        
        assert!(feedback_system.screen_shake.is_none());
        assert!(feedback_system.floating_texts.is_empty());
        assert!(!feedback_system.visual_effects_enabled);
    }

    #[test]
    fn test_feedback_type_properties() {
        assert_eq!(FeedbackType::Success.color(), Color::Green);
        assert_eq!(FeedbackType::Error.color(), Color::Red);
        assert_eq!(FeedbackType::Warning.color(), Color::Yellow);
        
        assert_eq!(FeedbackType::Success.icon(), "✓");
        assert_eq!(FeedbackType::Error.icon(), "✗");
        assert_eq!(FeedbackType::Warning.icon(), "⚠");
    }

    #[test]
    fn test_quick_feedback_methods() {
        let mut feedback_system = UIFeedbackSystem::new();
        
        feedback_system.show_success("Success!".to_string());
        feedback_system.show_error("Error!".to_string());
        feedback_system.show_warning("Warning!".to_string());
        
        assert_eq!(feedback_system.notifications.len(), 3);
        
        // Check that notifications have correct types
        let types: Vec<_> = feedback_system.notifications.iter()
            .map(|n| &n.feedback_type)
            .collect();
        
        assert!(types.contains(&&FeedbackType::Success));
        assert!(types.contains(&&FeedbackType::Error));
        assert!(types.contains(&&FeedbackType::Warning));
    }

    #[test]
    fn test_notification_positioning() {
        let feedback_system = UIFeedbackSystem::new();
        
        let (x, y) = feedback_system.calculate_position(
            &NotificationPosition::TopLeft,
            40, 3, 100, 50
        );
        assert_eq!((x, y), (2, 2));
        
        let (x, y) = feedback_system.calculate_position(
            &NotificationPosition::TopCenter,
            40, 3, 100, 50
        );
        assert_eq!((x, y), (30, 2)); // (100 - 40) / 2 = 30
        
        let (x, y) = feedback_system.calculate_position(
            &NotificationPosition::BottomRight,
            40, 3, 100, 50
        );
        assert_eq!((x, y), (58, 45)); // 100 - 40 - 2 = 58, 50 - 3 - 2 = 45
    }

    #[test]
    fn test_update_removes_expired_notifications() {
        let mut feedback_system = UIFeedbackSystem::new();
        
        // Add notification with very short duration
        let mut notification = Notification {
            id: 1,
            message: "Short lived".to_string(),
            feedback_type: FeedbackType::Info,
            created_at: Instant::now() - Duration::from_secs(10), // Already expired
            duration: Duration::from_secs(1),
            position: NotificationPosition::TopRight,
            priority: NotificationPriority::Normal,
            visual_effect: None,
            sound_cue: None,
            persistent: false,
            dismissible: true,
        };
        
        feedback_system.notifications.push_back(notification);
        assert_eq!(feedback_system.notifications.len(), 1);
        
        // Update should remove expired notification
        feedback_system.update(0.1);
        assert_eq!(feedback_system.notifications.len(), 0);
    }

    #[test]
    fn test_persistent_notifications() {
        let mut feedback_system = UIFeedbackSystem::new();
        
        // Add persistent notification
        let mut notification = Notification {
            id: 1,
            message: "Persistent".to_string(),
            feedback_type: FeedbackType::Info,
            created_at: Instant::now() - Duration::from_secs(10), // Already "expired"
            duration: Duration::from_secs(1),
            position: NotificationPosition::TopRight,
            priority: NotificationPriority::Normal,
            visual_effect: None,
            sound_cue: None,
            persistent: true, // This should keep it alive
            dismissible: true,
        };
        
        feedback_system.notifications.push_back(notification);
        assert_eq!(feedback_system.notifications.len(), 1);
        
        // Update should NOT remove persistent notification
        feedback_system.update(0.1);
        assert_eq!(feedback_system.notifications.len(), 1);
    }
}