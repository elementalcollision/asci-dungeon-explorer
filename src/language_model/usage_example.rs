use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use super::llama_integration::{LlamaConfig, LlamaRequest};
use super::model_manager::ModelManager;
use super::dialogue_system::{DialogueSystem, DialogueContext, DialogueEntry};

/// Example of how to use the llama.cpp integration
pub fn llama_integration_example() {
    // Initialize the model manager
    let mut model_manager = ModelManager::new();
    model_manager.initialize().expect("Failed to initialize model manager");
    
    // Add a model path (adjust the path to your model)
    let model_path = PathBuf::from("models/tinyllama-1.1b.gguf");
    model_manager.add_model_path("tiny", model_path);
    
    // Create a custom configuration
    let mut config = LlamaConfig::default();
    config.context_size = 1024;
    config.threads = 4;
    config.temperature = 0.8;
    
    // Load the model
    match model_manager.load_model("tiny", Some(config)) {
        Ok(_) => println!("Model loaded successfully"),
        Err(e) => {
            println!("Failed to load model: {}", e);
            println!("Using fallback responses instead");
        }
    }
    
    // Set as default model
    if model_manager.is_model_loaded("tiny") {
        model_manager.set_default_model("tiny").expect("Failed to set default model");
    }
    
    // Create a dialogue system
    let model_manager = Arc::new(Mutex::new(model_manager));
    let mut dialogue_system = DialogueSystem::new(model_manager.clone());
    
    // Configure the dialogue system
    dialogue_system.set_system_prompt(
        "You are a wise old wizard in a fantasy world. Respond in character with short, mystical answers.".to_string()
    );
    dialogue_system.set_max_history_length(5);
    
    // Create a dialogue context
    let mut context = DialogueContext {
        character_name: "Merlin".to_string(),
        character_description: "An ancient wizard with a flowing white beard and sparkling blue eyes".to_string(),
        location: "Tower of Mysteries".to_string(),
        history: Vec::new(),
        relationship: 50, // Friendly
        knowledge: vec![
            "The player is a novice adventurer".to_string(),
            "A dragon has been terrorizing the nearby village".to_string(),
            "The ancient amulet can control elemental forces".to_string(),
        ],
    };
    
    // Simulate a conversation
    let player_inputs = [
        "Hello, wise one. I seek your guidance.",
        "Tell me about the dragon problem.",
        "How can I defeat such a powerful creature?",
        "Where can I find this amulet?",
        "Thank you for your wisdom.",
    ];
    
    for input in player_inputs.iter() {
        println!("Player: {}", input);
        
        // Add player entry to history
        context.history.push(DialogueEntry {
            speaker: "Player".to_string(),
            text: input.to_string(),
            emotion: None,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        });
        
        // Generate response
        let response = match dialogue_system.generate_response(&context, input) {
            Ok(response) => response,
            Err(e) => {
                println!("Error generating response: {}", e);
                dialogue_system.fallback_response(&context)
            }
        };
        
        // Print response
        if let Some(emotion) = &response.emotion {
            println!("Merlin [{}]: {}", emotion, response.text);
        } else {
            println!("Merlin: {}", response.text);
        }
        
        // Add response to history
        context.history.push(response);
        
        println!();
    }
    
    // Clean up
    if let Ok(mut manager) = model_manager.lock() {
        manager.unload_all();
    }
}