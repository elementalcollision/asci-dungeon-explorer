use crossterm::{event::KeyCode, style::Color};
use std::time::{SystemTime, UNIX_EPOCH};
use crate::{
    ui::{
        ui_components::{UIComponent, UIRenderCommand, UIPanel, UIButton, UIText, UIList, TextAlignment},
        menu_system::{MenuRenderer, MenuInput},
    },
    achievements::achievement_system::{
        Achievement, AchievementSystem, AchievementType, AchievementRarity, 
        AchievementDifficulty, AchievementProgress, AchievementStatistics,
        UnlockedAchievement, AchievementNotification,
    },
};

/// Achievement UI state
#[derive(Debug, Clone, PartialEq)]
pub enum AchievementUIState {
    MainView,
    CategoryView(AchievementType),
    DetailView(String), // Achievement ID
    StatisticsView,
    NotificationView,
    Closed,
}

/// Achievement UI sorting options
#[derive(Debug, Clone, PartialEq)]
pub enum AchievementSortMode {
    Name,
    Rarity,
    Points,
    Progress,
    UnlockDate,
    Type,
}

/// Achievement UI filter options
#[derive(Debug, Clone, PartialEq)]
pub enum AchievementFilter {
    All,
    Unlocked,
    Locked,
    InProgress,
    Type(AchievementType),
    Rarity(AchievementRarity),
}

/// Achievement UI component
pub struct AchievementUI {
    pub state: AchievementUIState,
    pub selected_index: usize,
    pub scroll_offset: usize,
    pub max_visible_items: usize,
    pub sort_mode: AchievementSortMode,
    pub filter: AchievementFilter,
    pub show_hidden: bool,
    pub show_progress_bars: bool,
    pub notifications: Vec<AchievementNotification>,
    pub notification_display_time: u64, // seconds
}

impl AchievementUI {
    pub fn new() -> Self {
        AchievementUI {
            state: AchievementUIState::Closed,
            selected_index: 0,
            scroll_offset: 0,
            max_visible_items: 15,
            sort_mode: AchievementSortMode::Name,
            filter: AchievementFilter::All,
            show_hidden: false,
            show_progress_bars: true,
            notifications: Vec::new(),
            notification_display_time: 5, // 5 seconds
        }
    }

    /// Open achievement UI
    pub fn open(&mut self) {
        self.state = AchievementUIState::MainView;
        self.selected_index = 0;
        self.scroll_offset = 0;
    }

    /// Close achievement UI
    pub fn close(&mut self) {
        self.state = AchievementUIState::Closed;
    }

    /// Check if UI is open
    pub fn is_open(&self) -> bool {
        self.state != AchievementUIState::Closed
    }

    /// Handle input
    pub fn handle_input(&mut self, key: KeyCode, achievement_system: &AchievementSystem) -> bool {
        match self.state {
            AchievementUIState::MainView => self.handle_main_view_input(key, achievement_system),
            AchievementUIState::CategoryView(_) => self.handle_category_view_input(key, achievement_system),
            AchievementUIState::DetailView(_) => self.handle_detail_view_input(key),
            AchievementUIState::StatisticsView => self.handle_statistics_view_input(key),
            AchievementUIState::NotificationView => self.handle_notification_view_input(key),
            AchievementUIState::Closed => false,
        }
    }

    /// Handle main view input
    fn handle_main_view_input(&mut self, key: KeyCode, achievement_system: &AchievementSystem) -> bool {
        let achievements = self.get_filtered_achievements(achievement_system);
        
        match key {
            KeyCode::Up => {
                if self.selected_index > 0 {
                    self.selected_index -= 1;
                    self.update_scroll();
                }
                true
            },
            KeyCode::Down => {
                if self.selected_index < achievements.len().saturating_sub(1) {
                    self.selected_index += 1;
                    self.update_scroll();
                }
                true
            },
            KeyCode::Enter => {
                if self.selected_index < achievements.len() {
                    let achievement_id = achievements[self.selected_index].id.clone();
                    self.state = AchievementUIState::DetailView(achievement_id);
                }
                true
            },
            KeyCode::Char('s') | KeyCode::Char('S') => {
                self.state = AchievementUIState::StatisticsView;
                true
            },
            KeyCode::Char('f') | KeyCode::Char('F') => {
                self.cycle_filter();
                self.selected_index = 0;
                self.scroll_offset = 0;
                true
            },
            KeyCode::Char('o') | KeyCode::Char('O') => {
                self.cycle_sort_mode();
                self.selected_index = 0;
                self.scroll_offset = 0;
                true
            },
            KeyCode::Char('h') | KeyCode::Char('H') => {
                self.show_hidden = !self.show_hidden;
                self.selected_index = 0;
                self.scroll_offset = 0;
                true
            },
            KeyCode::Char('p') | KeyCode::Char('P') => {
                self.show_progress_bars = !self.show_progress_bars;
                true
            },
            KeyCode::Char('n') | KeyCode::Char('N') => {
                if !self.notifications.is_empty() {
                    self.state = AchievementUIState::NotificationView;
                }
                true
            },
            KeyCode::Char('1') => {
                self.state = AchievementUIState::CategoryView(AchievementType::Combat);
                self.selected_index = 0;
                self.scroll_offset = 0;
                true
            },
            KeyCode::Char('2') => {
                self.state = AchievementUIState::CategoryView(AchievementType::Exploration);
                self.selected_index = 0;
                self.scroll_offset = 0;
                true
            },
            KeyCode::Char('3') => {
                self.state = AchievementUIState::CategoryView(AchievementType::Collection);
                self.selected_index = 0;
                self.scroll_offset = 0;
                true
            },
            KeyCode::Char('4') => {
                self.state = AchievementUIState::CategoryView(AchievementType::Progression);
                self.selected_index = 0;
                self.scroll_offset = 0;
                true
            },
            KeyCode::Char('5') => {
                self.state = AchievementUIState::CategoryView(AchievementType::Special);
                self.selected_index = 0;
                self.scroll_offset = 0;
                true
            },
            KeyCode::Esc => {
                self.close();
                true
            },
            _ => false,
        }
    }

    /// Handle category view input
    fn handle_category_view_input(&mut self, key: KeyCode, achievement_system: &AchievementSystem) -> bool {
        if let AchievementUIState::CategoryView(category) = &self.state {
            let achievements = achievement_system.get_achievements_by_type(category);
            
            match key {
                KeyCode::Up => {
                    if self.selected_index > 0 {
                        self.selected_index -= 1;
                        self.update_scroll();
                    }
                    true
                },
                KeyCode::Down => {
                    if self.selected_index < achievements.len().saturating_sub(1) {
                        self.selected_index += 1;
                        self.update_scroll();
                    }
                    true
                },
                KeyCode::Enter => {
                    if self.selected_index < achievements.len() {
                        let achievement_id = achievements[self.selected_index].id.clone();
                        self.state = AchievementUIState::DetailView(achievement_id);
                    }
                    true
                },
                KeyCode::Esc => {
                    self.state = AchievementUIState::MainView;
                    true
                },
                _ => false,
            }
        } else {
            false
        }
    }

    /// Handle detail view input
    fn handle_detail_view_input(&mut self, key: KeyCode) -> bool {
        match key {
            KeyCode::Esc => {
                self.state = AchievementUIState::MainView;
                true
            },
            _ => false,
        }
    }

    /// Handle statistics view input
    fn handle_statistics_view_input(&mut self, key: KeyCode) -> bool {
        match key {
            KeyCode::Esc => {
                self.state = AchievementUIState::MainView;
                true
            },
            _ => false,
        }
    }

    /// Handle notification view input
    fn handle_notification_view_input(&mut self, key: KeyCode) -> bool {
        match key {
            KeyCode::Esc | KeyCode::Enter => {
                self.state = AchievementUIState::MainView;
                true
            },
            KeyCode::Char('c') | KeyCode::Char('C') => {
                self.notifications.clear();
                self.state = AchievementUIState::MainView;
                true
            },
            _ => false,
        }
    }

    /// Update scroll offset based on selected index
    fn update_scroll(&mut self) {
        if self.selected_index < self.scroll_offset {
            self.scroll_offset = self.selected_index;
        } else if self.selected_index >= self.scroll_offset + self.max_visible_items {
            self.scroll_offset = self.selected_index - self.max_visible_items + 1;
        }
    }

    /// Cycle through filter options
    fn cycle_filter(&mut self) {
        self.filter = match self.filter {
            AchievementFilter::All => AchievementFilter::Unlocked,
            AchievementFilter::Unlocked => AchievementFilter::Locked,
            AchievementFilter::Locked => AchievementFilter::InProgress,
            AchievementFilter::InProgress => AchievementFilter::Rarity(AchievementRarity::Common),
            AchievementFilter::Rarity(AchievementRarity::Common) => AchievementFilter::Rarity(AchievementRarity::Uncommon),
            AchievementFilter::Rarity(AchievementRarity::Uncommon) => AchievementFilter::Rarity(AchievementRarity::Rare),
            AchievementFilter::Rarity(AchievementRarity::Rare) => AchievementFilter::Rarity(AchievementRarity::Epic),
            AchievementFilter::Rarity(AchievementRarity::Epic) => AchievementFilter::Rarity(AchievementRarity::Legendary),
            AchievementFilter::Rarity(AchievementRarity::Legendary) => AchievementFilter::All,
            _ => AchievementFilter::All,
        };
    }

    /// Cycle through sort modes
    fn cycle_sort_mode(&mut self) {
        self.sort_mode = match self.sort_mode {
            AchievementSortMode::Name => AchievementSortMode::Rarity,
            AchievementSortMode::Rarity => AchievementSortMode::Points,
            AchievementSortMode::Points => AchievementSortMode::Progress,
            AchievementSortMode::Progress => AchievementSortMode::Type,
            AchievementSortMode::Type => AchievementSortMode::Name,
            AchievementSortMode::UnlockDate => AchievementSortMode::Name,
        };
    }

    /// Get filtered and sorted achievements
    fn get_filtered_achievements(&self, achievement_system: &AchievementSystem) -> Vec<&Achievement> {
        let mut achievements = achievement_system.get_achievements(self.show_hidden);
        
        // Apply filter
        achievements = achievements.into_iter().filter(|achievement| {
            match &self.filter {
                AchievementFilter::All => true,
                AchievementFilter::Unlocked => achievement_system.is_unlocked(&achievement.id),
                AchievementFilter::Locked => !achievement_system.is_unlocked(&achievement.id),
                AchievementFilter::InProgress => {
                    if let Some(progress) = achievement_system.get_progress(&achievement.id) {
                        progress.current > 0 && !progress.is_complete()
                    } else {
                        false
                    }
                },
                AchievementFilter::Type(filter_type) => &achievement.achievement_type == filter_type,
                AchievementFilter::Rarity(filter_rarity) => &achievement.rarity == filter_rarity,
            }
        }).collect();
        
        // Apply sort
        achievements.sort_by(|a, b| {
            match self.sort_mode {
                AchievementSortMode::Name => a.name.cmp(&b.name),
                AchievementSortMode::Rarity => a.rarity.cmp(&b.rarity),
                AchievementSortMode::Points => b.points.cmp(&a.points), // Descending
                AchievementSortMode::Progress => {
                    let a_progress = achievement_system.get_progress(&a.id)
                        .map(|p| p.progress_percentage())
                        .unwrap_or(if achievement_system.is_unlocked(&a.id) { 100.0 } else { 0.0 });
                    let b_progress = achievement_system.get_progress(&b.id)
                        .map(|p| p.progress_percentage())
                        .unwrap_or(if achievement_system.is_unlocked(&b.id) { 100.0 } else { 0.0 });
                    b_progress.partial_cmp(&a_progress).unwrap_or(std::cmp::Ordering::Equal)
                },
                AchievementSortMode::Type => a.achievement_type.to_string().cmp(&b.achievement_type.to_string()),
                AchievementSortMode::UnlockDate => std::cmp::Ordering::Equal, // Not implemented for this view
            }
        });
        
        achievements
    }

    /// Add notification
    pub fn add_notification(&mut self, notification: AchievementNotification) {
        self.notifications.push(notification);
    }

    /// Update notifications (remove expired ones)
    pub fn update_notifications(&mut self) {
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        self.notifications.retain(|notification| {
            current_time - notification.timestamp < self.notification_display_time
        });
    }

    /// Get current state
    pub fn get_state(&self) -> &AchievementUIState {
        &self.state
    }

    /// Get rarity color
    fn get_rarity_color(rarity: &AchievementRarity) -> Color {
        match rarity {
            AchievementRarity::Common => Color::White,
            AchievementRarity::Uncommon => Color::Green,
            AchievementRarity::Rare => Color::Blue,
            AchievementRarity::Epic => Color::Magenta,
            AchievementRarity::Legendary => Color::Yellow,
        }
    }

    /// Get type color
    fn get_type_color(achievement_type: &AchievementType) -> Color {
        match achievement_type {
            AchievementType::Combat => Color::Red,
            AchievementType::Exploration => Color::Cyan,
            AchievementType::Collection => Color::Yellow,
            AchievementType::Progression => Color::Green,
            AchievementType::Social => Color::Magenta,
            AchievementType::Special => Color::Blue,
            AchievementType::Hidden => Color::DarkGrey,
        }
    }

    /// Format progress bar
    fn format_progress_bar(&self, progress: &AchievementProgress, width: usize) -> String {
        let filled = ((progress.progress_percentage() / 100.0) * width as f32) as usize;
        let empty = width - filled;
        
        format!("[{}{}] {}/{}", 
            "█".repeat(filled),
            "░".repeat(empty),
            progress.current,
            progress.target
        )
    }
}

impl UIComponent for AchievementUI {
    fn render(&self) -> Vec<UIRenderCommand> {
        match &self.state {
            AchievementUIState::MainView => self.render_main_view(),
            AchievementUIState::CategoryView(category) => self.render_category_view(category),
            AchievementUIState::DetailView(achievement_id) => self.render_detail_view(achievement_id),
            AchievementUIState::StatisticsView => self.render_statistics_view(),
            AchievementUIState::NotificationView => self.render_notification_view(),
            AchievementUIState::Closed => Vec::new(),
        }
    }
}

impl AchievementUI {
    /// Render main view
    fn render_main_view(&self) -> Vec<UIRenderCommand> {
        let mut commands = Vec::new();
        
        // Main panel
        commands.push(UIRenderCommand::Panel(UIPanel {
            x: 5,
            y: 2,
            width: 70,
            height: 25,
            title: Some("Achievements".to_string()),
            border_color: Color::White,
            background_color: Color::Black,
        }));

        // Header with filter and sort info
        commands.push(UIRenderCommand::Text(UIText {
            x: 7,
            y: 4,
            text: format!("Filter: {:?} | Sort: {:?} | Hidden: {}", 
                self.filter, self.sort_mode, if self.show_hidden { "ON" } else { "OFF" }),
            color: Color::DarkGrey,
            alignment: TextAlignment::Left,
        }));

        // Controls
        commands.push(UIRenderCommand::Text(UIText {
            x: 7,
            y: 5,
            text: "↑/↓: Navigate | Enter: Details | F: Filter | O: Sort | H: Hidden | S: Stats | N: Notifications | 1-5: Categories".to_string(),
            color: Color::DarkGrey,
            alignment: TextAlignment::Left,
        }));

        // Achievement list would be rendered here
        // This is a simplified version - in a real implementation, you'd get the achievements
        // from the achievement system and render them
        
        commands.push(UIRenderCommand::Text(UIText {
            x: 7,
            y: 7,
            text: "Achievement list would be rendered here...".to_string(),
            color: Color::White,
            alignment: TextAlignment::Left,
        }));

        commands
    }

    /// Render category view
    fn render_category_view(&self, category: &AchievementType) -> Vec<UIRenderCommand> {
        let mut commands = Vec::new();
        
        commands.push(UIRenderCommand::Panel(UIPanel {
            x: 5,
            y: 2,
            width: 70,
            height: 25,
            title: Some(format!("{:?} Achievements", category)),
            border_color: Self::get_type_color(category),
            background_color: Color::Black,
        }));

        commands.push(UIRenderCommand::Text(UIText {
            x: 7,
            y: 4,
            text: "Category-specific achievements would be listed here...".to_string(),
            color: Color::White,
            alignment: TextAlignment::Left,
        }));

        commands
    }

    /// Render detail view
    fn render_detail_view(&self, achievement_id: &str) -> Vec<UIRenderCommand> {
        let mut commands = Vec::new();
        
        commands.push(UIRenderCommand::Panel(UIPanel {
            x: 10,
            y: 5,
            width: 60,
            height: 20,
            title: Some("Achievement Details".to_string()),
            border_color: Color::Yellow,
            background_color: Color::Black,
        }));

        commands.push(UIRenderCommand::Text(UIText {
            x: 12,
            y: 7,
            text: format!("Details for achievement: {}", achievement_id),
            color: Color::White,
            alignment: TextAlignment::Left,
        }));

        commands
    }

    /// Render statistics view
    fn render_statistics_view(&self) -> Vec<UIRenderCommand> {
        let mut commands = Vec::new();
        
        commands.push(UIRenderCommand::Panel(UIPanel {
            x: 10,
            y: 5,
            width: 60,
            height: 20,
            title: Some("Achievement Statistics".to_string()),
            border_color: Color::Green,
            background_color: Color::Black,
        }));

        commands.push(UIRenderCommand::Text(UIText {
            x: 12,
            y: 7,
            text: "Achievement statistics would be displayed here...".to_string(),
            color: Color::White,
            alignment: TextAlignment::Left,
        }));

        commands
    }

    /// Render notification view
    fn render_notification_view(&self) -> Vec<UIRenderCommand> {
        let mut commands = Vec::new();
        
        commands.push(UIRenderCommand::Panel(UIPanel {
            x: 15,
            y: 8,
            width: 50,
            height: 15,
            title: Some("Recent Achievements".to_string()),
            border_color: Color::Magenta,
            background_color: Color::Black,
        }));

        // Render notifications
        for (i, notification) in self.notifications.iter().enumerate() {
            let y = 10 + i as i32;
            if y < 20 { // Stay within panel bounds
                commands.push(UIRenderCommand::Text(UIText {
                    x: 17,
                    y,
                    text: format!("{} {} (+{} pts)", 
                        notification.achievement_icon,
                        notification.achievement_name,
                        notification.points
                    ),
                    color: Self::get_rarity_color(&notification.rarity),
                    alignment: TextAlignment::Left,
                }));
            }
        }

        commands.push(UIRenderCommand::Text(UIText {
            x: 17,
            y: 21,
            text: "C: Clear notifications | Esc: Back".to_string(),
            color: Color::DarkGrey,
            alignment: TextAlignment::Left,
        }));

        commands
    }
}

/// Achievement notification popup
pub struct AchievementNotificationPopup {
    pub notification: AchievementNotification,
    pub display_time_remaining: f32,
    pub animation_progress: f32,
}

impl AchievementNotificationPopup {
    pub fn new(notification: AchievementNotification, display_duration: f32) -> Self {
        AchievementNotificationPopup {
            notification,
            display_time_remaining: display_duration,
            animation_progress: 0.0,
        }
    }

    /// Update the popup (call each frame)
    pub fn update(&mut self, delta_time: f32) -> bool {
        self.display_time_remaining -= delta_time;
        
        // Animation progress (0.0 to 1.0)
        let total_duration = 3.0; // Total display duration
        self.animation_progress = (total_duration - self.display_time_remaining) / total_duration;
        self.animation_progress = self.animation_progress.clamp(0.0, 1.0);
        
        self.display_time_remaining > 0.0
    }

    /// Check if popup should be removed
    pub fn should_remove(&self) -> bool {
        self.display_time_remaining <= 0.0
    }
}

impl UIComponent for AchievementNotificationPopup {
    fn render(&self) -> Vec<UIRenderCommand> {
        let mut commands = Vec::new();
        
        // Calculate position based on animation
        let slide_offset = if self.animation_progress < 0.2 {
            // Slide in from right
            ((1.0 - self.animation_progress / 0.2) * 20.0) as i32
        } else if self.animation_progress > 0.8 {
            // Slide out to right
            ((self.animation_progress - 0.8) / 0.2 * 20.0) as i32
        } else {
            0
        };

        let x = 50 + slide_offset;
        let y = 3;

        // Notification panel
        commands.push(UIRenderCommand::Panel(UIPanel {
            x,
            y,
            width: 30,
            height: 5,
            title: None,
            border_color: AchievementUI::get_rarity_color(&self.notification.rarity),
            background_color: Color::Black,
        }));

        // Achievement icon and name
        commands.push(UIRenderCommand::Text(UIText {
            x: x + 1,
            y: y + 1,
            text: format!("{} Achievement Unlocked!", self.notification.achievement_icon),
            color: Color::Yellow,
            alignment: TextAlignment::Left,
        }));

        commands.push(UIRenderCommand::Text(UIText {
            x: x + 1,
            y: y + 2,
            text: self.notification.achievement_name.clone(),
            color: AchievementUI::get_rarity_color(&self.notification.rarity),
            alignment: TextAlignment::Left,
        }));

        commands.push(UIRenderCommand::Text(UIText {
            x: x + 1,
            y: y + 3,
            text: format!("+{} points", self.notification.points),
            color: Color::Green,
            alignment: TextAlignment::Left,
        }));

        commands
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::achievements::achievement_system::AchievementSystem;

    #[test]
    fn test_achievement_ui_creation() {
        let ui = AchievementUI::new();
        assert_eq!(ui.state, AchievementUIState::Closed);
        assert!(!ui.is_open());
    }

    #[test]
    fn test_achievement_ui_open_close() {
        let mut ui = AchievementUI::new();
        
        ui.open();
        assert_eq!(ui.state, AchievementUIState::MainView);
        assert!(ui.is_open());
        
        ui.close();
        assert_eq!(ui.state, AchievementUIState::Closed);
        assert!(!ui.is_open());
    }

    #[test]
    fn test_filter_cycling() {
        let mut ui = AchievementUI::new();
        
        assert_eq!(ui.filter, AchievementFilter::All);
        
        ui.cycle_filter();
        assert_eq!(ui.filter, AchievementFilter::Unlocked);
        
        ui.cycle_filter();
        assert_eq!(ui.filter, AchievementFilter::Locked);
    }

    #[test]
    fn test_sort_mode_cycling() {
        let mut ui = AchievementUI::new();
        
        assert_eq!(ui.sort_mode, AchievementSortMode::Name);
        
        ui.cycle_sort_mode();
        assert_eq!(ui.sort_mode, AchievementSortMode::Rarity);
        
        ui.cycle_sort_mode();
        assert_eq!(ui.sort_mode, AchievementSortMode::Points);
    }

    #[test]
    fn test_notification_management() {
        let mut ui = AchievementUI::new();
        
        let notification = AchievementNotification {
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
        };
        
        ui.add_notification(notification);
        assert_eq!(ui.notifications.len(), 1);
        
        // Test notification expiry (would need to mock time for proper testing)
        ui.update_notifications();
        // Notifications should still be there since they're recent
        assert_eq!(ui.notifications.len(), 1);
    }

    #[test]
    fn test_notification_popup() {
        let notification = AchievementNotification {
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
        };
        
        let mut popup = AchievementNotificationPopup::new(notification, 3.0);
        
        assert!(!popup.should_remove());
        assert_eq!(popup.animation_progress, 0.0);
        
        // Update popup
        let still_active = popup.update(0.1);
        assert!(still_active);
        assert!(popup.animation_progress > 0.0);
    }

    #[test]
    fn test_rarity_colors() {
        assert_eq!(AchievementUI::get_rarity_color(&AchievementRarity::Common), Color::White);
        assert_eq!(AchievementUI::get_rarity_color(&AchievementRarity::Legendary), Color::Yellow);
    }

    #[test]
    fn test_type_colors() {
        assert_eq!(AchievementUI::get_type_color(&AchievementType::Combat), Color::Red);
        assert_eq!(AchievementUI::get_type_color(&AchievementType::Exploration), Color::Cyan);
    }
}