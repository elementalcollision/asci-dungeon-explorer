use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use super::model_manager::ModelManager;
use super::dialogue_system_trait::{DialogueSystem, DialogueConfig};
use super::llama_dialogue_system::LlamaDialogueSystem;
use super::conversation_manager::ConversationManager;
use super::character_persona::{PersonaManager, ExtendedPersona};
use super::dialogue_history::DialogueHistoryManager;

/// Example of how to use the dialogue system components
pub fn dialogue_system_example() {
    println!("Initializing dialogue system components...");
    
    // Initialize the model manager
    let mut model_manager = ModelManager::new();
    model_manager.initialize().expect("Failed to initialize model manager");
    
    // Add a model path (adjust the path to your model)
    let model_path = PathBuf::from("models/tinyllama-1.1b.gguf");
    model_manager.add_model_path("tiny", model_path);
    
    // Try to load the model
    match model_manager.load_model("tiny", None) {
        Ok(_) => println!("Model loaded successfully"),
        Err(e) => {
            println!("Failed to load model: {}", e);
            println!("Using fallback responses instead");
        }
    }
    
    // Set as default model if loaded
    if model_manager.is_model_loaded("tiny") {
        model_manager.set_default_model("tiny").expect("Failed to set default model");
    }
    
    // Create shared model manager
    let model_manager = Arc::new(Mutex::new(model_manager));
    
    // Create dialogue system
    let mut dialogue_system = LlamaDialogueSystem::new(model_manager.clone());
    dialogue_system.initialize().expect("Failed to initialize dialogue system");
    
    // Configure the dialogue system
    let config = DialogueConfig {
        max_history_length: 10,
        system_prompt: "You are an NPC in a fantasy roguelike game. Respond in character based on your personality, knowledge, and relationship with the player. Keep responses concise (1-3 sentences) and appropriate to the fantasy setting.".to_string(),
        temperature: 0.7,
        max_tokens: 100,
        stop_sequences: vec!["\n".to_string(), "Player:".to_string()],
        model_name: Some("tiny".to_string()),
        timeout_seconds: 10,
    };
    dialogue_system.set_config(config);
    
    // Create persona manager
    let mut persona_manager = PersonaManager::new(PathBuf::from("data/personas"));
    persona_manager.initialize().expect("Failed to initialize persona manager");
    
    // Create or load a persona
    let persona = match persona_manager.load_persona("wizard") {
        Ok(persona) => {
            println!("Loaded existing persona: {}", persona.base.name);
            persona
        },
        Err(_) => {
            println!("Creating new persona...");
            let mut persona = persona_manager.create_persona(
                "Eldrin the Wise", 
                "An ancient wizard with flowing white beard and sparkling blue eyes"
            ).expect("Failed to create persona");
            
            // Add details to the persona
            persona.base.background = "Eldrin has lived for centuries, studying the arcane arts and collecting knowledge from across the realm.".to_string();
            persona.base.traits = vec!["Wise".to_string(), "Patient".to_string(), "Mysterious".to_string()];
            persona.base.speech_style = "Formal and eloquent, often using archaic terms and speaking in riddles.".to_string();
            
            // Add knowledge
            persona.add_knowledge(
                "dragons", 
                "Dragons have been returning to the northern mountains after centuries of absence.", 
                90, 
                "observation"
            );
            
            persona.add_knowledge(
                "ancient artifact", 
                "The Orb of Zephyr is hidden somewhere in the Forgotten Depths.", 
                70, 
                "ancient texts"
            );
            
            // Add relationships
            persona.add_relationship("player", 20, "The player helped Eldrin recover a stolen spellbook");
            
            // Add goals and fears
            persona.add_goal("Discover the secret of eternal youth");
            persona.add_fear("The return of the Dark Sorcerer Malthus");
            
            // Save the persona
            persona_manager.save_persona(&persona).expect("Failed to save persona");
            
            persona
        }
    };
    
    // Create conversation manager
    let mut conversation_manager = ConversationManager::new(model_manager.clone());
    conversation_manager.initialize().expect("Failed to initialize conversation manager");
    
    // Register the dialogue system
    conversation_manager.register_dialogue_system("default", dialogue_system);
    
    // Create dialogue history manager
    let mut history_manager = DialogueHistoryManager::new(PathBuf::from("data/dialogue_history"));
    history_manager.initialize().expect("Failed to initialize history manager");
    
    // Start a conversation
    println!("Starting conversation with {}...", persona.base.name);
    let context = conversation_manager.start_conversation("wizard", "Tower of Magic")
        .expect("Failed to start conversation");
    
    // Simulate a conversation
    let player_inputs = [
        "Hello, wise one. I seek your guidance.",
        "Tell me about the dragons in the north.",
        "Do you know anything about ancient artifacts?",
        "Thank you for your wisdom.",
    ];
    
    for input in player_inputs.iter() {
        println!("\nPlayer: {}", input);
        
        // Generate response
        let response = match conversation_manager.generate_response("wizard", input) {
            Ok(response) => response,
            Err(e) => {
                println!("Error generating response: {}", e);
                continue;
            }
        };
        
        // Print response
        if let Some(emotion) = &response.emotion {
            println!("{} [{}]: {}", persona.base.name, emotion, response.text);
        } else {
            println!("{}: {}", persona.base.name, response.text);
        }
        
        // Add to history with importance based on content
        let importance = if input.contains("artifact") || input.contains("dragon") {
            90 // Important topics
        } else {
            50 // Regular conversation
        };
        
        history_manager.add_entry(
            "wizard",
            response.clone(),
            Some(&context),
            "Tower of Magic",
            vec!["conversation".to_string()],
            importance
        );
    }
    
    // Generate dialogue options
    println!("\nGenerating dialogue options...");
    match conversation_manager.generate_options("wizard", "ancient artifacts") {
        Ok(options) => {
            println!("Available dialogue options:");
            for (i, option) in options.options.iter().enumerate() {
                println!("{}. {}", i + 1, option.text);
            }
        },
        Err(e) => {
            println!("Error generating options: {}", e);
        }
    }
    
    // End the conversation
    conversation_manager.end_conversation("wizard").expect("Failed to end conversation");
    
    // Save history
    history_manager.save_all_histories().expect("Failed to save dialogue history");
    
    println!("\nDialogue system example completed.");
}