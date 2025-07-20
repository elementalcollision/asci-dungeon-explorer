use std::collections::HashMap;

use super::response_processor::{ResponseProcessor, FilterConfig, ValidationSeverity};
use super::emotion_analyzer::EmotionAnalyzer;
use super::intent_recognizer::{IntentRecognizer, IntentPattern, IntentAction, ActionType, ActionCondition};

/// Example of how to use the response processing components
pub fn response_processing_example() {
    println!("Initializing response processing components...");
    
    // Create response processor
    let mut response_processor = ResponseProcessor::new();
    
    // Configure filter settings
    let mut filter_config = FilterConfig::default();
    filter_config.length_limits = (10, 200); // Shorter responses for this example
    filter_config.profanity_filter = true;
    
    // Add some banned words for demonstration
    filter_config.banned_words.insert("stupid".to_string());
    filter_config.banned_words.insert("idiot".to_string());
    
    response_processor.set_filter_config(filter_config);
    
    // Set character traits for consistency checking
    response_processor.set_character_traits("wizard", vec![
        "wise".to_string(),
        "polite".to_string(),
        "mysterious".to_string(),
    ]);
    
    // Create emotion analyzer
    let mut emotion_analyzer = EmotionAnalyzer::new();
    
    // Create emotion profile for wizard
    let wizard_traits = vec!["wise".to_string(), "calm".to_string(), "patient".to_string()];
    emotion_analyzer.create_emotion_profile("wizard", "calm", &wizard_traits);
    
    // Create intent recognizer
    let mut intent_recognizer = IntentRecognizer::new();
    
    // Add custom intent pattern for the wizard
    let custom_pattern = IntentPattern {
        intent_name: "magic_inquiry".to_string(),
        keywords: vec!["magic".to_string(), "spell".to_string(), "enchantment".to_string()],
        phrases: vec!["teach me magic".to_string(), "how does magic work".to_string()],
        context_clues: vec!["arcane".to_string(), "mystical".to_string()],
        confidence_threshold: 0.7,
        actions: vec![
            IntentAction {
                action_type: ActionType::ProvideInformation("magic_explanation".to_string()),
                parameters: HashMap::new(),
                priority: 90,
                conditions: vec![ActionCondition::MinRelationship(30)],
            },
            IntentAction {
                action_type: ActionType::ShowEmotion("pleased".to_string()),
                parameters: HashMap::new(),
                priority: 60,
                conditions: vec![],
            },
        ],
    };
    
    intent_recognizer.add_intent_pattern(custom_pattern);
    
    // Set available intents for the wizard
    intent_recognizer.set_character_intents("wizard", vec![
        "greeting".to_string(),
        "information_request".to_string(),
        "help_request".to_string(),
        "magic_inquiry".to_string(),
        "farewell".to_string(),
    ]);
    
    // Test responses
    let test_responses = vec![
        ("Hello there, young adventurer! How may I assist you on your journey?", "greeting"),
        ("The ancient arts of magic are complex and require years of study to master.", "magic_inquiry"),
        ("I'm afraid I cannot help you with that, you stupid fool!", "inappropriate"),
        ("Magic magic magic magic magic magic magic magic magic", "repetitive"),
        ("Yes no maybe perhaps could be might not sure unclear", "incoherent"),
        ("Greetings! I sense you seek knowledge of the arcane arts. Very well, I shall teach you.", "magic_inquiry"),
        ("Farewell, and may your path be illuminated by wisdom.", "farewell"),
    ];
    
    println!("\nProcessing test responses...\n");
    
    for (response_text, expected_context) in test_responses {
        println!("=== Processing Response ===");
        println!("Original: {}", response_text);
        println!("Expected Context: {}", expected_context);
        
        // Process the response
        let processed = response_processor.process_response(response_text, "wizard");
        
        println!("Filtered: {}", processed.filtered_text);
        println!("Valid: {}", processed.is_valid());
        println!("Score: {:.2}", processed.get_score());
        
        // Show emotions
        if !processed.emotions.is_empty() {
            println!("Detected Emotions:");
            for emotion in &processed.emotions {
                println!("  - {} (intensity: {}, confidence: {:.2})", 
                    emotion.emotion, emotion.intensity, emotion.confidence);
                if !emotion.indicators.is_empty() {
                    println!("    Indicators: {}", emotion.indicators.join(", "));
                }
            }
        }
        
        // Show intents
        if !processed.intents.is_empty() {
            println!("Detected Intents:");
            for intent in &processed.intents {
                println!("  - {} (confidence: {:.2})", intent.intent, intent.confidence);
                if !intent.indicators.is_empty() {
                    println!("    Indicators: {}", intent.indicators.join(", "));
                }
                if !intent.parameters.is_empty() {
                    println!("    Parameters: {:?}", intent.parameters);
                }
            }
        }
        
        // Show validation issues
        if !processed.validation.issues.is_empty() {
            println!("Validation Issues:");
            for issue in &processed.validation.issues {
                let severity = match issue.severity {
                    ValidationSeverity::Low => "LOW",
                    ValidationSeverity::Medium => "MEDIUM",
                    ValidationSeverity::High => "HIGH",
                    ValidationSeverity::Critical => "CRITICAL",
                };
                println!("  - [{}] {}", severity, issue.description);
            }
        }
        
        // Show suggestions
        if !processed.validation.suggestions.is_empty() {
            println!("Suggestions:");
            for suggestion in &processed.validation.suggestions {
                println!("  - {}", suggestion);
            }
        }
        
        // Analyze emotions with emotion analyzer
        if let Some(emotion_state) = emotion_analyzer.analyze_emotions("wizard", &processed.emotions, expected_context) {
            println!("Emotion Analysis:");
            println!("  Primary: {} (intensity: {})", emotion_state.primary_emotion, emotion_state.intensity);
            
            if !emotion_state.secondary_emotions.is_empty() {
                println!("  Secondary: {:?}", emotion_state.secondary_emotions);
            }
            
            let emotion_desc = emotion_analyzer.generate_emotion_description(&emotion_state);
            println!("  Description: {}", emotion_desc);
        }
        
        // Recognize intents with intent recognizer
        let intent_results = intent_recognizer.recognize_intents(
            "wizard",
            &processed.intents,
            expected_context,
            50 // Assume friendly relationship
        );
        
        if !intent_results.is_empty() {
            println!("Intent Recognition:");
            for result in &intent_results {
                println!("  Intent: {} (confidence: {:.2}, relevance: {:.2})", 
                    result.intent.intent, result.confidence_score, result.context_relevance);
                
                if !result.suggested_actions.is_empty() {
                    println!("  Suggested Actions:");
                    for action in &result.suggested_actions {
                        let description = intent_recognizer.generate_action_description(action);
                        println!("    - [Priority: {}] {}", action.priority, description);
                    }
                }
            }
        }
        
        println!();
    }
    
    // Demonstrate emotion state tracking
    println!("=== Emotion State Tracking ===");
    
    let emotion_sequence = vec![
        ("Hello! I'm so excited to learn magic!", "excited"),
        ("Oh no, this spell is too difficult for me.", "frustrated"),
        ("Thank you for your patience, master.", "grateful"),
        ("I think I'm starting to understand now.", "hopeful"),
    ];
    
    for (text, context) in emotion_sequence {
        println!("Input: {}", text);
        
        let processed = response_processor.process_response(text, "wizard");
        
        if let Some(emotion_state) = emotion_analyzer.analyze_emotions("wizard", &processed.emotions, context) {
            let emotion_desc = emotion_analyzer.generate_emotion_description(&emotion_state);
            println!("Wizard's emotional state: {}", emotion_desc);
            println!("Duration: {} seconds", emotion_state.duration);
        }
        
        println!();
    }
    
    // Demonstrate intent action history
    println!("=== Intent Action History ===");
    
    let recent_actions = intent_recognizer.get_recent_actions("wizard", 5);
    if !recent_actions.is_empty() {
        println!("Recent actions for wizard:");
        for (action, timestamp) in recent_actions {
            println!("  - {} (timestamp: {})", action, timestamp);
        }
    }
    
    let action_stats = intent_recognizer.get_action_statistics("wizard");
    if !action_stats.is_empty() {
        println!("Action statistics for wizard:");
        for (action, count) in action_stats {
            println!("  - {}: {} times", action, count);
        }
    }
    
    println!("\nResponse processing example completed.");
}