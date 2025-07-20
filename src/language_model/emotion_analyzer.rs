use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use log::{info, warn, error};

use super::response_processor::DetectedEmotion;

/// Emotion state for a character
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmotionState {
    pub primary_emotion: String,
    pub intensity: u8, // 0-100
    pub secondary_emotions: Vec<(String, u8)>, // (emotion, intensity)
    pub stability: f32, // 0.0-1.0, how stable the emotion is
    pub duration: u64, // How long this emotion has been active (seconds)
    pub triggers: Vec<String>, // What triggered this emotion
    pub timestamp: u64,
}

/// Emotion transition rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmotionTransition {
    pub from_emotion: String,
    pub to_emotion: String,
    pub trigger_words: Vec<String>,
    pub probability: f32, // 0.0-1.0
    pub intensity_change: i8, // -100 to 100
    pub conditions: Vec<TransitionCondition>,
}

/// Condition for emotion transition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransitionCondition {
    MinIntensity(u8),
    MaxIntensity(u8),
    TimeElapsed(u64), // Seconds
    ContextContains(String),
    RelationshipValue(i32), // Minimum relationship value
    Custom(String, String),
}

/// Emotion profile for a character
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmotionProfile {
    pub character_id: String,
    pub default_emotion: String,
    pub emotional_range: Vec<String>, // Emotions this character can express
    pub stability_factor: f32, // How quickly emotions change (0.0-1.0)
    pub intensity_modifier: f32, // Multiplier for emotion intensity
    pub transitions: Vec<EmotionTransition>,
    pub emotion_memory: HashMap<String, u64>, // Last time each emotion was felt
}

/// Advanced emotion analyzer
pub struct EmotionAnalyzer {
    emotion_profiles: HashMap<String, EmotionProfile>,
    current_states: HashMap<String, EmotionState>,
    emotion_weights: HashMap<String, f32>,
    context_modifiers: HashMap<String, f32>,
}

impl EmotionAnalyzer {
    /// Create a new emotion analyzer
    pub fn new() -> Self {
        let mut analyzer = EmotionAnalyzer {
            emotion_profiles: HashMap::new(),
            current_states: HashMap::new(),
            emotion_weights: HashMap::new(),
            context_modifiers: HashMap::new(),
        };
        
        analyzer.initialize_emotion_weights();
        analyzer.initialize_context_modifiers();
        
        analyzer
    }
    
    /// Initialize emotion weights
    fn initialize_emotion_weights(&mut self) {
        // Base weights for different emotions
        self.emotion_weights.insert("happy".to_string(), 1.0);
        self.emotion_weights.insert("sad".to_string(), 0.8);
        self.emotion_weights.insert("angry".to_string(), 1.2);
        self.emotion_weights.insert("confused".to_string(), 0.6);
        self.emotion_weights.insert("surprised".to_string(), 0.9);
        self.emotion_weights.insert("curious".to_string(), 0.7);
        self.emotion_weights.insert("concerned".to_string(), 0.8);
        self.emotion_weights.insert("calm".to_string(), 0.5);
        self.emotion_weights.insert("excited".to_string(), 1.1);
        self.emotion_weights.insert("fearful".to_string(), 1.0);
    }
    
    /// Initialize context modifiers
    fn initialize_context_modifiers(&mut self) {
        // Context-based emotion modifiers
        self.context_modifiers.insert("combat".to_string(), 1.5);
        self.context_modifiers.insert("peaceful".to_string(), 0.7);
        self.context_modifiers.insert("mysterious".to_string(), 1.2);
        self.context_modifiers.insert("familiar".to_string(), 0.8);
        self.context_modifiers.insert("dangerous".to_string(), 1.3);
        self.context_modifiers.insert("safe".to_string(), 0.6);
    }
    
    /// Create an emotion profile for a character
    pub fn create_emotion_profile(&mut self, character_id: &str, default_emotion: &str, traits: &[String]) -> EmotionProfile {
        let mut emotional_range = vec![
            "happy".to_string(),
            "sad".to_string(),
            "angry".to_string(),
            "confused".to_string(),
            "surprised".to_string(),
            "curious".to_string(),
            "concerned".to_string(),
            "calm".to_string(),
        ];
        
        // Adjust emotional range based on character traits
        for trait_name in traits {
            match trait_name.as_str() {
                "wise" => {
                    emotional_range.push("contemplative".to_string());
                    emotional_range.push("patient".to_string());
                },
                "aggressive" => {
                    emotional_range.push("furious".to_string());
                    emotional_range.push("hostile".to_string());
                },
                "cheerful" => {
                    emotional_range.push("joyful".to_string());
                    emotional_range.push("excited".to_string());
                },
                "mysterious" => {
                    emotional_range.push("secretive".to_string());
                    emotional_range.push("enigmatic".to_string());
                },
                _ => {}
            }
        }
        
        // Create default transitions
        let transitions = self.create_default_transitions(&emotional_range);
        
        // Calculate stability factor based on traits
        let stability_factor = self.calculate_stability_factor(traits);
        
        let profile = EmotionProfile {
            character_id: character_id.to_string(),
            default_emotion: default_emotion.to_string(),
            emotional_range,
            stability_factor,
            intensity_modifier: 1.0,
            transitions,
            emotion_memory: HashMap::new(),
        };
        
        // Initialize current state
        self.current_states.insert(character_id.to_string(), EmotionState {
            primary_emotion: default_emotion.to_string(),
            intensity: 30, // Mild default intensity
            secondary_emotions: Vec::new(),
            stability: stability_factor,
            duration: 0,
            triggers: Vec::new(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        });
        
        self.emotion_profiles.insert(character_id.to_string(), profile.clone());
        
        profile
    }
    
    /// Create default emotion transitions
    fn create_default_transitions(&self, emotional_range: &[String]) -> Vec<EmotionTransition> {
        let mut transitions = Vec::new();
        
        // Common transitions
        transitions.push(EmotionTransition {
            from_emotion: "calm".to_string(),
            to_emotion: "surprised".to_string(),
            trigger_words: vec!["what".to_string(), "unexpected".to_string(), "suddenly".to_string()],
            probability: 0.7,
            intensity_change: 40,
            conditions: vec![TransitionCondition::MinIntensity(20)],
        });
        
        transitions.push(EmotionTransition {
            from_emotion: "surprised".to_string(),
            to_emotion: "curious".to_string(),
            trigger_words: vec!["interesting".to_string(), "tell me".to_string(), "explain".to_string()],
            probability: 0.6,
            intensity_change: 20,
            conditions: vec![TransitionCondition::TimeElapsed(5)],
        });
        
        transitions.push(EmotionTransition {
            from_emotion: "curious".to_string(),
            to_emotion: "satisfied".to_string(),
            trigger_words: vec!["understand".to_string(), "clear".to_string(), "makes sense".to_string()],
            probability: 0.8,
            intensity_change: -20,
            conditions: vec![],
        });
        
        transitions.push(EmotionTransition {
            from_emotion: "happy".to_string(),
            to_emotion: "concerned".to_string(),
            trigger_words: vec!["danger".to_string(), "careful".to_string(), "worried".to_string()],
            probability: 0.5,
            intensity_change: 30,
            conditions: vec![],
        });
        
        transitions.push(EmotionTransition {
            from_emotion: "angry".to_string(),
            to_emotion: "calm".to_string(),
            trigger_words: vec!["sorry".to_string(), "apologize".to_string(), "peace".to_string()],
            probability: 0.4,
            intensity_change: -50,
            conditions: vec![TransitionCondition::TimeElapsed(10)],
        });
        
        // Add more transitions based on emotional range
        for emotion in emotional_range {
            if emotion != "calm" {
                transitions.push(EmotionTransition {
                    from_emotion: emotion.clone(),
                    to_emotion: "calm".to_string(),
                    trigger_words: vec!["relax".to_string(), "peaceful".to_string()],
                    probability: 0.3,
                    intensity_change: -30,
                    conditions: vec![TransitionCondition::TimeElapsed(30)],
                });
            }
        }
        
        transitions
    }
    
    /// Calculate stability factor based on character traits
    fn calculate_stability_factor(&self, traits: &[String]) -> f32 {
        let mut stability = 0.5; // Base stability
        
        for trait_name in traits {
            match trait_name.as_str() {
                "calm" | "patient" | "wise" => stability += 0.2,
                "volatile" | "impulsive" | "emotional" => stability -= 0.2,
                "stoic" | "controlled" => stability += 0.3,
                "passionate" | "dramatic" => stability -= 0.1,
                _ => {}
            }
        }
        
        stability.clamp(0.1, 1.0)
    }
    
    /// Analyze emotions in detected emotions and update character state
    pub fn analyze_emotions(&mut self, character_id: &str, detected_emotions: &[DetectedEmotion], context: &str) -> Option<EmotionState> {
        let profile = self.emotion_profiles.get(character_id)?.clone();
        let current_state = self.current_states.get(character_id)?.clone();
        
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        // Calculate new emotion state
        let new_state = self.calculate_new_emotion_state(
            &profile,
            &current_state,
            detected_emotions,
            context,
            now
        );
        
        // Update current state
        self.current_states.insert(character_id.to_string(), new_state.clone());
        
        Some(new_state)
    }
    
    /// Calculate new emotion state
    fn calculate_new_emotion_state(
        &self,
        profile: &EmotionProfile,
        current_state: &EmotionState,
        detected_emotions: &[DetectedEmotion],
        context: &str,
        timestamp: u64
    ) -> EmotionState {
        let duration = timestamp - current_state.timestamp;
        
        // Find the strongest detected emotion that the character can express
        let mut new_primary_emotion = current_state.primary_emotion.clone();
        let mut new_intensity = current_state.intensity;
        let mut triggers = Vec::new();
        
        for detected in detected_emotions {
            if profile.emotional_range.contains(&detected.emotion) {
                // Check if this emotion should trigger a transition
                if let Some(transition) = self.find_applicable_transition(
                    profile,
                    &current_state.primary_emotion,
                    &detected.emotion,
                    context,
                    duration
                ) {
                    if self.should_transition(transition, detected.confidence) {
                        new_primary_emotion = detected.emotion.clone();
                        new_intensity = self.calculate_new_intensity(
                            current_state.intensity,
                            detected.intensity,
                            transition.intensity_change,
                            profile.intensity_modifier
                        );
                        triggers.extend(detected.indicators.clone());
                        break;
                    }
                }
            }
        }
        
        // Apply context modifiers
        let context_modifier = self.get_context_modifier(context);
        new_intensity = ((new_intensity as f32 * context_modifier) as u8).clamp(0, 100);
        
        // Apply stability factor (emotions gradually return to default)
        if new_primary_emotion == current_state.primary_emotion && duration > 60 {
            let decay_factor = 1.0 - (profile.stability_factor * 0.1);
            new_intensity = ((new_intensity as f32 * decay_factor) as u8).max(10);
            
            // Gradually return to default emotion if intensity is very low
            if new_intensity < 20 && new_primary_emotion != profile.default_emotion {
                new_primary_emotion = profile.default_emotion.clone();
                new_intensity = 30;
            }
        }
        
        // Calculate secondary emotions
        let secondary_emotions = self.calculate_secondary_emotions(
            detected_emotions,
            &new_primary_emotion,
            profile
        );
        
        EmotionState {
            primary_emotion: new_primary_emotion,
            intensity: new_intensity,
            secondary_emotions,
            stability: profile.stability_factor,
            duration: if new_primary_emotion == current_state.primary_emotion {
                current_state.duration + duration
            } else {
                0
            },
            triggers,
            timestamp,
        }
    }
    
    /// Find applicable emotion transition
    fn find_applicable_transition(
        &self,
        profile: &EmotionProfile,
        from_emotion: &str,
        to_emotion: &str,
        context: &str,
        duration: u64
    ) -> Option<&EmotionTransition> {
        for transition in &profile.transitions {
            if transition.from_emotion == from_emotion && transition.to_emotion == to_emotion {
                // Check if trigger words are present in context
                let has_trigger = transition.trigger_words.iter()
                    .any(|word| context.to_lowercase().contains(&word.to_lowercase()));
                
                if has_trigger && self.check_transition_conditions(&transition.conditions, duration, context) {
                    return Some(transition);
                }
            }
        }
        
        None
    }
    
    /// Check if transition conditions are met
    fn check_transition_conditions(&self, conditions: &[TransitionCondition], duration: u64, context: &str) -> bool {
        for condition in conditions {
            match condition {
                TransitionCondition::TimeElapsed(min_time) => {
                    if duration < *min_time {
                        return false;
                    }
                },
                TransitionCondition::ContextContains(text) => {
                    if !context.to_lowercase().contains(&text.to_lowercase()) {
                        return false;
                    }
                },
                _ => {} // Other conditions would be checked with additional context
            }
        }
        
        true
    }
    
    /// Determine if transition should occur based on probability
    fn should_transition(&self, transition: &EmotionTransition, confidence: f32) -> bool {
        let adjusted_probability = transition.probability * confidence;
        rand::random::<f32>() < adjusted_probability
    }
    
    /// Calculate new emotion intensity
    fn calculate_new_intensity(&self, current: u8, detected: u8, change: i8, modifier: f32) -> u8 {
        let base_intensity = if change > 0 {
            current.max(detected)
        } else {
            current.min(detected)
        };
        
        let adjusted_change = (change as f32 * modifier) as i8;
        let new_intensity = (base_intensity as i8 + adjusted_change).clamp(0, 100) as u8;
        
        new_intensity
    }
    
    /// Get context modifier for emotion intensity
    fn get_context_modifier(&self, context: &str) -> f32 {
        let context_lower = context.to_lowercase();
        
        for (context_key, modifier) in &self.context_modifiers {
            if context_lower.contains(context_key) {
                return *modifier;
            }
        }
        
        1.0 // Default modifier
    }
    
    /// Calculate secondary emotions
    fn calculate_secondary_emotions(
        &self,
        detected_emotions: &[DetectedEmotion],
        primary_emotion: &str,
        profile: &EmotionProfile
    ) -> Vec<(String, u8)> {
        let mut secondary = Vec::new();
        
        for detected in detected_emotions {
            if detected.emotion != primary_emotion && profile.emotional_range.contains(&detected.emotion) {
                // Secondary emotions have reduced intensity
                let intensity = (detected.intensity as f32 * 0.6) as u8;
                if intensity > 20 { // Only include significant secondary emotions
                    secondary.push((detected.emotion.clone(), intensity));
                }
            }
        }
        
        // Sort by intensity (highest first) and limit to top 2
        secondary.sort_by(|a, b| b.1.cmp(&a.1));
        secondary.truncate(2);
        
        secondary
    }
    
    /// Get current emotion state for a character
    pub fn get_current_state(&self, character_id: &str) -> Option<&EmotionState> {
        self.current_states.get(character_id)
    }
    
    /// Get emotion profile for a character
    pub fn get_emotion_profile(&self, character_id: &str) -> Option<&EmotionProfile> {
        self.emotion_profiles.get(character_id)
    }
    
    /// Update emotion profile
    pub fn update_emotion_profile(&mut self, character_id: &str, profile: EmotionProfile) {
        self.emotion_profiles.insert(character_id.to_string(), profile);
    }
    
    /// Add custom emotion transition
    pub fn add_emotion_transition(&mut self, character_id: &str, transition: EmotionTransition) {
        if let Some(profile) = self.emotion_profiles.get_mut(character_id) {
            profile.transitions.push(transition);
        }
    }
    
    /// Generate emotion description for dialogue
    pub fn generate_emotion_description(&self, state: &EmotionState) -> String {
        let intensity_desc = match state.intensity {
            0..=20 => "slightly",
            21..=40 => "somewhat",
            41..=60 => "moderately",
            61..=80 => "quite",
            81..=100 => "very",
        };
        
        let mut description = format!("{} {}", intensity_desc, state.primary_emotion);
        
        if !state.secondary_emotions.is_empty() {
            let secondary_desc: Vec<String> = state.secondary_emotions.iter()
                .map(|(emotion, intensity)| {
                    let sec_intensity_desc = match *intensity {
                        0..=30 => "slightly",
                        31..=60 => "somewhat",
                        61..=100 => "quite",
                    };
                    format!("{} {}", sec_intensity_desc, emotion)
                })
                .collect();
            
            description.push_str(&format!(" with hints of {}", secondary_desc.join(" and ")));
        }
        
        description
    }
    
    /// Reset character emotion to default
    pub fn reset_emotion(&mut self, character_id: &str) {
        if let Some(profile) = self.emotion_profiles.get(character_id) {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            
            self.current_states.insert(character_id.to_string(), EmotionState {
                primary_emotion: profile.default_emotion.clone(),
                intensity: 30,
                secondary_emotions: Vec::new(),
                stability: profile.stability_factor,
                duration: 0,
                triggers: Vec::new(),
                timestamp: now,
            });
        }
    }
}