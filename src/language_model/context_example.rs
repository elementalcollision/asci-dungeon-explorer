use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

use super::model_manager::ModelManager;
use super::dialogue_system_trait::{DialogueSystem, DialogueConfig, CharacterPersona};
use super::llama_dialogue_system::LlamaDialogueSystem;
use super::context_builder::{ContextBuilder, KnowledgeNode, KnowledgeConnection};
use super::knowledge_system::{KnowledgeSystem, KnowledgeCategory, KnowledgeEntry, CharacterKnowledge};
use super::relationship_context::{RelationshipNetwork, RelationshipType};

/// Example of how to use the context building components
pub fn context_building_example() {
    println!("Initializing context building components...");
    
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
    
    // Create context builder
    let mut context_builder = ContextBuilder::new();
    
    // Create knowledge system
    let mut knowledge_system = KnowledgeSystem::new(PathBuf::from("data/knowledge"));
    knowledge_system.initialize().expect("Failed to initialize knowledge system");
    
    // Create default knowledge base if it doesn't exist
    if knowledge_system.get_all_knowledge_nodes().is_empty() {
        println!("Creating default knowledge base...");
        knowledge_system.create_default_knowledge_base();
    }
    
    // Create relationship network
    let mut relationship_network = RelationshipNetwork::new(PathBuf::from("data/relationships"));
    relationship_network.initialize().expect("Failed to initialize relationship network");
    
    // Create default relationships if they don't exist
    if relationship_network.get_all_relationships("player").is_empty() {
        println!("Creating default relationships...");
        relationship_network.create_default_relationships();
    }
    
    // Create a character persona
    let wizard_persona = CharacterPersona {
        id: "wizard".to_string(),
        name: "Eldrin the Wise".to_string(),
        description: "An ancient wizard with flowing white beard and sparkling blue eyes".to_string(),
        background: "Eldrin has lived for centuries, studying the arcane arts and collecting knowledge from across the realm.".to_string(),
        traits: vec!["Wise".to_string(), "Patient".to_string(), "Mysterious".to_string()],
        speech_style: "Formal and eloquent, often using archaic terms and speaking in riddles.".to_string(),
        knowledge_base: vec![
            "Dragons have been returning to the northern mountains after centuries of absence.".to_string(),
            "The Orb of Zephyr is hidden somewhere in the Forgotten Depths.".to_string(),
            "The ancient mage Zephyrus was my mentor many centuries ago.".to_string(),
        ],
        faction: Some("Order of the Azure Flame".to_string()),
        default_emotion: "calm".to_string(),
        available_emotions: vec![
            "calm".to_string(), 
            "curious".to_string(), 
            "concerned".to_string(), 
            "amused".to_string(),
        ],
        dialogue_templates: HashMap::new(),
        metadata: HashMap::new(),
    };
    
    // Add location descriptions to context builder
    context_builder.add_location_description("Tower of Magic", "A tall stone tower with glowing runes etched into its walls. The air hums with magical energy, and strange artifacts float in glass cases around the circular room.");
    context_builder.add_location_description("Ravenhollow", "A small town nestled in the shadow of the Misty Mountains. The buildings are made of dark wood and stone, and the townsfolk eye strangers with suspicion.");
    context_builder.add_location_description("Forgotten Depths", "An ancient dungeon complex beneath the Misty Mountains. The stone walls are covered in strange symbols, and the air is thick with dust and the scent of old magic.");
    
    // Add character knowledge to context builder
    context_builder.add_character_knowledge(
        "wizard",
        "dragons",
        "Dragons have been returning to the northern mountains after centuries of absence. The red dragon Infernus has been seen circling the peaks.",
        90,
        "observation"
    );
    
    context_builder.add_character_knowledge(
        "wizard",
        "orb",
        "The Orb of Zephyr is hidden somewhere in the Forgotten Depths, guarded by ancient traps and magical constructs.",
        70,
        "ancient texts"
    );
    
    context_builder.add_character_knowledge(
        "wizard",
        "player",
        "The player seems to have a natural affinity for magic, particularly elemental spells.",
        60,
        "observation"
    );
    
    // Add faction relationships to context builder
    context_builder.add_faction_relationship("Order of the Azure Flame", "Ravenhollow Townsfolk", 30);
    context_builder.add_faction_relationship("Order of the Azure Flame", "Crimson Brotherhood", -70);
    
    // Add world state to context builder
    context_builder.add_world_state("dragon_sightings", "Increased in the northern mountains");
    context_builder.add_world_state("magical_disturbances", "Growing stronger near the Forgotten Depths");
    
    // Add recent events to context builder
    context_builder.add_recent_event("A merchant caravan was attacked by bandits on the north road");
    context_builder.add_recent_event("Strange lights were seen in the sky above the Misty Mountains");
    
    // Add knowledge nodes to context builder
    let node1 = KnowledgeNode {
        id: "orb-of-zephyr".to_string(),
        content: "Orb of Zephyr: A powerful artifact that can control the winds and weather.".to_string(),
        node_type: "artifact".to_string(),
        connections: Vec::new(),
        metadata: HashMap::new(),
    };
    
    let node2 = KnowledgeNode {
        id: "zephyrus".to_string(),
        content: "Zephyrus: An ancient mage who created the Orb of Zephyr and disappeared centuries ago.".to_string(),
        node_type: "character".to_string(),
        connections: Vec::new(),
        metadata: HashMap::new(),
    };
    
    context_builder.add_knowledge_node(node1);
    context_builder.add_knowledge_node(node2);
    context_builder.connect_knowledge_nodes("orb-of-zephyr", "zephyrus", "created_by", 90);
    
    // Build dialogue context
    println!("\nBuilding dialogue context...");
    let context = context_builder.build_dialogue_context(&wizard_persona, "Tower of Magic");
    
    // Format system prompt
    let system_prompt = context_builder.format_system_prompt(&wizard_persona);
    println!("\nSystem Prompt:\n{}", system_prompt);
    
    // Format dialogue prompt
    let player_input = "Tell me about the Orb of Zephyr.";
    let prompt = context_builder.format_dialogue_prompt(&context, player_input);
    println!("\nDialogue Prompt:\n{}", prompt);
    
    // Get relevant knowledge for a topic
    println!("\nRelevant knowledge for 'orb':");
    let relevant_knowledge = context_builder.get_relevant_knowledge(&context, "orb");
    for node in relevant_knowledge {
        println!("- {}", node.content);
    }
    
    // Build topic context
    println!("\nBuilding topic context for 'dragons':");
    let topic_context = context_builder.build_topic_context(&context, "dragons");
    for element in topic_context {
        println!("- [Priority: {}] {}", element.priority, element.content);
    }
    
    // Format prompt with specific context elements
    let prompt_with_elements = context_builder.format_prompt_with_elements(&context, &topic_context, "Are the dragons dangerous?");
    println!("\nPrompt with specific context elements:\n{}", prompt_with_elements);
    
    // Generate relationship context
    println!("\nRelationship context for wizard:");
    let relationship_context = relationship_network.generate_relationship_context("wizard");
    for (rel_type, entities) in relationship_context {
        println!("- {}: {}", rel_type, entities);
    }
    
    // Generate knowledge text for a character
    println!("\nKnowledge text for wizard:");
    let knowledge_text = knowledge_system.generate_knowledge_text("wizard");
    for text in knowledge_text {
        println!("- {}", text);
    }
    
    println!("\nContext building example completed.");
}