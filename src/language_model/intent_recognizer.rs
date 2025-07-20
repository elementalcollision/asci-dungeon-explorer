use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use log::{info, warn, error};

use super::response_processor::DetectedIntent;

/// Action that can be triggered by an intent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentAction {
    pub action_type: ActionType,
    pub parameters: HashMap<String, String>,
    pub priority: u8, // 0-100, higher is more important
    pub conditions: Vec<ActionCondition>,
}

/// Type of action that can be performed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActionType {
    GiveItem(String),
    StartQuest(String),
    ProvideInformation(String),
    ChangeRelationship(i32),
    TriggerEvent(String),
    MoveToLocation(String),
    CallNPC(String),
    ShowEmotion(String),
    EndConversation,
    Custom(String, String),
}

/// Condition for an action to be performed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActionCondition {
    HasItem(String),
    LacksItem(String),
    MinRelationship(i32),
    MaxRelationship(i32),
    QuestStatus(String, String), // Quest ID, Status
    LocationIs(String),
    TimeOfDay(String),
    Custom(String, String),
}

/// Intent pattern for recognition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentPattern {
    pub intent_name: String,
    pub keywords: Vec<String>,
    pub phrases: Vec<String>,
    pub context_clues: Vec<String>,
    pub confidence_threshold: f32,
    pub actions: Vec<IntentAction>,
}

/// Intent recognition result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentResult {
    pub intent: DetectedIntent,
    pub suggested_actions: Vec<IntentAction>,
    pub confidence_score: f32,
    pub context_relevance: f32,
}

/// Advanced intent recognizer
pub struct IntentRecognizer {
    intent_patterns: HashMap<String, IntentPattern>,
    character_intents: HashMap<String, Vec<String>>, // Character ID -> Available intents
    context_weights: HashMap<String, f32>,
    action_history: HashMap<String, Vec<(String, u64)>>, // Character ID -> (Action, Timestamp)
}

impl IntentRecognizer {
    /// Create a new intent recognizer
    pub fn new() -> Self {
        let mut recognizer = IntentRecognizer {
            intent_patterns: HashMap::new(),
            character_intents: HashMap::new(),
            context_weights: HashMap::new(),
            action_history: HashMap::new(),
        };
        
        recognizer.initialize_default_patterns();
        recognizer.initialize_context_weights();
        
        recognizer
    }
    
    /// Initialize default intent patterns
    fn initialize_default_patterns(&mut self) {
        // Greeting intent
        self.add_intent_pattern(IntentPattern {
            intent_name: "greeting".to_string(),
            keywords: vec!["hello".to_string(), "hi".to_string(), "greetings".to_string()],
            phrases: vec!["good day".to_string(), "pleased to meet".to_string()],
            context_clues: vec!["first time".to_string(), "new".to_string()],
            confidence_threshold: 0.6,
            actions: vec![
                IntentAction {
                    action_type: ActionType::ShowEmotion("friendly".to_string()),
                    parameters: HashMap::new(),
                    priority: 70,
                    conditions: vec![ActionCondition::MinRelationship(-50)],
                },
                IntentAction {
                    action_type: ActionType::ProvideInformation("greeting_response".to_string()),
                    parameters: HashMap::new(),
                    priority: 80,
                    conditions: vec![],
                },
            ],
        });
        
        // Information request intent
        self.add_intent_pattern(IntentPattern {
            intent_name: "information_request".to_string(),
            keywords: vec!["tell".to_string(), "know".to_string(), "about".to_string(), "what".to_string()],
            phrases: vec!["tell me about".to_string(), "do you know".to_string(), "what can you tell me".to_string()],
            context_clues: vec!["information".to_string(), "knowledge".to_string()],
            confidence_threshold: 0.7,
            actions: vec![
                IntentAction {
                    action_type: ActionType::ProvideInformation("requested_topic".to_string()),
                    parameters: HashMap::new(),
                    priority: 90,
                    conditions: vec![ActionCondition::MinRelationship(0)],
                },
                IntentAction {
                    action_type: ActionType::ShowEmotion("thoughtful".to_string()),
                    parameters: HashMap::new(),
                    priority: 50,
                    conditions: vec![],
                },
            ],
        });
        
        // Help request intent
        self.add_intent_pattern(IntentPattern {
            intent_name: "help_request".to_string(),
            keywords: vec!["help".to_string(), "assist".to_string(), "aid".to_string()],
            phrases: vec!["can you help".to_string(), "need assistance".to_string(), "could you aid".to_string()],
            context_clues: vec!["problem".to_string(), "trouble".to_string(), "difficulty".to_string()],
            confidence_threshold: 0.8,
            actions: vec![
                IntentAction {
                    action_type: ActionType::ProvideInformation("help_offer".to_string()),
                    parameters: HashMap::new(),
                    priority: 85,
                    conditions: vec![ActionCondition::MinRelationship(20)],
                },
                IntentAction {
                    action_type: ActionType::ShowEmotion("helpful".to_string()),
                    parameters: HashMap::new(),
                    priority: 60,
                    conditions: vec![],
                },
            ],
        });
        
        // Quest inquiry intent
        self.add_intent_pattern(IntentPattern {
            intent_name: "quest_inquiry".to_string(),
            keywords: vec!["quest".to_string(), "mission".to_string(), "task".to_string(), "job".to_string()],
            phrases: vec!["any quests".to_string(), "need help with".to_string(), "something to do".to_string()],
            context_clues: vec!["adventure".to_string(), "work".to_string(), "reward".to_string()],
            confidence_threshold: 0.75,
            actions: vec![
                IntentAction {
                    action_type: ActionType::StartQuest("available_quest".to_string()),
                    parameters: HashMap::new(),
                    priority: 95,
                    conditions: vec![
                        ActionCondition::MinRelationship(30),
                        ActionCondition::QuestStatus("prerequisite".to_string(), "completed".to_string()),
                    ],
                },
                IntentAction {
                    action_type: ActionType::ProvideInformation("quest_info".to_string()),
                    parameters: HashMap::new(),
                    priority: 70,
                    conditions: vec![],
                },
            ],
        });
        
        // Trade intent
        self.add_intent_pattern(IntentPattern {
            intent_name: "trade".to_string(),
            keywords: vec!["buy".to_string(), "sell".to_string(), "trade".to_string(), "purchase".to_string()],
            phrases: vec!["want to buy".to_string(), "looking to sell".to_string(), "interested in trading".to_string()],
            context_clues: vec!["gold".to_string(), "coins".to_string(), "price".to_string()],
            confidence_threshold: 0.8,
            actions: vec![
                IntentAction {
                    action_type: ActionType::ProvideInformation("trade_options".to_string()),
                    parameters: HashMap::new(),
                    priority: 90,
                    conditions: vec![ActionCondition::MinRelationship(0)],
                },
                IntentAction {
                    action_type: ActionType::ShowEmotion("business-like".to_string()),
                    parameters: HashMap::new(),
                    priority: 40,
                    conditions: vec![],
                },
            ],
        });
        
        // Farewell intent
        self.add_intent_pattern(IntentPattern {
            intent_name: "farewell".to_string(),
            keywords: vec!["goodbye".to_string(), "bye".to_string(), "farewell".to_string()],
            phrases: vec!["see you later".to_string(), "until next time".to_string(), "safe travels".to_string()],
            context_clues: vec!["leaving".to_string(), "going".to_string()],
            confidence_threshold: 0.7,
            actions: vec![
                IntentAction {
                    action_type: ActionType::ShowEmotion("polite".to_string()),
                    parameters: HashMap::new(),
                    priority: 60,
                    conditions: vec![],
                },
                IntentAction {
                    action_type: ActionType::EndConversation,
                    parameters: HashMap::new(),
                    priority: 80,
                    conditions: vec![],
                },
            ],
        });
        
        // Threat intent
        self.add_intent_pattern(IntentPattern {
            intent_name: "threat".to_string(),
            keywords: vec!["kill".to_string(), "hurt".to_string(), "attack".to_string(), "destroy".to_string()],
            phrases: vec!["I'll kill you".to_string(), "you're dead".to_string(), "prepare to die".to_string()],
            context_clues: vec!["weapon".to_string(), "fight".to_string(), "violence".to_string()],
            confidence_threshold: 0.9,
            actions: vec![
                IntentAction {
                    action_type: ActionType::ChangeRelationship(-50),
                    parameters: HashMap::new(),
                    priority: 100,
                    conditions: vec![],
                },
                IntentAction {
                    action_type: ActionType::ShowEmotion("angry".to_string()),
                    parameters: HashMap::new(),
                    priority: 90,
                    conditions: vec![],
                },
                IntentAction {
                    action_type: ActionType::TriggerEvent("combat_warning".to_string()),
                    parameters: HashMap::new(),
                    priority: 95,
                    conditions: vec![],
                },
            ],
        });
    }
    
    /// Initialize context weights
    fn initialize_context_weights(&mut self) {
        self.context_weights.insert("first_meeting".to_string(), 1.2);
        self.context_weights.insert("repeat_conversation".to_string(), 0.8);
        self.context_weights.insert("urgent_situation".to_string(), 1.5);
        self.context_weights.insert("casual_chat".to_string(), 0.9);
        self.context_weights.insert("formal_setting".to_string(), 1.1);
        self.context_weights.insert("combat_nearby".to_string(), 1.3);
    }
    
    /// Add an intent pattern
    pub fn add_intent_pattern(&mut self, pattern: IntentPattern) {
        self.intent_patterns.insert(pattern.intent_name.clone(), pattern);
    }
    
    /// Set available intents for a character
    pub fn set_character_intents(&mut self, character_id: &str, intents: Vec<String>) {
        self.character_intents.insert(character_id.to_string(), intents);
    }
    
    /// Recognize intents in detected intents and suggest actions
    pub fn recognize_intents(
        &mut self,
        character_id: &str,
        detected_intents: &[DetectedIntent],
        context: &str,
        relationship_value: i32
    ) -> Vec<IntentResult> {
        let mut results = Vec::new();
        
        // Get available intents for this character
        let available_intents = self.character_intents.get(character_id)
            .cloned()
            .unwrap_or_else(|| self.intent_patterns.keys().cloned().collect());
        
        for detected in detected_intents {
            if let Some(pattern) = self.intent_patterns.get(&detected.intent) {
                if available_intents.contains(&detected.intent) {
                    // Calculate context relevance
                    let context_relevance = self.calculate_context_relevance(pattern, context);
                    
                    // Calculate overall confidence
                    let confidence_score = detected.confidence * context_relevance;
                    
                    if confidence_score >= pattern.confidence_threshold {
                        // Filter actions based on conditions
                        let suggested_actions = self.filter_actions_by_conditions(
                            &pattern.actions,
                            character_id,
                            relationship_value,
                            context
                        );
                        
                        results.push(IntentResult {
                            intent: detected.clone(),
                            suggested_actions,
                            confidence_score,
                            context_relevance,
                        });
                        
                        // Record this intent in action history
                        self.record_intent_action(character_id, &detected.intent);
                    }
                }
            }
        }
        
        // Sort by confidence score (highest first)
        results.sort_by(|a, b| b.confidence_score.partial_cmp(&a.confidence_score).unwrap_or(std::cmp::Ordering::Equal));
        
        results
    }
    
    /// Calculate context relevance for an intent pattern
    fn calculate_context_relevance(&self, pattern: &IntentPattern, context: &str) -> f32 {
        let context_lower = context.to_lowercase();
        let mut relevance = 1.0;
        
        // Check for context clues
        let mut clue_matches = 0;
        for clue in &pattern.context_clues {
            if context_lower.contains(&clue.to_lowercase()) {
                clue_matches += 1;
            }
        }
        
        if !pattern.context_clues.is_empty() {
            relevance *= 0.8 + (0.4 * clue_matches as f32 / pattern.context_clues.len() as f32);
        }
        
        // Apply context weights
        for (context_key, weight) in &self.context_weights {
            if context_lower.contains(context_key) {
                relevance *= weight;
                break;
            }
        }
        
        relevance.clamp(0.1, 2.0)
    }
    
    /// Filter actions based on conditions
    fn filter_actions_by_conditions(
        &self,
        actions: &[IntentAction],
        character_id: &str,
        relationship_value: i32,
        context: &str
    ) -> Vec<IntentAction> {
        let mut filtered_actions = Vec::new();
        
        for action in actions {
            if self.check_action_conditions(&action.conditions, character_id, relationship_value, context) {
                filtered_actions.push(action.clone());
            }
        }
        
        // Sort by priority (highest first)
        filtered_actions.sort_by(|a, b| b.priority.cmp(&a.priority));
        
        filtered_actions
    }
    
    /// Check if action conditions are met
    fn check_action_conditions(
        &self,
        conditions: &[ActionCondition],
        character_id: &str,
        relationship_value: i32,
        context: &str
    ) -> bool {
        for condition in conditions {
            match condition {
                ActionCondition::MinRelationship(min_value) => {
                    if relationship_value < *min_value {
                        return false;
                    }
                },
                ActionCondition::MaxRelationship(max_value) => {
                    if relationship_value > *max_value {
                        return false;
                    }
                },
                ActionCondition::LocationIs(location) => {
                    if !context.to_lowercase().contains(&location.to_lowercase()) {
                        return false;
                    }
                },
                // Other conditions would require additional game state context
                _ => {}
            }
        }
        
        true
    }
    
    /// Record an intent action in history
    fn record_intent_action(&mut self, character_id: &str, intent: &str) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        self.action_history
            .entry(character_id.to_string())
            .or_insert_with(Vec::new)
            .push((intent.to_string(), now));
        
        // Keep only recent history (last 50 actions)
        let history = self.action_history.get_mut(character_id).unwrap();
        if history.len() > 50 {
            history.remove(0);
        }
    }
    
    /// Get recent intent actions for a character
    pub fn get_recent_actions(&self, character_id: &str, limit: usize) -> Vec<&(String, u64)> {
        if let Some(history) = self.action_history.get(character_id) {
            history.iter().rev().take(limit).collect()
        } else {
            Vec::new()
        }
    }
    
    /// Check if an intent was recently used
    pub fn was_intent_recently_used(&self, character_id: &str, intent: &str, within_seconds: u64) -> bool {
        if let Some(history) = self.action_history.get(character_id) {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            
            for (action_intent, timestamp) in history.iter().rev() {
                if action_intent == intent && (now - timestamp) <= within_seconds {
                    return true;
                }
            }
        }
        
        false
    }
    
    /// Generate action description
    pub fn generate_action_description(&self, action: &IntentAction) -> String {
        match &action.action_type {
            ActionType::GiveItem(item) => format!("Give item: {}", item),
            ActionType::StartQuest(quest) => format!("Start quest: {}", quest),
            ActionType::ProvideInformation(info) => format!("Provide information: {}", info),
            ActionType::ChangeRelationship(change) => {
                if *change > 0 {
                    format!("Improve relationship by {}", change)
                } else {
                    format!("Worsen relationship by {}", change.abs())
                }
            },
            ActionType::TriggerEvent(event) => format!("Trigger event: {}", event),
            ActionType::MoveToLocation(location) => format!("Move to location: {}", location),
            ActionType::CallNPC(npc) => format!("Call NPC: {}", npc),
            ActionType::ShowEmotion(emotion) => format!("Show emotion: {}", emotion),
            ActionType::EndConversation => "End conversation".to_string(),
            ActionType::Custom(action_type, description) => format!("{}: {}", action_type, description),
        }
    }
    
    /// Get intent pattern
    pub fn get_intent_pattern(&self, intent_name: &str) -> Option<&IntentPattern> {
        self.intent_patterns.get(intent_name)
    }
    
    /// Update intent pattern
    pub fn update_intent_pattern(&mut self, pattern: IntentPattern) {
        self.intent_patterns.insert(pattern.intent_name.clone(), pattern);
    }
    
    /// Remove intent pattern
    pub fn remove_intent_pattern(&mut self, intent_name: &str) -> Option<IntentPattern> {
        self.intent_patterns.remove(intent_name)
    }
    
    /// Get all available intents
    pub fn get_all_intents(&self) -> Vec<&str> {
        self.intent_patterns.keys().map(|s| s.as_str()).collect()
    }
    
    /// Clear action history for a character
    pub fn clear_action_history(&mut self, character_id: &str) {
        self.action_history.remove(character_id);
    }
    
    /// Get action statistics for a character
    pub fn get_action_statistics(&self, character_id: &str) -> HashMap<String, usize> {
        let mut stats = HashMap::new();
        
        if let Some(history) = self.action_history.get(character_id) {
            for (intent, _) in history {
                *stats.entry(intent.clone()).or_insert(0) += 1;
            }
        }
        
        stats
    }
}