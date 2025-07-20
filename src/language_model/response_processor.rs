use std::collections::{HashMap, HashSet};
use serde::{Serialize, Deserialize};
use regex::Regex;
use log::{info, warn, error};

use super::dialogue_system_trait::DialogueEntry;

/// Emotion detected in a response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedEmotion {
    pub emotion: String,
    pub intensity: u8, // 0-100
    pub confidence: f32, // 0.0-1.0
    pub indicators: Vec<String>, // Words/phrases that indicated this emotion
}

/// Intent detected in a response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedIntent {
    pub intent: String,
    pub confidence: f32, // 0.0-1.0
    pub parameters: HashMap<String, String>,
    pub indicators: Vec<String>, // Words/phrases that indicated this intent
}

/// Response validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub issues: Vec<ValidationIssue>,
    pub score: f32, // 0.0-1.0, higher is better
    pub suggestions: Vec<String>,
}

/// Validation issue found in a response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationIssue {
    pub issue_type: ValidationIssueType,
    pub description: String,
    pub severity: ValidationSeverity,
    pub position: Option<(usize, usize)>, // Start and end character positions
}

/// Type of validation issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationIssueType {
    OutOfCharacter,
    Inappropriate,
    Repetitive,
    TooLong,
    TooShort,
    Incoherent,
    FactualError,
    LanguageError,
    Custom(String),
}

/// Severity of validation issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Response filter configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterConfig {
    pub profanity_filter: bool,
    pub length_limits: (usize, usize), // (min, max) characters
    pub banned_words: HashSet<String>,
    pub required_words: HashSet<String>,
    pub character_consistency: bool,
    pub factual_consistency: bool,
}

impl Default for FilterConfig {
    fn default() -> Self {
        FilterConfig {
            profanity_filter: true,
            length_limits: (5, 500),
            banned_words: HashSet::new(),
            required_words: HashSet::new(),
            character_consistency: true,
            factual_consistency: true,
        }
    }
}

/// Response processor for filtering and analyzing dialogue responses
pub struct ResponseProcessor {
    filter_config: FilterConfig,
    emotion_patterns: HashMap<String, Vec<Regex>>,
    intent_patterns: HashMap<String, Vec<Regex>>,
    profanity_list: HashSet<String>,
    character_traits: HashMap<String, Vec<String>>,
}

impl ResponseProcessor {
    /// Create a new response processor
    pub fn new() -> Self {
        let mut processor = ResponseProcessor {
            filter_config: FilterConfig::default(),
            emotion_patterns: HashMap::new(),
            intent_patterns: HashMap::new(),
            profanity_list: HashSet::new(),
            character_traits: HashMap::new(),
        };
        
        processor.initialize_patterns();
        processor.initialize_profanity_list();
        
        processor
    }
    
    /// Initialize emotion detection patterns
    fn initialize_patterns(&mut self) {
        // Emotion patterns
        self.add_emotion_pattern("happy", vec![
            r"\b(happy|joy|glad|pleased|delighted|cheerful|elated)\b",
            r"\b(smile|grin|laugh|chuckle)\b",
            r"[!]{2,}",
            r"\b(wonderful|fantastic|great|amazing)\b",
        ]);
        
        self.add_emotion_pattern("sad", vec![
            r"\b(sad|sorrow|grief|melancholy|depressed|gloomy)\b",
            r"\b(cry|weep|tear|sob)\b",
            r"\b(unfortunate|tragic|terrible|awful)\b",
        ]);
        
        self.add_emotion_pattern("angry", vec![
            r"\b(angry|furious|rage|mad|irritated|annoyed)\b",
            r"\b(hate|despise|loathe|detest)\b",
            r"\b(damn|curse|blast)\b",
            r"[!]{3,}",
        ]);
        
        self.add_emotion_pattern("confused", vec![
            r"\b(confused|puzzled|bewildered|perplexed|baffled)\b",
            r"\b(what|why|how|huh)\?",
            r"\b(understand|comprehend|grasp)\b.*\b(not|don't|can't)\b",
        ]);
        
        self.add_emotion_pattern("surprised", vec![
            r"\b(surprised|shocked|amazed|astonished|stunned)\b",
            r"\b(oh|wow|whoa|incredible|unbelievable)\b",
            r"\b(never|didn't expect|wouldn't have thought)\b",
        ]);
        
        self.add_emotion_pattern("curious", vec![
            r"\b(curious|interested|intrigued|wonder)\b",
            r"\b(tell me|explain|describe|what about)\b",
            r"\?{2,}",
        ]);
        
        self.add_emotion_pattern("concerned", vec![
            r"\b(concerned|worried|anxious|troubled|uneasy)\b",
            r"\b(careful|cautious|beware|danger)\b",
            r"\b(hope|pray|wish)\b.*\b(safe|well|okay)\b",
        ]);
        
        // Intent patterns
        self.add_intent_pattern("greeting", vec![
            r"\b(hello|hi|greetings|salutations|good day)\b",
            r"\b(welcome|pleased to meet)\b",
        ]);
        
        self.add_intent_pattern("farewell", vec![
            r"\b(goodbye|farewell|bye|see you|until next time)\b",
            r"\b(safe travels|may.*bless|good luck)\b",
        ]);
        
        self.add_intent_pattern("question", vec![
            r"\?",
            r"\b(what|where|when|why|how|who)\b",
            r"\b(do you|can you|would you|could you)\b",
        ]);
        
        self.add_intent_pattern("offer_help", vec![
            r"\b(help|assist|aid|support)\b",
            r"\b(can I|may I|would you like)\b",
            r"\b(service|guidance|advice)\b",
        ]);
        
        self.add_intent_pattern("request", vec![
            r"\b(please|could you|would you|can you)\b",
            r"\b(need|want|require|seek)\b",
            r"\b(give me|show me|tell me)\b",
        ]);
        
        self.add_intent_pattern("information", vec![
            r"\b(know|heard|understand|aware)\b",
            r"\b(tell you|inform you|let you know)\b",
            r"\b(fact|truth|information|knowledge)\b",
        ]);
        
        self.add_intent_pattern("warning", vec![
            r"\b(beware|careful|caution|danger|warning)\b",
            r"\b(avoid|stay away|don't go)\b",
            r"\b(risky|hazardous|perilous|unsafe)\b",
        ]);
    }
    
    /// Initialize profanity list
    fn initialize_profanity_list(&mut self) {
        // Basic profanity list - in a real implementation, this would be more comprehensive
        let profanity_words = vec![
            "damn", "hell", "bastard", "bitch", "shit", "fuck", "ass", "crap",
            // Add more as needed, but keep it appropriate for the fantasy setting
        ];
        
        for word in profanity_words {
            self.profanity_list.insert(word.to_string());
        }
    }
    
    /// Add emotion detection pattern
    fn add_emotion_pattern(&mut self, emotion: &str, patterns: Vec<&str>) {
        let compiled_patterns: Vec<Regex> = patterns.iter()
            .filter_map(|pattern| {
                match Regex::new(&format!("(?i){}", pattern)) {
                    Ok(regex) => Some(regex),
                    Err(e) => {
                        warn!("Failed to compile emotion pattern '{}': {}", pattern, e);
                        None
                    }
                }
            })
            .collect();
        
        self.emotion_patterns.insert(emotion.to_string(), compiled_patterns);
    }
    
    /// Add intent detection pattern
    fn add_intent_pattern(&mut self, intent: &str, patterns: Vec<&str>) {
        let compiled_patterns: Vec<Regex> = patterns.iter()
            .filter_map(|pattern| {
                match Regex::new(&format!("(?i){}", pattern)) {
                    Ok(regex) => Some(regex),
                    Err(e) => {
                        warn!("Failed to compile intent pattern '{}': {}", pattern, e);
                        None
                    }
                }
            })
            .collect();
        
        self.intent_patterns.insert(intent.to_string(), compiled_patterns);
    }
    
    /// Process a dialogue response
    pub fn process_response(&self, response: &str, character_id: &str) -> ProcessedResponse {
        let filtered_response = self.filter_response(response);
        let emotions = self.extract_emotions(&filtered_response);
        let intents = self.recognize_intents(&filtered_response);
        let validation = self.validate_response(&filtered_response, character_id);
        
        ProcessedResponse {
            original_text: response.to_string(),
            filtered_text: filtered_response,
            emotions,
            intents,
            validation,
        }
    }
    
    /// Filter a response based on configuration
    pub fn filter_response(&self, response: &str) -> String {
        let mut filtered = response.to_string();
        
        // Apply profanity filter
        if self.filter_config.profanity_filter {
            filtered = self.apply_profanity_filter(&filtered);
        }
        
        // Apply length limits
        let (min_len, max_len) = self.filter_config.length_limits;
        if filtered.len() > max_len {
            filtered = self.truncate_response(&filtered, max_len);
        }
        
        // Remove banned words
        for banned_word in &self.filter_config.banned_words {
            let pattern = format!(r"\b{}\b", regex::escape(banned_word));
            if let Ok(regex) = Regex::new(&format!("(?i){}", pattern)) {
                filtered = regex.replace_all(&filtered, "[FILTERED]").to_string();
            }
        }
        
        // Clean up extra whitespace
        filtered = self.clean_whitespace(&filtered);
        
        filtered
    }
    
    /// Apply profanity filter
    fn apply_profanity_filter(&self, text: &str) -> String {
        let mut filtered = text.to_string();
        
        for profanity in &self.profanity_list {
            let pattern = format!(r"\b{}\b", regex::escape(profanity));
            if let Ok(regex) = Regex::new(&format!("(?i){}", pattern)) {
                // Replace with asterisks, keeping first letter
                let replacement = if profanity.len() > 1 {
                    format!("{}{}",
                        profanity.chars().next().unwrap(),
                        "*".repeat(profanity.len() - 1)
                    )
                } else {
                    "*".to_string()
                };
                
                filtered = regex.replace_all(&filtered, replacement).to_string();
            }
        }
        
        filtered
    }
    
    /// Truncate response to maximum length
    fn truncate_response(&self, text: &str, max_len: usize) -> String {
        if text.len() <= max_len {
            return text.to_string();
        }
        
        // Try to truncate at sentence boundary
        let truncated = &text[..max_len];
        
        // Find the last sentence ending
        if let Some(pos) = truncated.rfind('.') {
            if pos > max_len / 2 { // Only if we're not cutting too much
                return truncated[..=pos].to_string();
            }
        }
        
        // Find the last word boundary
        if let Some(pos) = truncated.rfind(' ') {
            if pos > max_len / 2 { // Only if we're not cutting too much
                return format!("{}...", &truncated[..pos]);
            }
        }
        
        // Hard truncate
        format!("{}...", &text[..max_len.saturating_sub(3)])
    }
    
    /// Clean up extra whitespace
    fn clean_whitespace(&self, text: &str) -> String {
        // Replace multiple spaces with single space
        let space_regex = Regex::new(r"\s+").unwrap();
        let cleaned = space_regex.replace_all(text, " ");
        
        // Trim leading and trailing whitespace
        cleaned.trim().to_string()
    }
    
    /// Extract emotions from text
    pub fn extract_emotions(&self, text: &str) -> Vec<DetectedEmotion> {
        let mut emotions = Vec::new();
        
        for (emotion_name, patterns) in &self.emotion_patterns {
            let mut indicators = Vec::new();
            let mut match_count = 0;
            
            for pattern in patterns {
                for mat in pattern.find_iter(text) {
                    indicators.push(mat.as_str().to_string());
                    match_count += 1;
                }
            }
            
            if !indicators.is_empty() {
                // Calculate intensity based on number of matches and pattern strength
                let intensity = ((match_count as f32 * 20.0).min(100.0)) as u8;
                
                // Calculate confidence based on pattern specificity
                let confidence = (match_count as f32 / patterns.len() as f32).min(1.0);
                
                emotions.push(DetectedEmotion {
                    emotion: emotion_name.clone(),
                    intensity,
                    confidence,
                    indicators,
                });
            }
        }
        
        // Sort by confidence (highest first)
        emotions.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap_or(std::cmp::Ordering::Equal));
        
        emotions
    }
    
    /// Recognize intents in text
    pub fn recognize_intents(&self, text: &str) -> Vec<DetectedIntent> {
        let mut intents = Vec::new();
        
        for (intent_name, patterns) in &self.intent_patterns {
            let mut indicators = Vec::new();
            let mut match_count = 0;
            
            for pattern in patterns {
                for mat in pattern.find_iter(text) {
                    indicators.push(mat.as_str().to_string());
                    match_count += 1;
                }
            }
            
            if !indicators.is_empty() {
                // Calculate confidence based on pattern matches
                let confidence = (match_count as f32 / patterns.len() as f32).min(1.0);
                
                // Extract parameters (simplified - in a real implementation, this would be more sophisticated)
                let parameters = self.extract_intent_parameters(intent_name, text);
                
                intents.push(DetectedIntent {
                    intent: intent_name.clone(),
                    confidence,
                    parameters,
                    indicators,
                });
            }
        }
        
        // Sort by confidence (highest first)
        intents.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap_or(std::cmp::Ordering::Equal));
        
        intents
    }
    
    /// Extract parameters for an intent (simplified implementation)
    fn extract_intent_parameters(&self, intent: &str, text: &str) -> HashMap<String, String> {
        let mut parameters = HashMap::new();
        
        match intent {
            "question" => {
                // Extract question words
                let question_regex = Regex::new(r"\b(what|where|when|why|how|who)\b").unwrap();
                if let Some(mat) = question_regex.find(text) {
                    parameters.insert("question_type".to_string(), mat.as_str().to_string());
                }
            },
            "request" => {
                // Extract what is being requested (simplified)
                let request_regex = Regex::new(r"\b(give me|show me|tell me)\s+(.+?)(?:\.|$)").unwrap();
                if let Some(caps) = request_regex.captures(text) {
                    if let Some(object) = caps.get(2) {
                        parameters.insert("object".to_string(), object.as_str().trim().to_string());
                    }
                }
            },
            _ => {}
        }
        
        parameters
    }
    
    /// Validate a response
    pub fn validate_response(&self, text: &str, character_id: &str) -> ValidationResult {
        let mut issues = Vec::new();
        let mut score = 1.0;
        let mut suggestions = Vec::new();
        
        // Check length
        let (min_len, max_len) = self.filter_config.length_limits;
        if text.len() < min_len {
            issues.push(ValidationIssue {
                issue_type: ValidationIssueType::TooShort,
                description: format!("Response is too short ({} characters, minimum {})", text.len(), min_len),
                severity: ValidationSeverity::Medium,
                position: None,
            });
            score -= 0.2;
            suggestions.push("Consider expanding the response with more detail.".to_string());
        }
        
        if text.len() > max_len {
            issues.push(ValidationIssue {
                issue_type: ValidationIssueType::TooLong,
                description: format!("Response is too long ({} characters, maximum {})", text.len(), max_len),
                severity: ValidationSeverity::Medium,
                position: None,
            });
            score -= 0.2;
            suggestions.push("Consider shortening the response.".to_string());
        }
        
        // Check for repetition
        if self.has_repetitive_content(text) {
            issues.push(ValidationIssue {
                issue_type: ValidationIssueType::Repetitive,
                description: "Response contains repetitive content".to_string(),
                severity: ValidationSeverity::Low,
                position: None,
            });
            score -= 0.1;
            suggestions.push("Avoid repeating the same words or phrases.".to_string());
        }
        
        // Check for coherence
        if !self.is_coherent(text) {
            issues.push(ValidationIssue {
                issue_type: ValidationIssueType::Incoherent,
                description: "Response appears incoherent or fragmented".to_string(),
                severity: ValidationSeverity::High,
                position: None,
            });
            score -= 0.4;
            suggestions.push("Ensure the response flows logically and makes sense.".to_string());
        }
        
        // Check character consistency
        if self.filter_config.character_consistency {
            if let Some(character_issues) = self.check_character_consistency(text, character_id) {
                issues.extend(character_issues);
                score -= 0.3;
                suggestions.push("Ensure the response matches the character's personality and traits.".to_string());
            }
        }
        
        // Check for inappropriate content
        if self.has_inappropriate_content(text) {
            issues.push(ValidationIssue {
                issue_type: ValidationIssueType::Inappropriate,
                description: "Response contains inappropriate content".to_string(),
                severity: ValidationSeverity::Critical,
                position: None,
            });
            score -= 0.5;
            suggestions.push("Remove inappropriate content and keep responses suitable for the game setting.".to_string());
        }
        
        // Ensure score is within bounds
        score = score.max(0.0).min(1.0);
        
        ValidationResult {
            is_valid: score >= 0.5 && !issues.iter().any(|i| matches!(i.severity, ValidationSeverity::Critical)),
            issues,
            score,
            suggestions,
        }
    }
    
    /// Check for repetitive content
    fn has_repetitive_content(&self, text: &str) -> bool {
        let words: Vec<&str> = text.split_whitespace().collect();
        
        if words.len() < 4 {
            return false;
        }
        
        // Check for repeated phrases (3+ words)
        for i in 0..words.len().saturating_sub(5) {
            let phrase = &words[i..i+3];
            
            for j in (i+3)..words.len().saturating_sub(2) {
                if j + phrase.len() <= words.len() {
                    let other_phrase = &words[j..j+phrase.len()];
                    if phrase == other_phrase {
                        return true;
                    }
                }
            }
        }
        
        // Check for repeated words (more than 3 times)
        let mut word_counts = HashMap::new();
        for word in &words {
            let word_lower = word.to_lowercase();
            // Skip common words
            if !["the", "a", "an", "and", "or", "but", "in", "on", "at", "to", "for", "of", "with", "by"].contains(&word_lower.as_str()) {
                *word_counts.entry(word_lower).or_insert(0) += 1;
            }
        }
        
        word_counts.values().any(|&count| count > 3)
    }
    
    /// Check if text is coherent
    fn is_coherent(&self, text: &str) -> bool {
        // Simple coherence checks
        
        // Check for complete sentences
        let sentences: Vec<&str> = text.split('.').collect();
        if sentences.len() > 1 {
            for sentence in &sentences[..sentences.len()-1] { // Exclude last empty part after final period
                let sentence = sentence.trim();
                if !sentence.is_empty() && sentence.split_whitespace().count() < 2 {
                    return false; // Very short "sentences" might indicate fragmentation
                }
            }
        }
        
        // Check for balanced punctuation
        let open_parens = text.matches('(').count();
        let close_parens = text.matches(')').count();
        if open_parens != close_parens {
            return false;
        }
        
        let open_quotes = text.matches('"').count();
        if open_quotes % 2 != 0 {
            return false;
        }
        
        // Check for reasonable word distribution
        let words: Vec<&str> = text.split_whitespace().collect();
        if words.len() > 5 {
            // Check if there are too many very short or very long words
            let short_words = words.iter().filter(|w| w.len() <= 2).count();
            let long_words = words.iter().filter(|w| w.len() >= 15).count();
            
            if short_words as f32 / words.len() as f32 > 0.7 {
                return false; // Too many short words
            }
            
            if long_words as f32 / words.len() as f32 > 0.3 {
                return false; // Too many long words
            }
        }
        
        true
    }
    
    /// Check character consistency
    fn check_character_consistency(&self, text: &str, character_id: &str) -> Option<Vec<ValidationIssue>> {
        if let Some(traits) = self.character_traits.get(character_id) {
            let mut issues = Vec::new();
            
            // This is a simplified implementation
            // In a real system, you'd have more sophisticated character consistency checks
            
            // Check if response contradicts known character traits
            let text_lower = text.to_lowercase();
            
            for trait_name in traits {
                match trait_name.as_str() {
                    "polite" => {
                        if text_lower.contains("shut up") || text_lower.contains("go away") {
                            issues.push(ValidationIssue {
                                issue_type: ValidationIssueType::OutOfCharacter,
                                description: "Response is rude, but character is supposed to be polite".to_string(),
                                severity: ValidationSeverity::Medium,
                                position: None,
                            });
                        }
                    },
                    "wise" => {
                        if text_lower.contains("i don't know") && text_lower.matches("i don't know").count() > 1 {
                            issues.push(ValidationIssue {
                                issue_type: ValidationIssueType::OutOfCharacter,
                                description: "Character claims ignorance too often for someone who is supposed to be wise".to_string(),
                                severity: ValidationSeverity::Low,
                                position: None,
                            });
                        }
                    },
                    "mysterious" => {
                        if text.len() > 200 && !text_lower.contains("perhaps") && !text_lower.contains("maybe") {
                            issues.push(ValidationIssue {
                                issue_type: ValidationIssueType::OutOfCharacter,
                                description: "Response is too direct for a mysterious character".to_string(),
                                severity: ValidationSeverity::Low,
                                position: None,
                            });
                        }
                    },
                    _ => {}
                }
            }
            
            if !issues.is_empty() {
                return Some(issues);
            }
        }
        
        None
    }
    
    /// Check for inappropriate content
    fn has_inappropriate_content(&self, text: &str) -> bool {
        let text_lower = text.to_lowercase();
        
        // Check for explicit content (simplified)
        let inappropriate_patterns = vec![
            r"\b(kill yourself|die|murder)\b",
            r"\b(sexual|erotic|porn)\b",
            r"\b(drugs|cocaine|heroin)\b",
        ];
        
        for pattern in inappropriate_patterns {
            if let Ok(regex) = Regex::new(pattern) {
                if regex.is_match(&text_lower) {
                    return true;
                }
            }
        }
        
        false
    }
    
    /// Set character traits for consistency checking
    pub fn set_character_traits(&mut self, character_id: &str, traits: Vec<String>) {
        self.character_traits.insert(character_id.to_string(), traits);
    }
    
    /// Set filter configuration
    pub fn set_filter_config(&mut self, config: FilterConfig) {
        self.filter_config = config;
    }
    
    /// Get filter configuration
    pub fn get_filter_config(&self) -> &FilterConfig {
        &self.filter_config
    }
    
    /// Add custom emotion pattern
    pub fn add_custom_emotion_pattern(&mut self, emotion: &str, pattern: &str) {
        if let Ok(regex) = Regex::new(&format!("(?i){}", pattern)) {
            self.emotion_patterns
                .entry(emotion.to_string())
                .or_insert_with(Vec::new)
                .push(regex);
        }
    }
    
    /// Add custom intent pattern
    pub fn add_custom_intent_pattern(&mut self, intent: &str, pattern: &str) {
        if let Ok(regex) = Regex::new(&format!("(?i){}", pattern)) {
            self.intent_patterns
                .entry(intent.to_string())
                .or_insert_with(Vec::new)
                .push(regex);
        }
    }
}

/// Processed response containing all analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessedResponse {
    pub original_text: String,
    pub filtered_text: String,
    pub emotions: Vec<DetectedEmotion>,
    pub intents: Vec<DetectedIntent>,
    pub validation: ValidationResult,
}

impl ProcessedResponse {
    /// Get the primary emotion (highest confidence)
    pub fn get_primary_emotion(&self) -> Option<&DetectedEmotion> {
        self.emotions.first()
    }
    
    /// Get the primary intent (highest confidence)
    pub fn get_primary_intent(&self) -> Option<&DetectedIntent> {
        self.intents.first()
    }
    
    /// Check if the response is valid
    pub fn is_valid(&self) -> bool {
        self.validation.is_valid
    }
    
    /// Get validation score
    pub fn get_score(&self) -> f32 {
        self.validation.score
    }
    
    /// Get critical issues
    pub fn get_critical_issues(&self) -> Vec<&ValidationIssue> {
        self.validation.issues.iter()
            .filter(|issue| matches!(issue.severity, ValidationSeverity::Critical))
            .collect()
    }
    
    /// Get suggestions for improvement
    pub fn get_suggestions(&self) -> &Vec<String> {
        &self.validation.suggestions
    }
}