use std::collections::VecDeque;
use std::time::{Duration, Instant};
use crossterm::style::Color;
use crate::{
    ui::{
        ui_components::{UIComponent, UIRenderCommand, UIPanel, UIText, TextAlignment},
    },
    achievements::achievement_system::{
        AchievementNotification, AchievementRarity, AchievementType,
    },
};

/// Notification display style
#[derive(Debug, Clone, PartialEq)]
pub enum NotificationStyle {
    Popup,
    Toast,
    Banner,
    Minimal,
}

/// Notification animation type
#[derive(Debug, Clone, PartialEq)]
pub enum NotificationAnimation {
    SlideIn,
    FadeIn,
    Bounce,
    None,
}

/// Notification position on screen
#[derive(Debug, Clone, PartialEq)]
pub enum NotificationPosition {
    TopRight,
    TopLeft,
    TopCenter,
    BottomRight,
    BottomLeft,
    BottomCenter,
    Center,
}

/// Configuration for achievement notifications
#[derive(Debug, Clone)]
pub struct NotificationConfig {
    pub style: NotificationStyle,
    pub animation: NotificationAnimation,
    pub position: NotificationPosition,
    pub display_duration: Duration,
    pub max_concurrent: usize,
    pub show_progress: bool,
    pub show_rarity_effects: bool,
    pub sound_enabled: bool,
    pub auto_dismiss: bool,
}

impl Default for NotificationConfig {
    fn default() -> Self {
        NotificationConfig {
            style: NotificationStyle::Popup,
            animation: NotificationAnimation::SlideIn,
            position: NotificationPosition::TopRight,
            display_duration: Duration::from_secs(4),
            max_concurrent: 3,
            show_progress: true,
            show_rarity_effects: true,
            sound_enabled: true,
            auto_dismiss: true,
        }
    }
}

/// Active notification with display state
#[derive(Debug, Clone)]
pub struct ActiveNotification {
    pub notification: AchievementNotification,
    pub created_at: Instant,
    pub animation_progress: f32,
    pub display_progress: f32,
    pub is_dismissed: bool,
    pub hover_time: Duration,
}

impl ActiveNotification {
    pub fn new(notification: AchievementNotification) -> Self {
        ActiveNotification {
            notification,
            created_at: Instant::now(),
            animation_progress: 0.0,
            display_progress: 0.0,
            is_dismissed: false,
            hover_time: Duration::from_secs(0),
        }
    }

    /// Update animation and display progress
    pub fn update(&mut self, delta_time: Duration, config: &NotificationConfig) -> bool {
        if self.is_dismissed {
            return false;
        }

        let elapsed = self.created_at.elapsed();
        let total_duration = config.display_duration;

        // Update display progress (0.0 to 1.0)
        self.display_progress = (elapsed.as_secs_f32() / total_duration.as_secs_f32()).min(1.0);

        // Update animation progress
        match config.animation {
            NotificationAnimation::SlideIn => {
                self.animation_progress = if elapsed < Duration::from_millis(300) {
                    elapsed.as_secs_f32() / 0.3 // 300ms slide-in
                } else if elapsed > total_duration - Duration::from_millis(300) {
                    1.0 - ((elapsed - (total_duration - Duration::from_millis(300))).as_secs_f32() / 0.3)
                } else {
                    1.0
                };
            },
            NotificationAnimation::FadeIn => {
                self.animation_progress = if elapsed < Duration::from_millis(500) {
                    elapsed.as_secs_f32() / 0.5 // 500ms fade-in
                } else if elapsed > total_duration - Duration::from_millis(500) {
                    1.0 - ((elapsed - (total_duration - Duration::from_millis(500))).as_secs_f32() / 0.5)
                } else {
                    1.0
                };
            },
            NotificationAnimation::Bounce => {
                if elapsed < Duration::from_millis(600) {
                    let t = elapsed.as_secs_f32() / 0.6;
                    self.animation_progress = 1.0 - (1.0 - t).powi(2) * (2.0 * std::f32::consts::PI * t).cos().abs();
                } else {
                    self.animation_progress = 1.0;
                }
            },
            NotificationAnimation::None => {
                self.animation_progress = 1.0;
            },
        }

        // Check if notification should be dismissed
        if config.auto_dismiss && elapsed >= total_duration {
            self.is_dismissed = true;
            return false;
        }

        true
    }

    /// Check if notification is expired
    pub fn is_expired(&self, config: &NotificationConfig) -> bool {
        self.is_dismissed || (config.auto_dismiss && self.created_at.elapsed() >= config.display_duration)
    }

    /// Dismiss the notification
    pub fn dismiss(&mut self) {
        self.is_dismissed = true;
    }
}

/// Achievement notification system
pub struct AchievementNotificationSystem {
    config: NotificationConfig,
    active_notifications: VecDeque<ActiveNotification>,
    notification_queue: VecDeque<AchievementNotification>,
    last_update: Instant,
}

impl AchievementNotificationSystem {
    pub fn new(config: NotificationConfig) -> Self {
        AchievementNotificationSystem {
            config,
            active_notifications: VecDeque::new(),
            notification_queue: VecDeque::new(),
            last_update: Instant::now(),
        }
    }

    /// Add a notification to the queue
    pub fn add_notification(&mut self, notification: AchievementNotification) {
        // Check for duplicate notifications
        if !self.notification_queue.iter().any(|n| n.achievement_id == notification.achievement_id) &&
           !self.active_notifications.iter().any(|n| n.notification.achievement_id == notification.achievement_id) {
            self.notification_queue.push_back(notification);
        }
    }

    /// Update the notification system
    pub fn update(&mut self) {
        let now = Instant::now();
        let delta_time = now.duration_since(self.last_update);
        self.last_update = now;

        // Update active notifications
        self.active_notifications.retain_mut(|notification| {
            notification.update(delta_time, &self.config)
        });

        // Remove expired notifications
        self.active_notifications.retain(|notification| {
            !notification.is_expired(&self.config)
        });

        // Add new notifications from queue if there's space
        while self.active_notifications.len() < self.config.max_concurrent && 
              !self.notification_queue.is_empty() {
            if let Some(notification) = self.notification_queue.pop_front() {
                self.active_notifications.push_back(ActiveNotification::new(notification));
            }
        }
    }

    /// Get active notifications for rendering
    pub fn get_active_notifications(&self) -> &VecDeque<ActiveNotification> {
        &self.active_notifications
    }

    /// Clear all notifications
    pub fn clear_all(&mut self) {
        self.active_notifications.clear();
        self.notification_queue.clear();
    }

    /// Dismiss a specific notification
    pub fn dismiss_notification(&mut self, achievement_id: &str) {
        for notification in &mut self.active_notifications {
            if notification.notification.achievement_id == achievement_id {
                notification.dismiss();
                break;
            }
        }
    }

    /// Get notification count
    pub fn get_notification_count(&self) -> usize {
        self.active_notifications.len() + self.notification_queue.len()
    }

    /// Update configuration
    pub fn update_config(&mut self, config: NotificationConfig) {
        self.config = config;
    }

    /// Get current configuration
    pub fn get_config(&self) -> &NotificationConfig {
        &self.config
    }

    /// Check if notifications are enabled
    pub fn is_enabled(&self) -> bool {
        self.config.display_duration > Duration::from_secs(0)
    }

    /// Get rarity-specific effects
    fn get_rarity_effects(&self, rarity: &AchievementRarity) -> (Color, String, f32) {
        match rarity {
            AchievementRarity::Common => (Color::White, "".to_string(), 1.0),
            AchievementRarity::Uncommon => (Color::Green, "✨".to_string(), 1.1),
            AchievementRarity::Rare => (Color::Blue, "⭐".to_string(), 1.2),
            AchievementRarity::Epic => (Color::Magenta, "💫".to_string(), 1.3),
            AchievementRarity::Legendary => (Color::Yellow, "🌟".to_string(), 1.5),
        }
    }

    /// Calculate notification position
    fn calculate_position(&self, index: usize, notification_height: i32) -> (i32, i32) {
        let spacing = notification_height + 1;
        let offset = index as i32 * spacing;

        match self.config.position {
            NotificationPosition::TopRight => (50, 2 + offset),
            NotificationPosition::TopLeft => (2, 2 + offset),
            NotificationPosition::TopCenter => (25, 2 + offset),
            NotificationPosition::BottomRight => (50, 20 - offset),
            NotificationPosition::BottomLeft => (2, 20 - offset),
            NotificationPosition::BottomCenter => (25, 20 - offset),
            NotificationPosition::Center => (25, 10 + offset - (self.active_notifications.len() as i32 * spacing / 2)),
        }
    }
}

impl UIComponent for AchievementNotificationSystem {
    fn render(&self) -> Vec<UIRenderCommand> {
        let mut commands = Vec::new();

        for (index, active_notification) in self.active_notifications.iter().enumerate() {
            let notification_commands = self.render_notification(active_notification, index);
            commands.extend(notification_commands);
        }

        commands
    }
}

impl AchievementNotificationSystem {
    /// Render a single notification
    fn render_notification(&self, active_notification: &ActiveNotification, index: usize) -> Vec<UIRenderCommand> {
        let mut commands = Vec::new();
        let notification = &active_notification.notification;

        // Get rarity effects
        let (rarity_color, rarity_effect, scale) = if self.config.show_rarity_effects {
            self.get_rarity_effects(&notification.rarity)
        } else {
            (Color::White, "".to_string(), 1.0)
        };

        // Calculate dimensions
        let base_width = 35;
        let base_height = match self.config.style {
            NotificationStyle::Popup => 6,
            NotificationStyle::Toast => 4,
            NotificationStyle::Banner => 3,
            NotificationStyle::Minimal => 2,
        };

        let width = (base_width as f32 * scale) as i32;
        let height = (base_height as f32 * scale) as i32;

        // Calculate position with animation offset
        let (base_x, base_y) = self.calculate_position(index, height);
        let (x, y) = match self.config.animation {
            NotificationAnimation::SlideIn => {
                let slide_offset = ((1.0 - active_notification.animation_progress) * 30.0) as i32;
                match self.config.position {
                    NotificationPosition::TopRight | NotificationPosition::BottomRight => (base_x + slide_offset, base_y),
                    NotificationPosition::TopLeft | NotificationPosition::BottomLeft => (base_x - slide_offset, base_y),
                    _ => (base_x, base_y),
                }
            },
            NotificationAnimation::Bounce => {
                let bounce_offset = ((1.0 - active_notification.animation_progress) * 10.0 * 
                    (active_notification.animation_progress * 10.0).sin()) as i32;
                (base_x, base_y - bounce_offset)
            },
            _ => (base_x, base_y),
        };

        // Render based on style
        match self.config.style {
            NotificationStyle::Popup => {
                self.render_popup_notification(&mut commands, notification, x, y, width, height, rarity_color, &rarity_effect, active_notification);
            },
            NotificationStyle::Toast => {
                self.render_toast_notification(&mut commands, notification, x, y, width, height, rarity_color, &rarity_effect);
            },
            NotificationStyle::Banner => {
                self.render_banner_notification(&mut commands, notification, x, y, width, height, rarity_color, &rarity_effect);
            },
            NotificationStyle::Minimal => {
                self.render_minimal_notification(&mut commands, notification, x, y, width, height, rarity_color);
            },
        }

        commands
    }

    /// Render popup style notification
    fn render_popup_notification(
        &self,
        commands: &mut Vec<UIRenderCommand>,
        notification: &AchievementNotification,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        rarity_color: Color,
        rarity_effect: &str,
        active_notification: &ActiveNotification,
    ) {
        // Main panel
        commands.push(UIRenderCommand::Panel(UIPanel {
            x,
            y,
            width,
            height,
            title: Some("Achievement Unlocked!".to_string()),
            border_color: rarity_color,
            background_color: Color::Black,
        }));

        // Achievement icon and effect
        commands.push(UIRenderCommand::Text(UIText {
            x: x + 2,
            y: y + 2,
            text: format!("{} {}", notification.achievement_icon, rarity_effect),
            color: Color::Yellow,
            alignment: TextAlignment::Left,
        }));

        // Achievement name
        commands.push(UIRenderCommand::Text(UIText {
            x: x + 2,
            y: y + 3,
            text: notification.achievement_name.clone(),
            color: rarity_color,
            alignment: TextAlignment::Left,
        }));

        // Points
        commands.push(UIRenderCommand::Text(UIText {
            x: x + 2,
            y: y + 4,
            text: format!("+{} points", notification.points),
            color: Color::Green,
            alignment: TextAlignment::Left,
        }));

        // Progress bar if enabled
        if self.config.show_progress {
            let progress_width = width - 4;
            let filled = (active_notification.display_progress * progress_width as f32) as i32;
            let empty = progress_width - filled;

            commands.push(UIRenderCommand::Text(UIText {
                x: x + 2,
                y: y + height - 1,
                text: format!("{}{}", "█".repeat(filled as usize), "░".repeat(empty as usize)),
                color: Color::DarkGrey,
                alignment: TextAlignment::Left,
            }));
        }
    }

    /// Render toast style notification
    fn render_toast_notification(
        &self,
        commands: &mut Vec<UIRenderCommand>,
        notification: &AchievementNotification,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        rarity_color: Color,
        rarity_effect: &str,
    ) {
        // Simplified panel
        commands.push(UIRenderCommand::Panel(UIPanel {
            x,
            y,
            width,
            height,
            title: None,
            border_color: rarity_color,
            background_color: Color::Black,
        }));

        // Single line with icon, name, and points
        commands.push(UIRenderCommand::Text(UIText {
            x: x + 1,
            y: y + 1,
            text: format!("{} {} {} (+{})", 
                notification.achievement_icon,
                rarity_effect,
                notification.achievement_name,
                notification.points
            ),
            color: rarity_color,
            alignment: TextAlignment::Left,
        }));
    }

    /// Render banner style notification
    fn render_banner_notification(
        &self,
        commands: &mut Vec<UIRenderCommand>,
        notification: &AchievementNotification,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        rarity_color: Color,
        rarity_effect: &str,
    ) {
        // Full width banner
        commands.push(UIRenderCommand::Panel(UIPanel {
            x: 0,
            y,
            width: 80, // Full screen width
            height,
            title: None,
            border_color: rarity_color,
            background_color: Color::Black,
        }));

        // Centered text
        commands.push(UIRenderCommand::Text(UIText {
            x: 40 - (notification.achievement_name.len() as i32 / 2),
            y: y + 1,
            text: format!("{} {} {} - Achievement Unlocked! (+{} points)", 
                notification.achievement_icon,
                rarity_effect,
                notification.achievement_name,
                notification.points
            ),
            color: rarity_color,
            alignment: TextAlignment::Center,
        }));
    }

    /// Render minimal style notification
    fn render_minimal_notification(
        &self,
        commands: &mut Vec<UIRenderCommand>,
        notification: &AchievementNotification,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        rarity_color: Color,
    ) {
        // Just text, no panel
        commands.push(UIRenderCommand::Text(UIText {
            x,
            y,
            text: format!("{} {} (+{})", 
                notification.achievement_icon,
                notification.achievement_name,
                notification.points
            ),
            color: rarity_color,
            alignment: TextAlignment::Left,
        }));
    }
}

/// Sound effects for achievements (placeholder)
pub struct AchievementSoundSystem {
    enabled: bool,
}

impl AchievementSoundSystem {
    pub fn new() -> Self {
        AchievementSoundSystem {
            enabled: true,
        }
    }

    /// Play sound for achievement unlock
    pub fn play_unlock_sound(&self, rarity: &AchievementRarity) {
        if !self.enabled {
            return;
        }

        // In a real implementation, this would play actual sound files
        match rarity {
            AchievementRarity::Common => println!("🔊 *ding*"),
            AchievementRarity::Uncommon => println!("🔊 *chime*"),
            AchievementRarity::Rare => println!("🔊 *fanfare*"),
            AchievementRarity::Epic => println!("🔊 *epic fanfare*"),
            AchievementRarity::Legendary => println!("🔊 *legendary fanfare*"),
        }
    }

    /// Enable or disable sounds
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Check if sounds are enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::achievements::achievement_system::AchievementType;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn create_test_notification() -> AchievementNotification {
        AchievementNotification {
            achievement_id: "test".to_string(),
            achievement_name: "Test Achievement".to_string(),
            achievement_icon: "🏆".to_string(),
            message: "Test message".to_string(),
            points: 10,
            rarity: AchievementRarity::Common,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }

    #[test]
    fn test_notification_system_creation() {
        let config = NotificationConfig::default();
        let system = AchievementNotificationSystem::new(config);
        
        assert_eq!(system.get_notification_count(), 0);
        assert!(system.is_enabled());
    }

    #[test]
    fn test_add_notification() {
        let config = NotificationConfig::default();
        let mut system = AchievementNotificationSystem::new(config);
        
        let notification = create_test_notification();
        system.add_notification(notification);
        
        assert_eq!(system.get_notification_count(), 1);
    }

    #[test]
    fn test_duplicate_notification_prevention() {
        let config = NotificationConfig::default();
        let mut system = AchievementNotificationSystem::new(config);
        
        let notification1 = create_test_notification();
        let notification2 = create_test_notification(); // Same ID
        
        system.add_notification(notification1);
        system.add_notification(notification2);
        
        // Should only have one notification due to duplicate prevention
        assert_eq!(system.get_notification_count(), 1);
    }

    #[test]
    fn test_notification_update() {
        let config = NotificationConfig::default();
        let mut system = AchievementNotificationSystem::new(config);
        
        let notification = create_test_notification();
        system.add_notification(notification);
        
        // Update should move notification from queue to active
        system.update();
        assert_eq!(system.get_active_notifications().len(), 1);
    }

    #[test]
    fn test_max_concurrent_notifications() {
        let mut config = NotificationConfig::default();
        config.max_concurrent = 2;
        let mut system = AchievementNotificationSystem::new(config);
        
        // Add more notifications than the limit
        for i in 0..5 {
            let mut notification = create_test_notification();
            notification.achievement_id = format!("test_{}", i);
            system.add_notification(notification);
        }
        
        system.update();
        
        // Should only have max_concurrent active notifications
        assert_eq!(system.get_active_notifications().len(), 2);
        assert_eq!(system.get_notification_count(), 5); // Total including queued
    }

    #[test]
    fn test_notification_dismissal() {
        let config = NotificationConfig::default();
        let mut system = AchievementNotificationSystem::new(config);
        
        let notification = create_test_notification();
        system.add_notification(notification);
        system.update();
        
        assert_eq!(system.get_active_notifications().len(), 1);
        
        system.dismiss_notification("test");
        system.update();
        
        assert_eq!(system.get_active_notifications().len(), 0);
    }

    #[test]
    fn test_clear_all_notifications() {
        let config = NotificationConfig::default();
        let mut system = AchievementNotificationSystem::new(config);
        
        let notification = create_test_notification();
        system.add_notification(notification);
        system.update();
        
        assert_eq!(system.get_notification_count(), 1);
        
        system.clear_all();
        
        assert_eq!(system.get_notification_count(), 0);
    }

    #[test]
    fn test_active_notification_update() {
        let notification = create_test_notification();
        let mut active = ActiveNotification::new(notification);
        let config = NotificationConfig::default();
        
        assert_eq!(active.animation_progress, 0.0);
        assert_eq!(active.display_progress, 0.0);
        
        // Update with some time passed
        let still_active = active.update(Duration::from_millis(100), &config);
        assert!(still_active);
        assert!(active.display_progress > 0.0);
    }

    #[test]
    fn test_sound_system() {
        let mut sound_system = AchievementSoundSystem::new();
        
        assert!(sound_system.is_enabled());
        
        sound_system.play_unlock_sound(&AchievementRarity::Legendary);
        
        sound_system.set_enabled(false);
        assert!(!sound_system.is_enabled());
    }

    #[test]
    fn test_rarity_effects() {
        let config = NotificationConfig::default();
        let system = AchievementNotificationSystem::new(config);
        
        let (color, effect, scale) = system.get_rarity_effects(&AchievementRarity::Legendary);
        assert_eq!(color, Color::Yellow);
        assert_eq!(effect, "🌟");
        assert_eq!(scale, 1.5);
    }

    #[test]
    fn test_position_calculation() {
        let config = NotificationConfig {
            position: NotificationPosition::TopRight,
            ..NotificationConfig::default()
        };
        let system = AchievementNotificationSystem::new(config);
        
        let (x, y) = system.calculate_position(0, 5);
        assert_eq!(x, 50);
        assert_eq!(y, 2);
        
        let (x, y) = system.calculate_position(1, 5);
        assert_eq!(x, 50);
        assert_eq!(y, 8); // 2 + (1 * (5 + 1))
    }
}