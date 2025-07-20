use crossterm::{event::KeyCode, style::Color};
use specs::{World, Entity};
use std::collections::HashMap;
use crate::ui::{
    ui_components::{UIComponent, UIRenderCommand, UIPanel, UIText, TextAlignment},
    menu_system::{MenuRenderer, MenuInput},
};

/// Help system state
#[derive(Debug, Clone, PartialEq)]
pub enum HelpSystemState {
    MainHelp,
    Controls,
    GameMechanics,
    Combat,
    Items,
    Magic,
    Tutorial,
    ContextHelp,
    Closed,
}

/// Tutorial step information
#[derive(Debug, Clone)]
pub struct TutorialStep {
    pub id: String,
    pub title: String,
    pub content: String,
    pub trigger_condition: TutorialTrigger,
    pub completed: bool,
    pub required_action: Option<String>,
    pub next_step: Option<String>,
}

/// Tutorial trigger conditions
#[derive(Debug, Clone, PartialEq)]
pub enum TutorialTrigger {
    GameStart,
    FirstMovement,
    FirstCombat,
    FirstItemPickup,
    FirstLevelUp,
    FirstDeath,
    EnterDungeon,
    FindStairs,
    OpenInventory,
    OpenCharacterScreen,
    EquipItem,
    UseConsumable,
    Custom(String),
}

/// Context-sensitive help topics
#[derive(Debug, Clone, PartialEq)]
pub enum HelpContext {
    MainGame,
    Inventory,
    Character,
    Combat,
    Shopping,
    Dialogue,
    Menu,
}

/// Help content structure
#[derive(Debug, Clone)]
pub struct HelpContent {
    pub title: String,
    pub sections: Vec<HelpSection>,
    pub related_topics: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct HelpSection {
    pub title: String,
    pub content: Vec<String>,
    pub examples: Vec<String>,
}

/// Main help and tutorial system
pub struct HelpSystem {
    pub state: HelpSystemState,
    pub selected_topic: usize,
    pub selected_line: usize,
    pub scroll_offset: usize,
    pub help_content: HashMap<String, HelpContent>,
    pub tutorial_steps: HashMap<String, TutorialStep>,
    pub completed_tutorials: Vec<String>,
    pub current_tutorial: Option<String>,
    pub context_help_enabled: bool,
    pub tutorial_enabled: bool,
    pub show_tooltips: bool,
    pub current_context: HelpContext,
}

impl HelpSystem {
    pub fn new() -> Self {
        let mut system = HelpSystem {
            state: HelpSystemState::Closed,
            selected_topic: 0,
            selected_line: 0,
            scroll_offset: 0,
            help_content: HashMap::new(),
            tutorial_steps: HashMap::new(),
            completed_tutorials: Vec::new(),
            current_tutorial: None,
            context_help_enabled: true,
            tutorial_enabled: true,
            show_tooltips: true,
            current_context: HelpContext::MainGame,
        };

        system.initialize_help_content();
        system.initialize_tutorial_steps();
        system
    }

    pub fn open(&mut self, topic: Option<HelpSystemState>) {
        self.state = topic.unwrap_or(HelpSystemState::MainHelp);
        self.selected_topic = 0;
        self.selected_line = 0;
        self.scroll_offset = 0;
    }

    pub fn close(&mut self) {
        self.state = HelpSystemState::Closed;
    }

    pub fn is_open(&self) -> bool {
        self.state != HelpSystemState::Closed
    }

    pub fn set_context(&mut self, context: HelpContext) {
        self.current_context = context;
    }

    pub fn trigger_tutorial(&mut self, trigger: TutorialTrigger) -> Option<String> {
        if !self.tutorial_enabled {
            return None;
        }

        // Find tutorial step that matches the trigger
        for (step_id, step) in &mut self.tutorial_steps {
            if step.trigger_condition == trigger && !step.completed {
                if !self.completed_tutorials.contains(step_id) {
                    self.current_tutorial = Some(step_id.clone());
                    return Some(step.content.clone());
                }
            }
        }

        None
    }

    pub fn complete_tutorial_step(&mut self, step_id: &str) {
        if let Some(step) = self.tutorial_steps.get_mut(step_id) {
            step.completed = true;
            self.completed_tutorials.push(step_id.to_string());
            
            // Check if there's a next step
            if let Some(next_step_id) = &step.next_step {
                self.current_tutorial = Some(next_step_id.clone());
            } else {
                self.current_tutorial = None;
            }
        }
    }

    pub fn get_context_help(&self) -> Option<Vec<String>> {
        if !self.context_help_enabled {
            return None;
        }

        match self.current_context {
            HelpContext::MainGame => Some(vec![
                "Movement: WASD or Arrow Keys".to_string(),
                "Wait/Rest: Space or Period".to_string(),
                "Pickup Item: G or Comma".to_string(),
                "Open Inventory: I".to_string(),
                "Open Character: C".to_string(),
                "Help: F1 or H".to_string(),
            ]),
            HelpContext::Inventory => Some(vec![
                "Navigate: Arrow Keys or WASD".to_string(),
                "Use/Equip: E or Enter".to_string(),
                "Drop: D".to_string(),
                "Examine: X".to_string(),
                "Filter: F".to_string(),
                "Sort: O".to_string(),
                "Close: ESC or Q".to_string(),
            ]),
            HelpContext::Character => Some(vec![
                "Switch Tabs: Tab".to_string(),
                "Navigate: Arrow Keys".to_string(),
                "Allocate Points: +/-".to_string(),
                "Learn Ability: Enter".to_string(),
                "Close: ESC or Q".to_string(),
            ]),
            HelpContext::Combat => Some(vec![
                "Attack: Move into enemy".to_string(),
                "Defend: Hold Shift".to_string(),
                "Use Item: I then select".to_string(),
                "Flee: Move away".to_string(),
            ]),
            HelpContext::Shopping => Some(vec![
                "Browse: Arrow Keys".to_string(),
                "Buy: Enter or B".to_string(),
                "Sell: S".to_string(),
                "Exit: ESC or Q".to_string(),
            ]),
            HelpContext::Dialogue => Some(vec![
                "Select Option: Arrow Keys + Enter".to_string(),
                "Continue: Space or Enter".to_string(),
                "Skip: ESC".to_string(),
            ]),
            HelpContext::Menu => Some(vec![
                "Navigate: Arrow Keys".to_string(),
                "Select: Enter or Space".to_string(),
                "Back: ESC or Backspace".to_string(),
            ]),
        }
    }

    pub fn handle_key(&mut self, key: KeyCode) -> bool {
        match key {
            KeyCode::Up | KeyCode::Char('k') | KeyCode::Char('w') => {
                self.navigate_up();
                true
            }
            KeyCode::Down | KeyCode::Char('j') | KeyCode::Char('s') => {
                self.navigate_down();
                true
            }
            KeyCode::Left | KeyCode::Char('h') | KeyCode::Char('a') => {
                self.navigate_left();
                true
            }
            KeyCode::Right | KeyCode::Char('l') | KeyCode::Char('d') => {
                self.navigate_right();
                true
            }
            KeyCode::Enter | KeyCode::Char(' ') => {
                self.select_topic();
                true
            }
            KeyCode::Tab => {
                self.next_section();
                true
            }
            KeyCode::BackTab => {
                self.previous_section();
                true
            }
            KeyCode::Esc | KeyCode::Char('q') => {
                self.close();
                true
            }
            KeyCode::F1 | KeyCode::Char('?') => {
                if self.is_open() {
                    self.close();
                } else {
                    self.open(None);
                }
                true
            }
            _ => false,
        }
    }

    fn navigate_up(&mut self) {
        if self.selected_line > 0 {
            self.selected_line -= 1;
            self.ensure_line_visible();
        }
    }

    fn navigate_down(&mut self) {
        let max_lines = self.get_max_lines_for_current_state();
        if self.selected_line < max_lines.saturating_sub(1) {
            self.selected_line += 1;
            self.ensure_line_visible();
        }
    }

    fn navigate_left(&mut self) {
        if self.selected_topic > 0 {
            self.selected_topic -= 1;
            self.selected_line = 0;
            self.scroll_offset = 0;
        }
    }

    fn navigate_right(&mut self) {
        let max_topics = self.get_max_topics_for_current_state();
        if self.selected_topic < max_topics.saturating_sub(1) {
            self.selected_topic += 1;
            self.selected_line = 0;
            self.scroll_offset = 0;
        }
    }

    fn select_topic(&mut self) {
        // Navigate to selected help topic
        match self.state {
            HelpSystemState::MainHelp => {
                self.state = match self.selected_topic {
                    0 => HelpSystemState::Controls,
                    1 => HelpSystemState::GameMechanics,
                    2 => HelpSystemState::Combat,
                    3 => HelpSystemState::Items,
                    4 => HelpSystemState::Magic,
                    5 => HelpSystemState::Tutorial,
                    _ => HelpSystemState::MainHelp,
                };
                self.selected_line = 0;
                self.scroll_offset = 0;
            }
            _ => {}
        }
    }

    fn next_section(&mut self) {
        match self.state {
            HelpSystemState::MainHelp => {
                self.state = HelpSystemState::Controls;
            }
            HelpSystemState::Controls => {
                self.state = HelpSystemState::GameMechanics;
            }
            HelpSystemState::GameMechanics => {
                self.state = HelpSystemState::Combat;
            }
            HelpSystemState::Combat => {
                self.state = HelpSystemState::Items;
            }
            HelpSystemState::Items => {
                self.state = HelpSystemState::Magic;
            }
            HelpSystemState::Magic => {
                self.state = HelpSystemState::Tutorial;
            }
            HelpSystemState::Tutorial => {
                self.state = HelpSystemState::MainHelp;
            }
            _ => {}
        }
        self.selected_line = 0;
        self.scroll_offset = 0;
    }

    fn previous_section(&mut self) {
        match self.state {
            HelpSystemState::MainHelp => {
                self.state = HelpSystemState::Tutorial;
            }
            HelpSystemState::Controls => {
                self.state = HelpSystemState::MainHelp;
            }
            HelpSystemState::GameMechanics => {
                self.state = HelpSystemState::Controls;
            }
            HelpSystemState::Combat => {
                self.state = HelpSystemState::GameMechanics;
            }
            HelpSystemState::Items => {
                self.state = HelpSystemState::Combat;
            }
            HelpSystemState::Magic => {
                self.state = HelpSystemState::Items;
            }
            HelpSystemState::Tutorial => {
                self.state = HelpSystemState::Magic;
            }
            _ => {}
        }
        self.selected_line = 0;
        self.scroll_offset = 0;
    }

    fn get_max_lines_for_current_state(&self) -> usize {
        match self.state {
            HelpSystemState::MainHelp => 6, // Number of main help topics
            _ => {
                if let Some(content) = self.get_current_help_content() {
                    content.sections.iter().map(|s| s.content.len() + s.examples.len() + 2).sum()
                } else {
                    0
                }
            }
        }
    }

    fn get_max_topics_for_current_state(&self) -> usize {
        match self.state {
            HelpSystemState::MainHelp => 6,
            _ => 1,
        }
    }

    fn ensure_line_visible(&mut self) {
        let lines_per_page = 20; // Adjust based on screen size
        
        if self.selected_line < self.scroll_offset {
            self.scroll_offset = self.selected_line;
        } else if self.selected_line >= self.scroll_offset + lines_per_page {
            self.scroll_offset = self.selected_line - lines_per_page + 1;
        }
    }

    fn get_current_help_content(&self) -> Option<&HelpContent> {
        let topic_key = match self.state {
            HelpSystemState::Controls => "controls",
            HelpSystemState::GameMechanics => "game_mechanics",
            HelpSystemState::Combat => "combat",
            HelpSystemState::Items => "items",
            HelpSystemState::Magic => "magic",
            HelpSystemState::Tutorial => "tutorial",
            _ => return None,
        };

        self.help_content.get(topic_key)
    }

    fn initialize_help_content(&mut self) {
        // Controls help
        let controls_content = HelpContent {
            title: "Game Controls".to_string(),
            sections: vec![
                HelpSection {
                    title: "Movement".to_string(),
                    content: vec![
                        "Use WASD keys or Arrow keys to move your character".to_string(),
                        "You can move in 8 directions (including diagonally)".to_string(),
                        "Moving into an enemy will attack them".to_string(),
                        "Moving into walls or obstacles is not possible".to_string(),
                    ],
                    examples: vec![
                        "W/↑: Move North".to_string(),
                        "S/↓: Move South".to_string(),
                        "A/←: Move West".to_string(),
                        "D/→: Move East".to_string(),
                    ],
                },
                HelpSection {
                    title: "Actions".to_string(),
                    content: vec![
                        "Space or Period: Wait/Rest (skip turn)".to_string(),
                        "G or Comma: Pick up items".to_string(),
                        "I: Open inventory".to_string(),
                        "C: Open character screen".to_string(),
                        "ESC: Open main menu or close current screen".to_string(),
                    ],
                    examples: vec![
                        "Press G when standing on an item to pick it up".to_string(),
                        "Press I to manage your inventory".to_string(),
                        "Press C to view character stats and abilities".to_string(),
                    ],
                },
                HelpSection {
                    title: "Interface".to_string(),
                    content: vec![
                        "F1 or H: Open this help system".to_string(),
                        "Tab: Switch between interface sections".to_string(),
                        "Enter: Confirm selection or activate item".to_string(),
                        "ESC: Cancel or go back".to_string(),
                    ],
                    examples: vec![
                        "Use Tab to navigate between inventory tabs".to_string(),
                        "Press Enter to use selected items".to_string(),
                        "Press ESC to close any open window".to_string(),
                    ],
                },
            ],
            related_topics: vec!["game_mechanics".to_string(), "combat".to_string()],
        };

        self.help_content.insert("controls".to_string(), controls_content);

        // Game Mechanics help
        let mechanics_content = HelpContent {
            title: "Game Mechanics".to_string(),
            sections: vec![
                HelpSection {
                    title: "Character Progression".to_string(),
                    content: vec![
                        "Gain experience by defeating enemies and completing objectives".to_string(),
                        "Level up to increase your attributes and learn new abilities".to_string(),
                        "Allocate attribute points to customize your character".to_string(),
                        "Learn skills through practice and training".to_string(),
                    ],
                    examples: vec![
                        "Each level grants 2 attribute points and 1 ability point".to_string(),
                        "Higher attributes improve related skills and abilities".to_string(),
                    ],
                },
                HelpSection {
                    title: "Dungeon Exploration".to_string(),
                    content: vec![
                        "Explore procedurally generated dungeons".to_string(),
                        "Find stairs (< >) to move between dungeon levels".to_string(),
                        "Search for treasure chests and hidden items".to_string(),
                        "Be careful of traps and environmental hazards".to_string(),
                    ],
                    examples: vec![
                        "Use > to go down stairs to deeper levels".to_string(),
                        "Use < to go up stairs to previous levels".to_string(),
                        "Look for secret doors and hidden passages".to_string(),
                    ],
                },
                HelpSection {
                    title: "Turn-Based System".to_string(),
                    content: vec![
                        "The game uses a turn-based system".to_string(),
                        "Each action takes one turn".to_string(),
                        "Enemies move after you complete your turn".to_string(),
                        "Plan your moves carefully".to_string(),
                    ],
                    examples: vec![
                        "Moving, attacking, and using items all take one turn".to_string(),
                        "Waiting/resting also takes one turn".to_string(),
                    ],
                },
            ],
            related_topics: vec!["controls".to_string(), "combat".to_string()],
        };

        self.help_content.insert("game_mechanics".to_string(), mechanics_content);

        // Combat help
        let combat_content = HelpContent {
            title: "Combat System".to_string(),
            sections: vec![
                HelpSection {
                    title: "Basic Combat".to_string(),
                    content: vec![
                        "Move into an enemy to attack them".to_string(),
                        "Your attack power determines damage dealt".to_string(),
                        "Enemy defense reduces incoming damage".to_string(),
                        "Health reaches 0 means death".to_string(),
                    ],
                    examples: vec![
                        "Higher Strength increases melee damage".to_string(),
                        "Higher Dexterity increases ranged damage and accuracy".to_string(),
                        "Higher Constitution increases health and defense".to_string(),
                    ],
                },
                HelpSection {
                    title: "Advanced Combat".to_string(),
                    content: vec![
                        "Use abilities for special attacks".to_string(),
                        "Equip better weapons and armor".to_string(),
                        "Use consumables during combat".to_string(),
                        "Position yourself strategically".to_string(),
                    ],
                    examples: vec![
                        "Doorways can limit enemy numbers".to_string(),
                        "Ranged weapons work better at distance".to_string(),
                        "Healing potions can save your life".to_string(),
                    ],
                },
            ],
            related_topics: vec!["items".to_string(), "game_mechanics".to_string()],
        };

        self.help_content.insert("combat".to_string(), combat_content);

        // Items help
        let items_content = HelpContent {
            title: "Items and Equipment".to_string(),
            sections: vec![
                HelpSection {
                    title: "Item Types".to_string(),
                    content: vec![
                        "Weapons: Increase attack power".to_string(),
                        "Armor: Increase defense".to_string(),
                        "Consumables: Provide temporary effects".to_string(),
                        "Tools: Utility items for exploration".to_string(),
                    ],
                    examples: vec![
                        "Swords, axes, and bows are weapons".to_string(),
                        "Helmets, armor, and shields provide protection".to_string(),
                        "Potions and food restore health".to_string(),
                    ],
                },
                HelpSection {
                    title: "Equipment".to_string(),
                    content: vec![
                        "Equip items to gain their benefits".to_string(),
                        "Each equipment slot can hold one item".to_string(),
                        "Better equipment provides greater bonuses".to_string(),
                        "Some items have special properties".to_string(),
                    ],
                    examples: vec![
                        "Press E in inventory to equip items".to_string(),
                        "Equipped items show [E] in inventory".to_string(),
                        "Compare items to see which is better".to_string(),
                    ],
                },
            ],
            related_topics: vec!["combat".to_string(), "controls".to_string()],
        };

        self.help_content.insert("items".to_string(), items_content);

        // Magic help
        let magic_content = HelpContent {
            title: "Magic System".to_string(),
            sections: vec![
                HelpSection {
                    title: "Magic Schools".to_string(),
                    content: vec![
                        "Arcane Magic: Offensive spells and enchantments".to_string(),
                        "Divine Magic: Healing and protection spells".to_string(),
                        "Learn spells by increasing magic skills".to_string(),
                        "Mana is required to cast spells".to_string(),
                    ],
                    examples: vec![
                        "Intelligence governs Arcane Magic".to_string(),
                        "Wisdom governs Divine Magic".to_string(),
                        "Higher skill levels unlock more powerful spells".to_string(),
                    ],
                },
                HelpSection {
                    title: "Spell Casting".to_string(),
                    content: vec![
                        "Select spells from your spellbook".to_string(),
                        "Target spells appropriately".to_string(),
                        "Manage your mana carefully".to_string(),
                        "Some spells have cooldowns".to_string(),
                    ],
                    examples: vec![
                        "Healing spells target yourself or allies".to_string(),
                        "Attack spells target enemies".to_string(),
                        "Area spells affect multiple targets".to_string(),
                    ],
                },
            ],
            related_topics: vec!["game_mechanics".to_string(), "combat".to_string()],
        };

        self.help_content.insert("magic".to_string(), magic_content);

        // Tutorial help
        let tutorial_content = HelpContent {
            title: "Tutorial System".to_string(),
            sections: vec![
                HelpSection {
                    title: "Getting Started".to_string(),
                    content: vec![
                        "Follow the tutorial messages for guidance".to_string(),
                        "Tutorial tips appear automatically".to_string(),
                        "You can disable tutorials in settings".to_string(),
                        "Context help is available with F1".to_string(),
                    ],
                    examples: vec![
                        "Tutorial messages appear at the bottom of screen".to_string(),
                        "Press any key to dismiss tutorial messages".to_string(),
                        "F1 opens context-sensitive help".to_string(),
                    ],
                },
            ],
            related_topics: vec!["controls".to_string(), "game_mechanics".to_string()],
        };

        self.help_content.insert("tutorial".to_string(), tutorial_content);
    }

    fn initialize_tutorial_steps(&mut self) {
        // Game start tutorial
        self.tutorial_steps.insert("welcome".to_string(), TutorialStep {
            id: "welcome".to_string(),
            title: "Welcome to ASCII Dungeon Explorer!".to_string(),
            content: "Welcome, brave adventurer! Use WASD or arrow keys to move around. Press F1 for help anytime.".to_string(),
            trigger_condition: TutorialTrigger::GameStart,
            completed: false,
            required_action: Some("Move your character".to_string()),
            next_step: Some("first_movement".to_string()),
        });

        // First movement tutorial
        self.tutorial_steps.insert("first_movement".to_string(), TutorialStep {
            id: "first_movement".to_string(),
            title: "Great! You're moving!".to_string(),
            content: "Excellent! You can move in 8 directions. Try exploring the area. Press G to pick up items you find.".to_string(),
            trigger_condition: TutorialTrigger::FirstMovement,
            completed: false,
            required_action: Some("Pick up an item".to_string()),
            next_step: Some("first_item".to_string()),
        });

        // First item pickup tutorial
        self.tutorial_steps.insert("first_item".to_string(), TutorialStep {
            id: "first_item".to_string(),
            title: "Item Collected!".to_string(),
            content: "Nice! You picked up an item. Press I to open your inventory and manage your items.".to_string(),
            trigger_condition: TutorialTrigger::FirstItemPickup,
            completed: false,
            required_action: Some("Open inventory".to_string()),
            next_step: Some("inventory_tutorial".to_string()),
        });

        // Inventory tutorial
        self.tutorial_steps.insert("inventory_tutorial".to_string(), TutorialStep {
            id: "inventory_tutorial".to_string(),
            title: "Inventory Management".to_string(),
            content: "This is your inventory! Use arrow keys to navigate, E to equip items, and ESC to close.".to_string(),
            trigger_condition: TutorialTrigger::OpenInventory,
            completed: false,
            required_action: Some("Equip an item".to_string()),
            next_step: Some("equipment_tutorial".to_string()),
        });

        // Equipment tutorial
        self.tutorial_steps.insert("equipment_tutorial".to_string(), TutorialStep {
            id: "equipment_tutorial".to_string(),
            title: "Equipment Equipped!".to_string(),
            content: "Great! Equipped items improve your stats. Press C to view your character screen and see your progress.".to_string(),
            trigger_condition: TutorialTrigger::EquipItem,
            completed: false,
            required_action: Some("Open character screen".to_string()),
            next_step: Some("character_tutorial".to_string()),
        });

        // Character screen tutorial
        self.tutorial_steps.insert("character_tutorial".to_string(), TutorialStep {
            id: "character_tutorial".to_string(),
            title: "Character Development".to_string(),
            content: "This shows your character's attributes, skills, and abilities. Level up to gain points to spend here!".to_string(),
            trigger_condition: TutorialTrigger::OpenCharacterScreen,
            completed: false,
            required_action: Some("Find enemies to fight".to_string()),
            next_step: Some("combat_tutorial".to_string()),
        });

        // Combat tutorial
        self.tutorial_steps.insert("combat_tutorial".to_string(), TutorialStep {
            id: "combat_tutorial".to_string(),
            title: "Combat Basics".to_string(),
            content: "To attack an enemy, simply move into them! Your weapon and stats determine damage dealt.".to_string(),
            trigger_condition: TutorialTrigger::FirstCombat,
            completed: false,
            required_action: Some("Defeat an enemy".to_string()),
            next_step: Some("victory_tutorial".to_string()),
        });

        // Victory tutorial
        self.tutorial_steps.insert("victory_tutorial".to_string(), TutorialStep {
            id: "victory_tutorial".to_string(),
            title: "Victory!".to_string(),
            content: "Well done! You defeated an enemy and gained experience. Keep exploring to find stairs (< >) to deeper levels.".to_string(),
            trigger_condition: TutorialTrigger::Custom("enemy_defeated".to_string()),
            completed: false,
            required_action: Some("Find the stairs".to_string()),
            next_step: Some("stairs_tutorial".to_string()),
        });

        // Stairs tutorial
        self.tutorial_steps.insert("stairs_tutorial".to_string(), TutorialStep {
            id: "stairs_tutorial".to_string(),
            title: "Dungeon Exploration".to_string(),
            content: "You found the stairs! Use > to go down to deeper, more dangerous levels with better rewards.".to_string(),
            trigger_condition: TutorialTrigger::FindStairs,
            completed: false,
            required_action: None,
            next_step: None,
        });
    }

    pub fn render(&self, screen_width: i32, screen_height: i32) -> Vec<UIRenderCommand> {
        match self.state {
            HelpSystemState::MainHelp => self.render_main_help(screen_width, screen_height),
            HelpSystemState::Controls => self.render_help_content("controls", screen_width, screen_height),
            HelpSystemState::GameMechanics => self.render_help_content("game_mechanics", screen_width, screen_height),
            HelpSystemState::Combat => self.render_help_content("combat", screen_width, screen_height),
            HelpSystemState::Items => self.render_help_content("items", screen_width, screen_height),
            HelpSystemState::Magic => self.render_help_content("magic", screen_width, screen_height),
            HelpSystemState::Tutorial => self.render_tutorial_help(screen_width, screen_height),
            HelpSystemState::ContextHelp => self.render_context_help(screen_width, screen_height),
            HelpSystemState::Closed => Vec::new(),
        }
    }

    fn render_main_help(&self, screen_width: i32, screen_height: i32) -> Vec<UIRenderCommand> {
        let mut commands = Vec::new();

        // Main help panel
        let panel_width = screen_width - 4;
        let panel_height = screen_height - 4;
        let panel = UIPanel::new(
            "Help System".to_string(),
            2,
            2,
            panel_width,
            panel_height,
        ).with_colors(Color::White, Color::Black, Color::Yellow);

        commands.extend(panel.render());

        // Help topics
        let topics = vec![
            ("Controls", "Learn the basic game controls"),
            ("Game Mechanics", "Understand how the game works"),
            ("Combat", "Master the combat system"),
            ("Items & Equipment", "Manage your inventory and gear"),
            ("Magic System", "Learn about spells and magic"),
            ("Tutorial", "Replay tutorial messages"),
        ];

        let start_y = 5;
        commands.push(UIRenderCommand::DrawText {
            x: 4,
            y: start_y,
            text: "Select a help topic:".to_string(),
            fg: Color::Cyan,
            bg: Color::Black,
        });

        for (i, (topic, description)) in topics.iter().enumerate() {
            let y = start_y + 2 + i as i32;
            let is_selected = i == self.selected_topic;

            let (fg, bg) = if is_selected {
                (Color::Black, Color::White)
            } else {
                (Color::White, Color::Black)
            };

            commands.push(UIRenderCommand::DrawText {
                x: 4,
                y,
                text: format!("{:<20} - {}", topic, description),
                fg,
                bg,
            });
        }

        // Navigation help
        let nav_y = screen_height - 4;
        commands.push(UIRenderCommand::DrawText {
            x: 4,
            y: nav_y,
            text: "Navigation: ↑↓ Select, Enter: Open Topic, Tab: Next Section, ESC: Close".to_string(),
            fg: Color::DarkGrey,
            bg: Color::Black,
        });

        commands
    }

    fn render_help_content(&self, topic_key: &str, screen_width: i32, screen_height: i32) -> Vec<UIRenderCommand> {
        let mut commands = Vec::new();

        if let Some(content) = self.help_content.get(topic_key) {
            // Content panel
            let panel_width = screen_width - 4;
            let panel_height = screen_height - 4;
            let panel = UIPanel::new(
                content.title.clone(),
                2,
                2,
                panel_width,
                panel_height,
            ).with_colors(Color::White, Color::Black, Color::Yellow);

            commands.extend(panel.render());

            let mut y = 5;
            let content_width = panel_width - 4;
            let max_content_height = panel_height - 8;

            // Render sections
            let mut line_count = 0;
            for section in &content.sections {
                // Skip lines before scroll offset
                if line_count < self.scroll_offset {
                    line_count += section.content.len() + section.examples.len() + 2;
                    continue;
                }

                // Check if we have room for more content
                if y >= 5 + max_content_height {
                    break;
                }

                // Section title
                commands.push(UIRenderCommand::DrawText {
                    x: 4,
                    y,
                    text: section.title.clone(),
                    fg: Color::Yellow,
                    bg: Color::Black,
                });
                y += 1;
                line_count += 1;

                // Section content
                for content_line in &section.content {
                    if y >= 5 + max_content_height {
                        break;
                    }

                    let wrapped_lines = self.wrap_text(content_line, content_width as usize - 2);
                    for wrapped_line in wrapped_lines {
                        if y >= 5 + max_content_height {
                            break;
                        }

                        commands.push(UIRenderCommand::DrawText {
                            x: 6,
                            y,
                            text: wrapped_line,
                            fg: Color::White,
                            bg: Color::Black,
                        });
                        y += 1;
                        line_count += 1;
                    }
                }

                // Examples
                if !section.examples.is_empty() && y < 5 + max_content_height {
                    commands.push(UIRenderCommand::DrawText {
                        x: 6,
                        y,
                        text: "Examples:".to_string(),
                        fg: Color::Cyan,
                        bg: Color::Black,
                    });
                    y += 1;
                    line_count += 1;

                    for example in &section.examples {
                        if y >= 5 + max_content_height {
                            break;
                        }

                        commands.push(UIRenderCommand::DrawText {
                            x: 8,
                            y,
                            text: format!("• {}", example),
                            fg: Color::Green,
                            bg: Color::Black,
                        });
                        y += 1;
                        line_count += 1;
                    }
                }

                y += 1; // Space between sections
                line_count += 1;
            }

            // Related topics
            if !content.related_topics.is_empty() && y < 5 + max_content_height {
                commands.push(UIRenderCommand::DrawText {
                    x: 4,
                    y,
                    text: "Related Topics:".to_string(),
                    fg: Color::Magenta,
                    bg: Color::Black,
                });
                y += 1;

                for related in &content.related_topics {
                    if y >= 5 + max_content_height {
                        break;
                    }

                    commands.push(UIRenderCommand::DrawText {
                        x: 6,
                        y,
                        text: format!("• {}", related.replace("_", " ")),
                        fg: Color::Cyan,
                        bg: Color::Black,
                    });
                    y += 1;
                }
            }

            // Scroll indicator
            if self.scroll_offset > 0 || line_count > max_content_height as usize {
                let scroll_text = if self.scroll_offset > 0 && line_count > max_content_height as usize {
                    "↑↓ Scroll"
                } else if self.scroll_offset > 0 {
                    "↑ More above"
                } else {
                    "↓ More below"
                };

                commands.push(UIRenderCommand::DrawText {
                    x: panel_width - 15,
                    y: 5,
                    text: scroll_text.to_string(),
                    fg: Color::DarkGrey,
                    bg: Color::Black,
                });
            }
        }

        // Navigation help
        let nav_y = screen_height - 4;
        commands.push(UIRenderCommand::DrawText {
            x: 4,
            y: nav_y,
            text: "Navigation: ↑↓ Scroll, Tab: Next Section, Shift+Tab: Previous, ESC: Back".to_string(),
            fg: Color::DarkGrey,
            bg: Color::Black,
        });

        commands
    }

    fn render_tutorial_help(&self, screen_width: i32, screen_height: i32) -> Vec<UIRenderCommand> {
        let mut commands = Vec::new();

        // Tutorial panel
        let panel_width = screen_width - 4;
        let panel_height = screen_height - 4;
        let panel = UIPanel::new(
            "Tutorial System".to_string(),
            2,
            2,
            panel_width,
            panel_height,
        ).with_colors(Color::White, Color::Black, Color::Yellow);

        commands.extend(panel.render());

        let mut y = 5;

        // Tutorial status
        commands.push(UIRenderCommand::DrawText {
            x: 4,
            y,
            text: format!("Tutorial Status: {}", if self.tutorial_enabled { "Enabled" } else { "Disabled" }),
            fg: if self.tutorial_enabled { Color::Green } else { Color::Red },
            bg: Color::Black,
        });
        y += 2;

        // Current tutorial
        if let Some(current_tutorial) = &self.current_tutorial {
            if let Some(step) = self.tutorial_steps.get(current_tutorial) {
                commands.push(UIRenderCommand::DrawText {
                    x: 4,
                    y,
                    text: "Current Tutorial:".to_string(),
                    fg: Color::Cyan,
                    bg: Color::Black,
                });
                y += 1;

                commands.push(UIRenderCommand::DrawText {
                    x: 6,
                    y,
                    text: step.title.clone(),
                    fg: Color::Yellow,
                    bg: Color::Black,
                });
                y += 1;

                let wrapped_content = self.wrap_text(&step.content, (panel_width - 8) as usize);
                for line in wrapped_content {
                    commands.push(UIRenderCommand::DrawText {
                        x: 6,
                        y,
                        text: line,
                        fg: Color::White,
                        bg: Color::Black,
                    });
                    y += 1;
                }
                y += 1;
            }
        } else {
            commands.push(UIRenderCommand::DrawText {
                x: 4,
                y,
                text: "No active tutorial".to_string(),
                fg: Color::Grey,
                bg: Color::Black,
            });
            y += 2;
        }

        // Completed tutorials
        commands.push(UIRenderCommand::DrawText {
            x: 4,
            y,
            text: format!("Completed Tutorials: {}/{}", 
                self.completed_tutorials.len(), 
                self.tutorial_steps.len()),
            fg: Color::Cyan,
            bg: Color::Black,
        });
        y += 2;

        // Tutorial list
        let tutorial_names: Vec<String> = self.tutorial_steps.keys().cloned().collect();
        let visible_tutorials = tutorial_names.iter()
            .skip(self.scroll_offset)
            .take(10);

        for (i, tutorial_id) in visible_tutorials.enumerate() {
            if let Some(step) = self.tutorial_steps.get(tutorial_id) {
                let is_completed = self.completed_tutorials.contains(tutorial_id);
                let is_current = self.current_tutorial.as_ref() == Some(tutorial_id);

                let status_icon = if is_completed {
                    "✓"
                } else if is_current {
                    "→"
                } else {
                    "○"
                };

                let (fg, bg) = if is_current {
                    (Color::Black, Color::Yellow)
                } else if is_completed {
                    (Color::Green, Color::Black)
                } else {
                    (Color::White, Color::Black)
                };

                commands.push(UIRenderCommand::DrawText {
                    x: 4,
                    y,
                    text: format!("{} {}", status_icon, step.title),
                    fg,
                    bg,
                });
                y += 1;
            }
        }

        commands
    }

    fn render_context_help(&self, screen_width: i32, screen_height: i32) -> Vec<UIRenderCommand> {
        let mut commands = Vec::new();

        if let Some(help_lines) = self.get_context_help() {
            // Context help panel (smaller, positioned at bottom-right)
            let panel_width = 40;
            let panel_height = help_lines.len() as i32 + 4;
            let panel_x = screen_width - panel_width - 2;
            let panel_y = screen_height - panel_height - 2;

            let panel = UIPanel::new(
                "Quick Help".to_string(),
                panel_x,
                panel_y,
                panel_width,
                panel_height,
            ).with_colors(Color::Yellow, Color::DarkBlue, Color::White);

            commands.extend(panel.render());

            let mut y = panel_y + 2;
            for help_line in help_lines {
                commands.push(UIRenderCommand::DrawText {
                    x: panel_x + 2,
                    y,
                    text: help_line,
                    fg: Color::White,
                    bg: Color::DarkBlue,
                });
                y += 1;
            }
        }

        commands
    }

    fn wrap_text(&self, text: &str, width: usize) -> Vec<String> {
        if width == 0 {
            return vec![text.to_string()];
        }

        let mut lines = Vec::new();
        let words: Vec<&str> = text.split_whitespace().collect();
        let mut current_line = String::new();

        for word in words {
            if current_line.is_empty() {
                current_line = word.to_string();
            } else if current_line.len() + 1 + word.len() <= width {
                current_line.push(' ');
                current_line.push_str(word);
            } else {
                lines.push(current_line);
                current_line = word.to_string();
            }
        }

        if !current_line.is_empty() {
            lines.push(current_line);
        }

        if lines.is_empty() {
            lines.push(String::new());
        }

        lines
    }
}

/// Tutorial message display component
pub struct TutorialMessage {
    pub message: String,
    pub title: String,
    pub visible: bool,
    pub auto_dismiss_timer: Option<f32>,
    pub requires_acknowledgment: bool,
}

impl TutorialMessage {
    pub fn new(title: String, message: String) -> Self {
        TutorialMessage {
            message,
            title,
            visible: true,
            auto_dismiss_timer: None,
            requires_acknowledgment: true,
        }
    }

    pub fn with_auto_dismiss(mut self, seconds: f32) -> Self {
        self.auto_dismiss_timer = Some(seconds);
        self.requires_acknowledgment = false;
        self
    }

    pub fn update(&mut self, delta_time: f32) {
        if let Some(timer) = &mut self.auto_dismiss_timer {
            *timer -= delta_time;
            if *timer <= 0.0 {
                self.visible = false;
            }
        }
    }

    pub fn dismiss(&mut self) {
        self.visible = false;
    }

    pub fn render(&self, screen_width: i32, screen_height: i32) -> Vec<UIRenderCommand> {
        if !self.visible {
            return Vec::new();
        }

        let mut commands = Vec::new();

        // Tutorial message panel (bottom of screen)
        let panel_width = screen_width - 4;
        let panel_height = 6;
        let panel_x = 2;
        let panel_y = screen_height - panel_height - 2;

        let panel = UIPanel::new(
            self.title.clone(),
            panel_x,
            panel_y,
            panel_width,
            panel_height,
        ).with_colors(Color::Yellow, Color::DarkBlue, Color::White);

        commands.extend(panel.render());

        // Message content
        let content_width = panel_width - 4;
        let wrapped_lines = self.wrap_text(&self.message, content_width as usize);
        
        let mut y = panel_y + 2;
        for line in wrapped_lines.iter().take(3) { // Limit to 3 lines
            commands.push(UIRenderCommand::DrawText {
                x: panel_x + 2,
                y,
                text: line.clone(),
                fg: Color::White,
                bg: Color::DarkBlue,
            });
            y += 1;
        }

        // Dismissal instruction
        if self.requires_acknowledgment {
            commands.push(UIRenderCommand::DrawText {
                x: panel_x + 2,
                y: panel_y + panel_height - 2,
                text: "Press any key to continue...".to_string(),
                fg: Color::Yellow,
                bg: Color::DarkBlue,
            });
        } else if let Some(timer) = self.auto_dismiss_timer {
            commands.push(UIRenderCommand::DrawText {
                x: panel_x + 2,
                y: panel_y + panel_height - 2,
                text: format!("Auto-dismiss in {:.1}s", timer),
                fg: Color::DarkGrey,
                bg: Color::DarkBlue,
            });
        }

        commands
    }

    fn wrap_text(&self, text: &str, width: usize) -> Vec<String> {
        if width == 0 {
            return vec![text.to_string()];
        }

        let mut lines = Vec::new();
        let words: Vec<&str> = text.split_whitespace().collect();
        let mut current_line = String::new();

        for word in words {
            if current_line.is_empty() {
                current_line = word.to_string();
            } else if current_line.len() + 1 + word.len() <= width {
                current_line.push(' ');
                current_line.push_str(word);
            } else {
                lines.push(current_line);
                current_line = word.to_string();
            }
        }

        if !current_line.is_empty() {
            lines.push(current_line);
        }

        if lines.is_empty() {
            lines.push(String::new());
        }

        lines
    }
}

impl UIComponent for HelpSystem {
    fn render(&self, _x: i32, _y: i32, width: i32, height: i32) -> Vec<UIRenderCommand> {
        self.render(width, height)
    }

    fn handle_input(&mut self, input: char) -> bool {
        let key = match input {
            '\n' => KeyCode::Enter,
            '\x1b' => KeyCode::Esc,
            '\t' => KeyCode::Tab,
            'k' | 'w' => KeyCode::Up,
            'j' | 's' => KeyCode::Down,
            'h' | 'a' => KeyCode::Left,
            'l' | 'd' => KeyCode::Right,
            '?' => KeyCode::F1,
            c => KeyCode::Char(c),
        };

        self.handle_key(key)
    }

    fn is_focused(&self) -> bool {
        self.is_open()
    }

    fn set_focus(&mut self, focused: bool) {
        if !focused {
            self.close();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_help_system_creation() {
        let help_system = HelpSystem::new();
        
        assert_eq!(help_system.state, HelpSystemState::Closed);
        assert!(help_system.tutorial_enabled);
        assert!(help_system.context_help_enabled);
        assert!(!help_system.help_content.is_empty());
        assert!(!help_system.tutorial_steps.is_empty());
    }

    #[test]
    fn test_help_system_open_close() {
        let mut help_system = HelpSystem::new();
        
        assert!(!help_system.is_open());
        
        help_system.open(Some(HelpSystemState::Controls));
        assert!(help_system.is_open());
        assert_eq!(help_system.state, HelpSystemState::Controls);
        
        help_system.close();
        assert!(!help_system.is_open());
        assert_eq!(help_system.state, HelpSystemState::Closed);
    }

    #[test]
    fn test_tutorial_trigger() {
        let mut help_system = HelpSystem::new();
        
        let message = help_system.trigger_tutorial(TutorialTrigger::GameStart);
        assert!(message.is_some());
        assert!(message.unwrap().contains("Welcome"));
        
        // Same trigger shouldn't fire again
        let message2 = help_system.trigger_tutorial(TutorialTrigger::GameStart);
        assert!(message2.is_none());
    }

    #[test]
    fn test_tutorial_completion() {
        let mut help_system = HelpSystem::new();
        
        assert!(!help_system.completed_tutorials.contains(&"welcome".to_string()));
        
        help_system.complete_tutorial_step("welcome");
        assert!(help_system.completed_tutorials.contains(&"welcome".to_string()));
        
        if let Some(step) = help_system.tutorial_steps.get("welcome") {
            assert!(step.completed);
        }
    }

    #[test]
    fn test_context_help() {
        let mut help_system = HelpSystem::new();
        
        help_system.set_context(HelpContext::Inventory);
        let help = help_system.get_context_help();
        assert!(help.is_some());
        
        let help_lines = help.unwrap();
        assert!(!help_lines.is_empty());
        assert!(help_lines.iter().any(|line| line.contains("Navigate")));
    }

    #[test]
    fn test_help_content_exists() {
        let help_system = HelpSystem::new();
        
        assert!(help_system.help_content.contains_key("controls"));
        assert!(help_system.help_content.contains_key("game_mechanics"));
        assert!(help_system.help_content.contains_key("combat"));
        assert!(help_system.help_content.contains_key("items"));
        assert!(help_system.help_content.contains_key("magic"));
        assert!(help_system.help_content.contains_key("tutorial"));
    }

    #[test]
    fn test_tutorial_message() {
        let mut message = TutorialMessage::new(
            "Test".to_string(),
            "This is a test message".to_string(),
        );
        
        assert!(message.visible);
        assert!(message.requires_acknowledgment);
        
        message.dismiss();
        assert!(!message.visible);
    }

    #[test]
    fn test_tutorial_message_auto_dismiss() {
        let mut message = TutorialMessage::new(
            "Test".to_string(),
            "This is a test message".to_string(),
        ).with_auto_dismiss(1.0);
        
        assert!(message.visible);
        assert!(!message.requires_acknowledgment);
        
        message.update(0.5);
        assert!(message.visible);
        
        message.update(0.6);
        assert!(!message.visible);
    }

    #[test]
    fn test_text_wrapping() {
        let help_system = HelpSystem::new();
        
        let text = "This is a long line that should be wrapped";
        let wrapped = help_system.wrap_text(text, 20);
        
        assert!(wrapped.len() > 1);
        for line in &wrapped {
            assert!(line.len() <= 20);
        }
    }
}