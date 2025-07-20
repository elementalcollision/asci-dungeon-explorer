use bevy::prelude::*;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use crate::ai::{AIComponent, AIBehaviorState, SpecialEnemyComponent, AttackWarning};
use crate::components::{Position, Health};

/// Visual indicator types for enemy states
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum StateIndicatorType {
    /// Health status indicators
    HealthBar {
        show_percentage: bool,
        color_coding: bool,
        show_when_full: bool,
    },
    /// Behavior state indicators
    BehaviorState {
        show_icon: bool,
        show_text: bool,
        color_coded: bool,
    },
    /// Alert level indicators
    AlertLevel {
        alert_radius: f32,
        intensity_scaling: bool,
        color_gradient: bool,
    },
    /// Special ability indicators
    AbilityCharging {
        ability_name: String,
        charge_time: f32,
        warning_color: Color,
    },
    /// Status effect indicators
    StatusEffect {
        effect_name: String,
        duration: f32,
        stacking: bool,
    },
    /// Aggression level indicators
    AggressionLevel {
        show_intensity: bool,
        color_scaling: bool,
        particle_effects: bool,
    },
    /// Detection indicators
    Detection {
        detection_cone: bool,
        hearing_radius: bool,
        line_of_sight: bool,
    },
    /// Phase transition indicators
    PhaseTransition {
        transition_name: String,
        transition_progress: f32,
        dramatic_effect: bool,
    },
}

/// Color schemes for different indicator types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IndicatorColorScheme {
    pub primary: Color,
    pub secondary: Color,
    pub warning: Color,
    pub danger: Color,
    pub neutral: Color,
    pub background: Color,
}

impl Default for IndicatorColorScheme {
    fn default() -> Self {
        IndicatorColorScheme {
            primary: Color::WHITE,
            secondary: Color::GRAY,
            warning: Color::YELLOW,
            danger: Color::RED,
            neutral: Color::BLUE,
            background: Color::BLACK,
        }
    }
}

impl IndicatorColorScheme {
    /// Health-based color scheme
    pub fn health_scheme() -> Self {
        IndicatorColorScheme {
            primary: Color::GREEN,
            secondary: Color::YELLOW,
            warning: Color::ORANGE,
            danger: Color::RED,
            neutral: Color::GRAY,
            background: Color::BLACK,
        }
    }

    /// Aggression-based color scheme
    pub fn aggression_scheme() -> Self {
        IndicatorColorScheme {
            primary: Color::BLUE,
            secondary: Color::CYAN,
            warning: Color::YELLOW,
            danger: Color::RED,
            neutral: Color::WHITE,
            background: Color::BLACK,
        }
    }

    /// Alert-based color scheme
    pub fn alert_scheme() -> Self {
        IndicatorColorScheme {
            primary: Color::GREEN,
            secondary: Color::YELLOW,
            warning: Color::ORANGE,
            danger: Color::RED,
            neutral: Color::GRAY,
            background: Color::BLACK,
        }
    }
}

/// Component for managing enemy state indicators
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct StateIndicatorComponent {
    pub indicators: Vec<StateIndicatorType>,
    pub color_scheme: IndicatorColorScheme,
    pub visibility_range: f32,
    pub always_visible: bool,
    pub fade_distance: f32,
    pub update_frequency: f32,
    pub last_update: f32,
    pub indicator_data: HashMap<String, f32>,
}

impl Default for StateIndicatorComponent {
    fn default() -> Self {
        StateIndicatorComponent {
            indicators: vec![
                StateIndicatorType::HealthBar {
                    show_percentage: false,
                    color_coding: true,
                    show_when_full: false,
                },
                StateIndicatorType::BehaviorState {
                    show_icon: true,
                    show_text: false,
                    color_coded: true,
                },
                StateIndicatorType::AlertLevel {
                    alert_radius: 8.0,
                    intensity_scaling: true,
                    color_gradient: true,
                },
            ],
            color_scheme: IndicatorColorScheme::default(),
            visibility_range: 15.0,
            always_visible: false,
            fade_distance: 3.0,
            update_frequency: 0.1,
            last_update: 0.0,
            indicator_data: HashMap::new(),
        }
    }
}

impl StateIndicatorComponent {
    /// Create indicators for a boss enemy
    pub fn boss_indicators() -> Self {
        StateIndicatorComponent {
            indicators: vec![
                StateIndicatorType::HealthBar {
                    show_percentage: true,
                    color_coding: true,
                    show_when_full: true,
                },
                StateIndicatorType::BehaviorState {
                    show_icon: true,
                    show_text: true,
                    color_coded: true,
                },
                StateIndicatorType::AggressionLevel {
                    show_intensity: true,
                    color_scaling: true,
                    particle_effects: true,
                },
                StateIndicatorType::PhaseTransition {
                    transition_name: "unknown".to_string(),
                    transition_progress: 0.0,
                    dramatic_effect: true,
                },
            ],
            color_scheme: IndicatorColorScheme::health_scheme(),
            always_visible: true,
            visibility_range: 25.0,
            ..Default::default()
        }
    }

    /// Create indicators for a stealth enemy
    pub fn stealth_indicators() -> Self {
        StateIndicatorComponent {
            indicators: vec![
                StateIndicatorType::Detection {
                    detection_cone: true,
                    hearing_radius: true,
                    line_of_sight: true,
                },
                StateIndicatorType::AlertLevel {
                    alert_radius: 10.0,
                    intensity_scaling: true,
                    color_gradient: true,
                },
            ],
            color_scheme: IndicatorColorScheme::alert_scheme(),
            visibility_range: 12.0,
            always_visible: false,
            ..Default::default()
        }
    }

    /// Create indicators for a spellcaster enemy
    pub fn spellcaster_indicators() -> Self {
        StateIndicatorComponent {
            indicators: vec![
                StateIndicatorType::HealthBar {
                    show_percentage: false,
                    color_coding: true,
                    show_when_full: false,
                },
                StateIndicatorType::AbilityCharging {
                    ability_name: "spell".to_string(),
                    charge_time: 0.0,
                    warning_color: Color::PURPLE,
                },
                StateIndicatorType::StatusEffect {
                    effect_name: "casting".to_string(),
                    duration: 0.0,
                    stacking: false,
                },
            ],
            color_scheme: IndicatorColorScheme::default(),
            ..Default::default()
        }
    }

    /// Update indicator data
    pub fn update_data(&mut self, key: String, value: f32) {
        self.indicator_data.insert(key, value);
    }

    /// Get indicator data
    pub fn get_data(&self, key: &str) -> f32 {
        self.indicator_data.get(key).copied().unwrap_or(0.0)
    }

    /// Check if indicators should be visible at distance
    pub fn should_show_at_distance(&self, distance: f32) -> bool {
        self.always_visible || distance <= self.visibility_range
    }

    /// Calculate fade alpha based on distance
    pub fn calculate_fade_alpha(&self, distance: f32) -> f32 {
        if self.always_visible {
            return 1.0;
        }

        if distance <= self.visibility_range - self.fade_distance {
            1.0
        } else if distance <= self.visibility_range {
            let fade_progress = (distance - (self.visibility_range - self.fade_distance)) / self.fade_distance;
            (1.0 - fade_progress).max(0.0)
        } else {
            0.0
        }
    }
}

/// Component for attack warning visuals
#[derive(Component, Debug, Clone)]
pub struct AttackWarningVisual {
    pub warning_type: String,
    pub intensity: f32,
    pub color: Color,
    pub affected_tiles: Vec<IVec2>,
    pub animation_phase: f32,
    pub pulse_speed: f32,
    pub show_countdown: bool,
    pub countdown_text: String,
}

impl AttackWarningVisual {
    pub fn new(warning_type: String, affected_tiles: Vec<IVec2>) -> Self {
        let color = match warning_type.as_str() {
            "aoe_attack" => Color::RED,
            "charge_attack" => Color::ORANGE,
            "teleport_attack" => Color::PURPLE,
            "combo_attack" => Color::YELLOW,
            _ => Color::WHITE,
        };

        AttackWarningVisual {
            warning_type,
            intensity: 1.0,
            color,
            affected_tiles,
            animation_phase: 0.0,
            pulse_speed: 2.0,
            show_countdown: true,
            countdown_text: String::new(),
        }
    }

    pub fn update(&mut self, delta_time: f32, time_remaining: f32) {
        self.animation_phase += delta_time * self.pulse_speed;
        self.intensity = (self.animation_phase.sin() * 0.3 + 0.7).max(0.1);
        
        if self.show_countdown && time_remaining > 0.0 {
            self.countdown_text = format!("{:.1}", time_remaining);
        } else {
            self.countdown_text.clear();
        }
    }
}

/// Component for behavior feedback visuals
#[derive(Component, Debug, Clone)]
pub struct BehaviorFeedbackVisual {
    pub current_behavior: AIBehaviorState,
    pub behavior_icon: String,
    pub behavior_color: Color,
    pub transition_animation: f32,
    pub show_behavior_text: bool,
    pub behavior_description: String,
}

impl BehaviorFeedbackVisual {
    pub fn new(behavior: AIBehaviorState) -> Self {
        let (icon, color, description) = Self::get_behavior_visual_data(&behavior);
        
        BehaviorFeedbackVisual {
            current_behavior: behavior,
            behavior_icon: icon,
            behavior_color: color,
            transition_animation: 0.0,
            show_behavior_text: false,
            behavior_description: description,
        }
    }

    pub fn update_behavior(&mut self, new_behavior: AIBehaviorState) {
        if new_behavior != self.current_behavior {
            self.current_behavior = new_behavior;
            let (icon, color, description) = Self::get_behavior_visual_data(&new_behavior);
            self.behavior_icon = icon;
            self.behavior_color = color;
            self.behavior_description = description;
            self.transition_animation = 1.0; // Start transition animation
        }
    }

    pub fn update(&mut self, delta_time: f32) {
        if self.transition_animation > 0.0 {
            self.transition_animation -= delta_time * 2.0;
            self.transition_animation = self.transition_animation.max(0.0);
        }
    }

    fn get_behavior_visual_data(behavior: &AIBehaviorState) -> (String, Color, String) {
        match behavior {
            AIBehaviorState::Idle => ("💤".to_string(), Color::GRAY, "Idle".to_string()),
            AIBehaviorState::Patrol => ("👁".to_string(), Color::BLUE, "Patrolling".to_string()),
            AIBehaviorState::Hunt => ("🎯".to_string(), Color::ORANGE, "Hunting".to_string()),
            AIBehaviorState::Attack => ("⚔".to_string(), Color::RED, "Attacking".to_string()),
            AIBehaviorState::Flee => ("💨".to_string(), Color::YELLOW, "Fleeing".to_string()),
            AIBehaviorState::Search => ("🔍".to_string(), Color::CYAN, "Searching".to_string()),
            AIBehaviorState::Guard => ("🛡".to_string(), Color::GREEN, "Guarding".to_string()),
            AIBehaviorState::Follow => ("👥".to_string(), Color::PURPLE, "Following".to_string()),
            AIBehaviorState::Wander => ("🚶".to_string(), Color::WHITE, "Wandering".to_string()),
            AIBehaviorState::Dead => ("💀".to_string(), Color::BLACK, "Dead".to_string()),
        }
    }
}

/// System for updating state indicators
pub fn state_indicator_system(
    time: Res<Time>,
    mut indicator_query: Query<(
        Entity,
        &mut StateIndicatorComponent,
        &AIComponent,
        &Health,
        &Position,
        Option<&SpecialEnemyComponent>,
    )>,
    player_query: Query<&Position, (With<crate::components::Player>, Without<AIComponent>)>,
) {
    let current_time = time.elapsed_seconds();
    let delta_time = time.delta_seconds();

    // Get player position for distance calculations
    let player_position = player_query.get_single().map(|pos| pos.0).unwrap_or(Vec2::ZERO);

    for (entity, mut indicators, ai, health, position, special_enemy) in indicator_query.iter_mut() {
        let distance_to_player = position.0.distance(player_position);

        // Skip update if too far and not always visible
        if !indicators.should_show_at_distance(distance_to_player) {
            continue;
        }

        // Update at specified frequency
        if current_time - indicators.last_update < indicators.update_frequency {
            continue;
        }
        indicators.last_update = current_time;

        // Update health data
        let health_percentage = health.current as f32 / health.max as f32;
        indicators.update_data("health_percentage".to_string(), health_percentage);
        indicators.update_data("health_current".to_string(), health.current as f32);
        indicators.update_data("health_max".to_string(), health.max as f32);

        // Update behavior data
        let behavior_intensity = match ai.current_state {
            AIBehaviorState::Idle => 0.1,
            AIBehaviorState::Patrol => 0.3,
            AIBehaviorState::Search => 0.5,
            AIBehaviorState::Hunt => 0.7,
            AIBehaviorState::Attack => 1.0,
            AIBehaviorState::Flee => 0.8,
            AIBehaviorState::Guard => 0.4,
            AIBehaviorState::Follow => 0.3,
            AIBehaviorState::Wander => 0.2,
            AIBehaviorState::Dead => 0.0,
        };
        indicators.update_data("behavior_intensity".to_string(), behavior_intensity);

        // Update aggression data
        indicators.update_data("aggression".to_string(), ai.personality.aggression);
        indicators.update_data("alertness".to_string(), ai.personality.alertness);
        indicators.update_data("courage".to_string(), ai.personality.courage);

        // Update detection data
        indicators.update_data("distance_to_target".to_string(), ai.decision_factors.distance_to_target);
        indicators.update_data("time_since_seen".to_string(), ai.decision_factors.time_since_last_seen_target);
        indicators.update_data("enemies_nearby".to_string(), ai.decision_factors.enemies_nearby as f32);

        // Update special enemy data
        if let Some(special) = special_enemy {
            indicators.update_data("is_enraged".to_string(), if special.is_enraged { 1.0 } else { 0.0 });
            indicators.update_data("enrage_threshold".to_string(), special.enrage_threshold);
            
            // Update phase data
            let phase_progress = match special.current_phase.as_str() {
                "normal" => 0.0,
                "enrage" => 0.5,
                "desperate" => 1.0,
                _ => 0.0,
            };
            indicators.update_data("phase_progress".to_string(), phase_progress);
        }

        // Update distance and visibility data
        indicators.update_data("distance_to_player".to_string(), distance_to_player);
        indicators.update_data("fade_alpha".to_string(), indicators.calculate_fade_alpha(distance_to_player));
    }
}

/// System for managing attack warning visuals
pub fn attack_warning_visual_system(
    time: Res<Time>,
    mut warning_query: Query<(&AttackWarning, &mut AttackWarningVisual)>,
    mut commands: Commands,
) {
    let delta_time = time.delta_seconds();

    for (warning, mut visual) in warning_query.iter_mut() {
        visual.update(delta_time, warning.warning_time_remaining);
    }
}

/// System for managing behavior feedback visuals
pub fn behavior_feedback_system(
    time: Res<Time>,
    mut feedback_query: Query<(&AIComponent, &mut BehaviorFeedbackVisual)>,
) {
    let delta_time = time.delta_seconds();

    for (ai, mut feedback) in feedback_query.iter_mut() {
        feedback.update_behavior(ai.current_state.clone());
        feedback.update(delta_time);
    }
}

/// System for creating attack warning visuals
pub fn attack_warning_creation_system(
    mut commands: Commands,
    warning_query: Query<(Entity, &AttackWarning), Added<AttackWarning>>,
) {
    for (entity, warning) in warning_query.iter() {
        let visual = AttackWarningVisual::new(
            warning.attack_type.clone(),
            warning.affected_area.clone(),
        );
        
        commands.entity(entity).insert(visual);
    }
}

/// System for creating behavior feedback visuals
pub fn behavior_feedback_creation_system(
    mut commands: Commands,
    ai_query: Query<(Entity, &AIComponent), (Added<AIComponent>, Without<BehaviorFeedbackVisual>)>,
) {
    for (entity, ai) in ai_query.iter() {
        let feedback = BehaviorFeedbackVisual::new(ai.current_state.clone());
        commands.entity(entity).insert(feedback);
    }
}

/// System for cleaning up expired warning visuals
pub fn warning_visual_cleanup_system(
    mut commands: Commands,
    warning_query: Query<Entity, (With<AttackWarningVisual>, Without<AttackWarning>)>,
) {
    for entity in warning_query.iter() {
        commands.entity(entity).remove::<AttackWarningVisual>();
    }
}

/// Utility functions for rendering indicators (would be used by rendering system)
pub mod indicator_rendering {
    use super::*;

    /// Calculate health bar color based on health percentage
    pub fn get_health_bar_color(health_percentage: f32, color_scheme: &IndicatorColorScheme) -> Color {
        if health_percentage > 0.7 {
            color_scheme.primary
        } else if health_percentage > 0.4 {
            color_scheme.secondary
        } else if health_percentage > 0.2 {
            color_scheme.warning
        } else {
            color_scheme.danger
        }
    }

    /// Calculate aggression indicator color
    pub fn get_aggression_color(aggression: f32, color_scheme: &IndicatorColorScheme) -> Color {
        if aggression > 0.8 {
            color_scheme.danger
        } else if aggression > 0.5 {
            color_scheme.warning
        } else if aggression > 0.2 {
            color_scheme.secondary
        } else {
            color_scheme.neutral
        }
    }

    /// Calculate alert level color
    pub fn get_alert_color(alertness: f32, color_scheme: &IndicatorColorScheme) -> Color {
        if alertness > 0.8 {
            color_scheme.danger
        } else if alertness > 0.6 {
            color_scheme.warning
        } else if alertness > 0.3 {
            color_scheme.secondary
        } else {
            color_scheme.primary
        }
    }

    /// Get behavior state color
    pub fn get_behavior_color(behavior: &AIBehaviorState) -> Color {
        match behavior {
            AIBehaviorState::Idle => Color::GRAY,
            AIBehaviorState::Patrol => Color::BLUE,
            AIBehaviorState::Hunt => Color::ORANGE,
            AIBehaviorState::Attack => Color::RED,
            AIBehaviorState::Flee => Color::YELLOW,
            AIBehaviorState::Search => Color::CYAN,
            AIBehaviorState::Guard => Color::GREEN,
            AIBehaviorState::Follow => Color::PURPLE,
            AIBehaviorState::Wander => Color::WHITE,
            AIBehaviorState::Dead => Color::BLACK,
        }
    }

    /// Calculate warning intensity based on time remaining
    pub fn calculate_warning_intensity(time_remaining: f32, total_time: f32) -> f32 {
        let progress = 1.0 - (time_remaining / total_time);
        (progress * 2.0).min(1.0)
    }
}

/// Plugin for state indicators
pub struct StateIndicatorsPlugin;

impl Plugin for StateIndicatorsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            state_indicator_system,
            attack_warning_visual_system,
            behavior_feedback_system,
            attack_warning_creation_system,
            behavior_feedback_creation_system,
            warning_visual_cleanup_system,
        ).chain());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_indicator_component() {
        let mut indicators = StateIndicatorComponent::default();
        
        // Test data updates
        indicators.update_data("health".to_string(), 0.5);
        assert_eq!(indicators.get_data("health"), 0.5);
        assert_eq!(indicators.get_data("nonexistent"), 0.0);
        
        // Test visibility
        assert!(indicators.should_show_at_distance(10.0));
        assert!(!indicators.should_show_at_distance(20.0));
        
        // Test fade alpha
        let alpha = indicators.calculate_fade_alpha(10.0);
        assert!(alpha > 0.0 && alpha <= 1.0);
    }

    #[test]
    fn test_boss_indicators() {
        let indicators = StateIndicatorComponent::boss_indicators();
        assert!(indicators.always_visible);
        assert_eq!(indicators.visibility_range, 25.0);
        assert!(indicators.indicators.len() >= 3);
    }

    #[test]
    fn test_attack_warning_visual() {
        let tiles = vec![IVec2::new(0, 0), IVec2::new(1, 0)];
        let mut visual = AttackWarningVisual::new("aoe_attack".to_string(), tiles.clone());
        
        assert_eq!(visual.warning_type, "aoe_attack");
        assert_eq!(visual.affected_tiles, tiles);
        assert_eq!(visual.color, Color::RED);
        
        // Test update
        visual.update(0.1, 2.0);
        assert!(!visual.countdown_text.is_empty());
    }

    #[test]
    fn test_behavior_feedback_visual() {
        let mut feedback = BehaviorFeedbackVisual::new(AIBehaviorState::Idle);
        assert_eq!(feedback.current_behavior, AIBehaviorState::Idle);
        assert_eq!(feedback.behavior_color, Color::GRAY);
        
        // Test behavior update
        feedback.update_behavior(AIBehaviorState::Attack);
        assert_eq!(feedback.current_behavior, AIBehaviorState::Attack);
        assert_eq!(feedback.behavior_color, Color::RED);
        assert!(feedback.transition_animation > 0.0);
    }

    #[test]
    fn test_color_schemes() {
        let health_scheme = IndicatorColorScheme::health_scheme();
        assert_eq!(health_scheme.primary, Color::GREEN);
        assert_eq!(health_scheme.danger, Color::RED);
        
        let aggression_scheme = IndicatorColorScheme::aggression_scheme();
        assert_eq!(aggression_scheme.primary, Color::BLUE);
        
        let alert_scheme = IndicatorColorScheme::alert_scheme();
        assert_eq!(alert_scheme.primary, Color::GREEN);
    }

    #[test]
    fn test_indicator_rendering_utilities() {
        use indicator_rendering::*;
        
        let scheme = IndicatorColorScheme::health_scheme();
        
        // Test health bar colors
        let high_health_color = get_health_bar_color(0.9, &scheme);
        assert_eq!(high_health_color, scheme.primary);
        
        let low_health_color = get_health_bar_color(0.1, &scheme);
        assert_eq!(low_health_color, scheme.danger);
        
        // Test aggression colors
        let high_aggression = get_aggression_color(0.9, &scheme);
        assert_eq!(high_aggression, scheme.danger);
        
        let low_aggression = get_aggression_color(0.1, &scheme);
        assert_eq!(low_aggression, scheme.neutral);
        
        // Test behavior colors
        let attack_color = get_behavior_color(&AIBehaviorState::Attack);
        assert_eq!(attack_color, Color::RED);
        
        let idle_color = get_behavior_color(&AIBehaviorState::Idle);
        assert_eq!(idle_color, Color::GRAY);
        
        // Test warning intensity
        let intensity = calculate_warning_intensity(1.0, 2.0);
        assert_eq!(intensity, 1.0);
        
        let early_intensity = calculate_warning_intensity(1.8, 2.0);
        assert!(early_intensity < 1.0);
    }
}