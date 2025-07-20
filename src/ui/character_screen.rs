use crossterm::{event::KeyCode, style::Color};
use specs::{World, Entity, Join, ReadStorage, WriteStorage, WorldExt};
use std::collections::HashMap;
use crate::components::{Player, Name, CombatStats};
use crate::items::{Equipment, ItemBonuses};
use crate::ui::{
    ui_components::{UIComponent, UIRenderCommand, UIPanel, UIText, TextAlignment},
    menu_system::{MenuRenderer, MenuInput},
};

/// Character screen state
#[derive(Debug, Clone, PartialEq)]
pub enum CharacterScreenState {
    Overview,
    Attributes,
    Skills,
    Abilities,
    Progression,
    StatAllocation,
    Closed,
}

/// Character attributes system
#[derive(Debug, Clone)]
pub struct CharacterAttributes {
    pub strength: i32,
    pub dexterity: i32,
    pub constitution: i32,
    pub intelligence: i32,
    pub wisdom: i32,
    pub charisma: i32,
    pub base_values: HashMap<String, i32>,
    pub bonuses: HashMap<String, i32>,
}

impl CharacterAttributes {
    pub fn new() -> Self {
        let mut base_values = HashMap::new();
        base_values.insert("Strength".to_string(), 10);
        base_values.insert("Dexterity".to_string(), 10);
        base_values.insert("Constitution".to_string(), 10);
        base_values.insert("Intelligence".to_string(), 10);
        base_values.insert("Wisdom".to_string(), 10);
        base_values.insert("Charisma".to_string(), 10);

        CharacterAttributes {
            strength: 10,
            dexterity: 10,
            constitution: 10,
            intelligence: 10,
            wisdom: 10,
            charisma: 10,
            base_values,
            bonuses: HashMap::new(),
        }
    }

    pub fn get_total(&self, attribute: &str) -> i32 {
        let base = self.base_values.get(attribute).unwrap_or(&10);
        let bonus = self.bonuses.get(attribute).unwrap_or(&0);
        base + bonus
    }

    pub fn get_modifier(&self, attribute: &str) -> i32 {
        (self.get_total(attribute) - 10) / 2
    }

    pub fn add_bonus(&mut self, attribute: String, bonus: i32) {
        *self.bonuses.entry(attribute).or_insert(0) += bonus;
    }

    pub fn set_base(&mut self, attribute: String, value: i32) {
        self.base_values.insert(attribute, value);
    }
}

/// Character skills system
#[derive(Debug, Clone)]
pub struct CharacterSkills {
    pub skills: HashMap<String, SkillInfo>,
}

#[derive(Debug, Clone)]
pub struct SkillInfo {
    pub level: i32,
    pub experience: i32,
    pub max_level: i32,
    pub governing_attribute: String,
    pub description: String,
}

impl CharacterSkills {
    pub fn new() -> Self {
        let mut skills = HashMap::new();
        
        // Combat skills
        skills.insert("Melee Combat".to_string(), SkillInfo {
            level: 1,
            experience: 0,
            max_level: 100,
            governing_attribute: "Strength".to_string(),
            description: "Proficiency with melee weapons".to_string(),
        });
        
        skills.insert("Ranged Combat".to_string(), SkillInfo {
            level: 1,
            experience: 0,
            max_level: 100,
            governing_attribute: "Dexterity".to_string(),
            description: "Proficiency with ranged weapons".to_string(),
        });
        
        skills.insert("Defense".to_string(), SkillInfo {
            level: 1,
            experience: 0,
            max_level: 100,
            governing_attribute: "Constitution".to_string(),
            description: "Ability to avoid and mitigate damage".to_string(),
        });
        
        // Magic skills
        skills.insert("Arcane Magic".to_string(), SkillInfo {
            level: 1,
            experience: 0,
            max_level: 100,
            governing_attribute: "Intelligence".to_string(),
            description: "Knowledge of arcane spells and rituals".to_string(),
        });
        
        skills.insert("Divine Magic".to_string(), SkillInfo {
            level: 1,
            experience: 0,
            max_level: 100,
            governing_attribute: "Wisdom".to_string(),
            description: "Connection to divine powers".to_string(),
        });
        
        // Utility skills
        skills.insert("Lockpicking".to_string(), SkillInfo {
            level: 1,
            experience: 0,
            max_level: 100,
            governing_attribute: "Dexterity".to_string(),
            description: "Ability to pick locks and disable traps".to_string(),
        });
        
        skills.insert("Stealth".to_string(), SkillInfo {
            level: 1,
            experience: 0,
            max_level: 100,
            governing_attribute: "Dexterity".to_string(),
            description: "Ability to move unseen and unheard".to_string(),
        });
        
        skills.insert("Persuasion".to_string(), SkillInfo {
            level: 1,
            experience: 0,
            max_level: 100,
            governing_attribute: "Charisma".to_string(),
            description: "Ability to influence others through speech".to_string(),
        });

        CharacterSkills { skills }
    }

    pub fn get_skill_bonus(&self, skill_name: &str) -> i32 {
        if let Some(skill) = self.skills.get(skill_name) {
            skill.level / 5 // Every 5 levels gives +1 bonus
        } else {
            0
        }
    }

    pub fn add_experience(&mut self, skill_name: &str, exp: i32) -> bool {
        if let Some(skill) = self.skills.get_mut(skill_name) {
            skill.experience += exp;
            let exp_needed = self.experience_for_level(skill.level + 1);
            
            if skill.experience >= exp_needed && skill.level < skill.max_level {
                skill.level += 1;
                skill.experience -= exp_needed;
                return true; // Level up occurred
            }
        }
        false
    }

    fn experience_for_level(&self, level: i32) -> i32 {
        // Exponential experience curve
        (level * level * 100) + (level * 50)
    }
}

/// Character abilities system
#[derive(Debug, Clone)]
pub struct CharacterAbilities {
    pub abilities: HashMap<String, AbilityInfo>,
    pub ability_points: i32,
}

#[derive(Debug, Clone)]
pub struct AbilityInfo {
    pub name: String,
    pub description: String,
    pub level: i32,
    pub max_level: i32,
    pub cost: i32,
    pub prerequisites: Vec<String>,
    pub ability_type: AbilityType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AbilityType {
    Passive,
    Active,
    Toggle,
}

impl CharacterAbilities {
    pub fn new() -> Self {
        let mut abilities = HashMap::new();
        
        // Combat abilities
        abilities.insert("Power Attack".to_string(), AbilityInfo {
            name: "Power Attack".to_string(),
            description: "Deal extra damage at the cost of accuracy".to_string(),
            level: 0,
            max_level: 5,
            cost: 1,
            prerequisites: vec!["Melee Combat".to_string()],
            ability_type: AbilityType::Active,
        });
        
        abilities.insert("Precise Strike".to_string(), AbilityInfo {
            name: "Precise Strike".to_string(),
            description: "Increased critical hit chance".to_string(),
            level: 0,
            max_level: 3,
            cost: 1,
            prerequisites: vec!["Ranged Combat".to_string()],
            ability_type: AbilityType::Passive,
        });
        
        abilities.insert("Toughness".to_string(), AbilityInfo {
            name: "Toughness".to_string(),
            description: "Increased health and damage resistance".to_string(),
            level: 0,
            max_level: 5,
            cost: 1,
            prerequisites: vec!["Defense".to_string()],
            ability_type: AbilityType::Passive,
        });
        
        // Magic abilities
        abilities.insert("Mana Efficiency".to_string(), AbilityInfo {
            name: "Mana Efficiency".to_string(),
            description: "Reduced mana cost for spells".to_string(),
            level: 0,
            max_level: 3,
            cost: 1,
            prerequisites: vec!["Arcane Magic".to_string()],
            ability_type: AbilityType::Passive,
        });
        
        abilities.insert("Healing Light".to_string(), AbilityInfo {
            name: "Healing Light".to_string(),
            description: "Restore health to self or allies".to_string(),
            level: 0,
            max_level: 5,
            cost: 2,
            prerequisites: vec!["Divine Magic".to_string()],
            ability_type: AbilityType::Active,
        });
        
        // Utility abilities
        abilities.insert("Nimble Fingers".to_string(), AbilityInfo {
            name: "Nimble Fingers".to_string(),
            description: "Improved lockpicking and trap disarming".to_string(),
            level: 0,
            max_level: 3,
            cost: 1,
            prerequisites: vec!["Lockpicking".to_string()],
            ability_type: AbilityType::Passive,
        });
        
        abilities.insert("Shadow Step".to_string(), AbilityInfo {
            name: "Shadow Step".to_string(),
            description: "Teleport short distances while stealthed".to_string(),
            level: 0,
            max_level: 3,
            cost: 2,
            prerequisites: vec!["Stealth".to_string()],
            ability_type: AbilityType::Active,
        });

        CharacterAbilities {
            abilities,
            ability_points: 0,
        }
    }

    pub fn can_learn_ability(&self, ability_name: &str, skills: &CharacterSkills) -> bool {
        if let Some(ability) = self.abilities.get(ability_name) {
            // Check if already at max level
            if ability.level >= ability.max_level {
                return false;
            }
            
            // Check if we have enough ability points
            if self.ability_points < ability.cost {
                return false;
            }
            
            // Check prerequisites
            for prereq in &ability.prerequisites {
                if let Some(skill) = skills.skills.get(prereq) {
                    if skill.level < 10 { // Minimum skill level required
                        return false;
                    }
                } else {
                    return false;
                }
            }
            
            true
        } else {
            false
        }
    }

    pub fn learn_ability(&mut self, ability_name: &str) -> bool {
        if let Some(ability) = self.abilities.get_mut(ability_name) {
            if self.ability_points >= ability.cost && ability.level < ability.max_level {
                self.ability_points -= ability.cost;
                ability.level += 1;
                return true;
            }
        }
        false
    }
}

/// Character progression tracking
#[derive(Debug, Clone)]
pub struct CharacterProgression {
    pub level: i32,
    pub experience: i32,
    pub experience_to_next: i32,
    pub total_experience: i32,
    pub attribute_points: i32,
    pub skill_points: i32,
    pub ability_points: i32,
    pub achievements: Vec<Achievement>,
}

#[derive(Debug, Clone)]
pub struct Achievement {
    pub name: String,
    pub description: String,
    pub unlocked: bool,
    pub unlock_date: Option<String>,
}

impl CharacterProgression {
    pub fn new() -> Self {
        CharacterProgression {
            level: 1,
            experience: 0,
            experience_to_next: 1000,
            total_experience: 0,
            attribute_points: 0,
            skill_points: 0,
            ability_points: 0,
            achievements: Vec::new(),
        }
    }

    pub fn add_experience(&mut self, exp: i32) -> bool {
        self.experience += exp;
        self.total_experience += exp;
        
        if self.experience >= self.experience_to_next {
            self.level_up();
            return true;
        }
        false
    }

    fn level_up(&mut self) {
        self.experience -= self.experience_to_next;
        self.level += 1;
        self.experience_to_next = self.calculate_experience_for_level(self.level + 1);
        
        // Award points for leveling up
        self.attribute_points += 2;
        self.skill_points += 3;
        self.ability_points += 1;
    }

    fn calculate_experience_for_level(&self, level: i32) -> i32 {
        // Exponential experience curve
        (level * level * 100) + (level * 200)
    }

    pub fn experience_percentage(&self) -> f32 {
        if self.experience_to_next > 0 {
            (self.experience as f32 / self.experience_to_next as f32) * 100.0
        } else {
            100.0
        }
    }
}

/// Main character screen component
pub struct CharacterScreen {
    pub state: CharacterScreenState,
    pub player_entity: Option<Entity>,
    pub selected_tab: usize,
    pub selected_item: usize,
    pub scroll_offset: usize,
    pub attributes: CharacterAttributes,
    pub skills: CharacterSkills,
    pub abilities: CharacterAbilities,
    pub progression: CharacterProgression,
    pub pending_attribute_points: HashMap<String, i32>,
    pub show_tooltips: bool,
}

impl CharacterScreen {
    pub fn new() -> Self {
        CharacterScreen {
            state: CharacterScreenState::Closed,
            player_entity: None,
            selected_tab: 0,
            selected_item: 0,
            scroll_offset: 0,
            attributes: CharacterAttributes::new(),
            skills: CharacterSkills::new(),
            abilities: CharacterAbilities::new(),
            progression: CharacterProgression::new(),
            pending_attribute_points: HashMap::new(),
            show_tooltips: true,
        }
    }

    pub fn open(&mut self, player_entity: Entity) {
        self.player_entity = Some(player_entity);
        self.state = CharacterScreenState::Overview;
        self.selected_tab = 0;
        self.selected_item = 0;
        self.scroll_offset = 0;
    }

    pub fn close(&mut self) {
        self.state = CharacterScreenState::Closed;
    }

    pub fn is_open(&self) -> bool {
        self.state != CharacterScreenState::Closed
    }

    pub fn update_from_world(&mut self, world: &World) {
        if let Some(player_entity) = self.player_entity {
            // Update attributes from equipment bonuses
            self.update_attribute_bonuses(world, player_entity);
            
            // Update combat stats
            self.update_combat_stats(world, player_entity);
        }
    }

    fn update_attribute_bonuses(&mut self, world: &World, player_entity: Entity) {
        // Clear existing bonuses
        self.attributes.bonuses.clear();
        
        let equipment = world.read_storage::<Equipment>();
        let item_bonuses = world.read_storage::<ItemBonuses>();
        
        if let Some(equip) = equipment.get(player_entity) {
            for item_entity in equip.get_all_equipped_items() {
                if let Some(bonuses) = item_bonuses.get(item_entity) {
                    for (attr, bonus) in &bonuses.attribute_bonuses {
                        self.attributes.add_bonus(attr.clone(), *bonus);
                    }
                }
            }
        }
    }

    fn update_combat_stats(&mut self, world: &World, player_entity: Entity) {
        let combat_stats = world.read_storage::<CombatStats>();
        
        if let Some(_stats) = combat_stats.get(player_entity) {
            // Update derived stats based on attributes
            // This would be where we calculate how attributes affect combat stats
        }
    }

    pub fn handle_key(&mut self, key: KeyCode, world: &World) -> bool {
        match key {
            KeyCode::Tab => {
                self.next_tab();
                true
            }
            KeyCode::BackTab => {
                self.previous_tab();
                true
            }
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
                self.activate_selected(world);
                true
            }
            KeyCode::Char('+') | KeyCode::Char('=') => {
                self.increase_selected();
                true
            }
            KeyCode::Char('-') => {
                self.decrease_selected();
                true
            }
            KeyCode::Esc | KeyCode::Char('q') => {
                self.close();
                true
            }
            _ => false,
        }
    }

    fn next_tab(&mut self) {
        let tab_count = 5; // Overview, Attributes, Skills, Abilities, Progression
        self.selected_tab = (self.selected_tab + 1) % tab_count;
        self.update_state_from_tab();
        self.selected_item = 0;
        self.scroll_offset = 0;
    }

    fn previous_tab(&mut self) {
        let tab_count = 5;
        self.selected_tab = if self.selected_tab == 0 {
            tab_count - 1
        } else {
            self.selected_tab - 1
        };
        self.update_state_from_tab();
        self.selected_item = 0;
        self.scroll_offset = 0;
    }

    fn update_state_from_tab(&mut self) {
        self.state = match self.selected_tab {
            0 => CharacterScreenState::Overview,
            1 => CharacterScreenState::Attributes,
            2 => CharacterScreenState::Skills,
            3 => CharacterScreenState::Abilities,
            4 => CharacterScreenState::Progression,
            _ => CharacterScreenState::Overview,
        };
    }

    fn navigate_up(&mut self) {
        if self.selected_item > 0 {
            self.selected_item -= 1;
            self.ensure_item_visible();
        }
    }

    fn navigate_down(&mut self) {
        let max_items = self.get_max_items_for_current_state();
        if self.selected_item < max_items.saturating_sub(1) {
            self.selected_item += 1;
            self.ensure_item_visible();
        }
    }

    fn navigate_left(&mut self) {
        match self.state {
            CharacterScreenState::StatAllocation => {
                self.decrease_selected();
            }
            _ => {}
        }
    }

    fn navigate_right(&mut self) {
        match self.state {
            CharacterScreenState::StatAllocation => {
                self.increase_selected();
            }
            _ => {}
        }
    }

    fn activate_selected(&mut self, world: &World) {
        match self.state {
            CharacterScreenState::Attributes => {
                if self.progression.attribute_points > 0 {
                    self.state = CharacterScreenState::StatAllocation;
                }
            }
            CharacterScreenState::Abilities => {
                let ability_names: Vec<String> = self.abilities.abilities.keys().cloned().collect();
                if let Some(ability_name) = ability_names.get(self.selected_item) {
                    if self.abilities.can_learn_ability(ability_name, &self.skills) {
                        self.abilities.learn_ability(ability_name);
                    }
                }
            }
            CharacterScreenState::StatAllocation => {
                self.apply_pending_attribute_points();
                self.state = CharacterScreenState::Attributes;
            }
            _ => {}
        }
    }

    fn increase_selected(&mut self) {
        match self.state {
            CharacterScreenState::StatAllocation => {
                let attribute_names = vec!["Strength", "Dexterity", "Constitution", "Intelligence", "Wisdom", "Charisma"];
                if let Some(attr_name) = attribute_names.get(self.selected_item) {
                    if self.progression.attribute_points > 0 {
                        *self.pending_attribute_points.entry(attr_name.to_string()).or_insert(0) += 1;
                        self.progression.attribute_points -= 1;
                    }
                }
            }
            _ => {}
        }
    }

    fn decrease_selected(&mut self) {
        match self.state {
            CharacterScreenState::StatAllocation => {
                let attribute_names = vec!["Strength", "Dexterity", "Constitution", "Intelligence", "Wisdom", "Charisma"];
                if let Some(attr_name) = attribute_names.get(self.selected_item) {
                    if let Some(pending) = self.pending_attribute_points.get_mut(&attr_name.to_string()) {
                        if *pending > 0 {
                            *pending -= 1;
                            self.progression.attribute_points += 1;
                            if *pending == 0 {
                                self.pending_attribute_points.remove(&attr_name.to_string());
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn apply_pending_attribute_points(&mut self) {
        for (attr_name, points) in &self.pending_attribute_points {
            let current_base = self.attributes.base_values.get(attr_name).unwrap_or(&10);
            self.attributes.set_base(attr_name.clone(), current_base + points);
        }
        self.pending_attribute_points.clear();
    }

    fn get_max_items_for_current_state(&self) -> usize {
        match self.state {
            CharacterScreenState::Attributes | CharacterScreenState::StatAllocation => 6, // 6 attributes
            CharacterScreenState::Skills => self.skills.skills.len(),
            CharacterScreenState::Abilities => self.abilities.abilities.len(),
            _ => 0,
        }
    }

    fn ensure_item_visible(&mut self) {
        let items_per_page = 15; // Adjust based on screen size
        
        if self.selected_item < self.scroll_offset {
            self.scroll_offset = self.selected_item;
        } else if self.selected_item >= self.scroll_offset + items_per_page {
            self.scroll_offset = self.selected_item - items_per_page + 1;
        }
    }

    pub fn render(&self, world: &World, screen_width: i32, screen_height: i32) -> Vec<UIRenderCommand> {
        let mut commands = Vec::new();

        // Main character screen panel
        let panel_width = screen_width - 4;
        let panel_height = screen_height - 4;
        let panel = UIPanel::new(
            "Character".to_string(),
            2,
            2,
            panel_width,
            panel_height,
        ).with_colors(Color::White, Color::Black, Color::Yellow);

        commands.extend(panel.render());

        // Render tabs
        commands.extend(self.render_tabs(screen_width));

        // Render content based on current state
        match self.state {
            CharacterScreenState::Overview => commands.extend(self.render_overview(world, screen_width, screen_height)),
            CharacterScreenState::Attributes => commands.extend(self.render_attributes(screen_width, screen_height)),
            CharacterScreenState::Skills => commands.extend(self.render_skills(screen_width, screen_height)),
            CharacterScreenState::Abilities => commands.extend(self.render_abilities(screen_width, screen_height)),
            CharacterScreenState::Progression => commands.extend(self.render_progression(screen_width, screen_height)),
            CharacterScreenState::StatAllocation => commands.extend(self.render_stat_allocation(screen_width, screen_height)),
            CharacterScreenState::Closed => {}
        }

        // Render controls help
        commands.extend(self.render_controls_help(screen_width, screen_height));

        commands
    }

    fn render_tabs(&self, screen_width: i32) -> Vec<UIRenderCommand> {
        let mut commands = Vec::new();
        let tabs = vec!["Overview", "Attributes", "Skills", "Abilities", "Progression"];
        let tab_width = 12;
        let start_x = 4;
        let tab_y = 4;

        for (i, tab_name) in tabs.iter().enumerate() {
            let x = start_x + (i as i32 * tab_width);
            let is_selected = i == self.selected_tab;

            let (fg, bg) = if is_selected {
                (Color::Black, Color::Yellow)
            } else {
                (Color::White, Color::DarkGrey)
            };

            commands.push(UIRenderCommand::DrawText {
                x,
                y: tab_y,
                text: format!("{:^width$}", tab_name, width = tab_width as usize),
                fg,
                bg,
            });
        }

        commands
    }

    fn render_overview(&self, world: &World, screen_width: i32, screen_height: i32) -> Vec<UIRenderCommand> {
        let mut commands = Vec::new();
        let start_y = 6;
        let mut y = start_y;

        if let Some(player_entity) = self.player_entity {
            let names = world.read_storage::<Name>();
            let combat_stats = world.read_storage::<CombatStats>();

            // Character name and level
            if let Some(name) = names.get(player_entity) {
                commands.push(UIRenderCommand::DrawText {
                    x: 4,
                    y,
                    text: format!("{} - Level {}", name.name, self.progression.level),
                    fg: Color::Yellow,
                    bg: Color::Black,
                });
                y += 2;
            }

            // Experience bar
            let exp_percentage = self.progression.experience_percentage();
            let exp_bar_width = 30;
            let filled_width = (exp_percentage / 100.0 * exp_bar_width as f32) as usize;
            let exp_bar = format!("{}{}",
                "█".repeat(filled_width),
                "░".repeat(exp_bar_width - filled_width)
            );

            commands.push(UIRenderCommand::DrawText {
                x: 4,
                y,
                text: format!("Experience: {} / {} ({}%)", 
                    self.progression.experience, 
                    self.progression.experience_to_next,
                    exp_percentage as i32),
                fg: Color::Cyan,
                bg: Color::Black,
            });
            y += 1;

            commands.push(UIRenderCommand::DrawText {
                x: 4,
                y,
                text: exp_bar,
                fg: Color::Green,
                bg: Color::Black,
            });
            y += 2;

            // Combat stats
            if let Some(stats) = combat_stats.get(player_entity) {
                commands.push(UIRenderCommand::DrawText {
                    x: 4,
                    y,
                    text: "Combat Statistics:".to_string(),
                    fg: Color::White,
                    bg: Color::Black,
                });
                y += 1;

                let health_percentage = (stats.hp as f32 / stats.max_hp as f32 * 100.0) as i32;
                commands.push(UIRenderCommand::DrawText {
                    x: 6,
                    y,
                    text: format!("Health: {} / {} ({}%)", stats.hp, stats.max_hp, health_percentage),
                    fg: if health_percentage > 75 { Color::Green } else if health_percentage > 25 { Color::Yellow } else { Color::Red },
                    bg: Color::Black,
                });
                y += 1;

                commands.push(UIRenderCommand::DrawText {
                    x: 6,
                    y,
                    text: format!("Attack Power: {}", stats.power),
                    fg: Color::Red,
                    bg: Color::Black,
                });
                y += 1;

                commands.push(UIRenderCommand::DrawText {
                    x: 6,
                    y,
                    text: format!("Defense: {}", stats.defense),
                    fg: Color::Blue,
                    bg: Color::Black,
                });
                y += 2;
            }

            // Available points
            commands.push(UIRenderCommand::DrawText {
                x: 4,
                y,
                text: "Available Points:".to_string(),
                fg: Color::White,
                bg: Color::Black,
            });
            y += 1;

            commands.push(UIRenderCommand::DrawText {
                x: 6,
                y,
                text: format!("Attribute Points: {}", self.progression.attribute_points),
                fg: if self.progression.attribute_points > 0 { Color::Green } else { Color::Grey },
                bg: Color::Black,
            });
            y += 1;

            commands.push(UIRenderCommand::DrawText {
                x: 6,
                y,
                text: format!("Skill Points: {}", self.progression.skill_points),
                fg: if self.progression.skill_points > 0 { Color::Green } else { Color::Grey },
                bg: Color::Black,
            });
            y += 1;

            commands.push(UIRenderCommand::DrawText {
                x: 6,
                y,
                text: format!("Ability Points: {}", self.progression.ability_points),
                fg: if self.progression.ability_points > 0 { Color::Green } else { Color::Grey },
                bg: Color::Black,
            });
        }

        commands
    }

    fn render_attributes(&self, screen_width: i32, screen_height: i32) -> Vec<UIRenderCommand> {
        let mut commands = Vec::new();
        let start_y = 6;
        let mut y = start_y;

        commands.push(UIRenderCommand::DrawText {
            x: 4,
            y,
            text: "Character Attributes:".to_string(),
            fg: Color::Yellow,
            bg: Color::Black,
        });
        y += 2;

        let attributes = vec![
            ("Strength", "Physical power and melee damage"),
            ("Dexterity", "Agility, accuracy, and ranged damage"),
            ("Constitution", "Health, stamina, and damage resistance"),
            ("Intelligence", "Magical power and mana capacity"),
            ("Wisdom", "Magical resistance and divine magic"),
            ("Charisma", "Social skills and leadership"),
        ];

        for (i, (attr_name, description)) in attributes.iter().enumerate() {
            let is_selected = i == self.selected_item && self.state == CharacterScreenState::Attributes;
            let total = self.attributes.get_total(attr_name);
            let modifier = self.attributes.get_modifier(attr_name);
            let base = self.attributes.base_values.get(&attr_name.to_string()).unwrap_or(&10);
            let bonus = self.attributes.bonuses.get(&attr_name.to_string()).unwrap_or(&0);

            let (fg, bg) = if is_selected {
                (Color::Black, Color::White)
            } else {
                (Color::White, Color::Black)
            };

            let modifier_text = if modifier >= 0 {
                format!("+{}", modifier)
            } else {
                format!("{}", modifier)
            };

            let attr_text = if *bonus != 0 {
                format!("{:<12} {:2} ({} + {}) [{}]", attr_name, total, base, bonus, modifier_text)
            } else {
                format!("{:<12} {:2} [{}]", attr_name, total, modifier_text)
            };

            commands.push(UIRenderCommand::DrawText {
                x: 4,
                y,
                text: format!("{:<width$}", attr_text, width = 40),
                fg,
                bg,
            });

            if is_selected && self.show_tooltips {
                commands.push(UIRenderCommand::DrawText {
                    x: 46,
                    y,
                    text: description.to_string(),
                    fg: Color::Cyan,
                    bg: Color::Black,
                });
            }

            y += 1;
        }

        if self.progression.attribute_points > 0 {
            y += 1;
            commands.push(UIRenderCommand::DrawText {
                x: 4,
                y,
                text: format!("Available Attribute Points: {} (Press ENTER to allocate)", self.progression.attribute_points),
                fg: Color::Green,
                bg: Color::Black,
            });
        }

        commands
    }

    fn render_skills(&self, screen_width: i32, screen_height: i32) -> Vec<UIRenderCommand> {
        let mut commands = Vec::new();
        let start_y = 6;
        let mut y = start_y;

        commands.push(UIRenderCommand::DrawText {
            x: 4,
            y,
            text: "Character Skills:".to_string(),
            fg: Color::Yellow,
            bg: Color::Black,
        });
        y += 2;

        let skill_names: Vec<String> = self.skills.skills.keys().cloned().collect();
        let visible_skills = skill_names.iter()
            .skip(self.scroll_offset)
            .take(15);

        for (i, skill_name) in visible_skills.enumerate() {
            let actual_index = i + self.scroll_offset;
            let is_selected = actual_index == self.selected_item && self.state == CharacterScreenState::Skills;
            
            if let Some(skill) = self.skills.skills.get(skill_name) {
                let (fg, bg) = if is_selected {
                    (Color::Black, Color::White)
                } else {
                    (Color::White, Color::Black)
                };

                let progress_percentage = if skill.level < skill.max_level {
                    let exp_needed = self.skills.experience_for_level(skill.level + 1);
                    (skill.experience as f32 / exp_needed as f32 * 100.0) as i32
                } else {
                    100
                };

                let skill_text = format!("{:<15} Lv.{:2} ({}%)", skill_name, skill.level, progress_percentage);

                commands.push(UIRenderCommand::DrawText {
                    x: 4,
                    y,
                    text: format!("{:<width$}", skill_text, width = 35),
                    fg,
                    bg,
                });

                if is_selected && self.show_tooltips {
                    commands.push(UIRenderCommand::DrawText {
                        x: 42,
                        y,
                        text: skill.description.clone(),
                        fg: Color::Cyan,
                        bg: Color::Black,
                    });
                }

                y += 1;
            }
        }

        commands
    }

    fn render_abilities(&self, screen_width: i32, screen_height: i32) -> Vec<UIRenderCommand> {
        let mut commands = Vec::new();
        let start_y = 6;
        let mut y = start_y;

        commands.push(UIRenderCommand::DrawText {
            x: 4,
            y,
            text: format!("Character Abilities (Points: {}):", self.abilities.ability_points),
            fg: Color::Yellow,
            bg: Color::Black,
        });
        y += 2;

        let ability_names: Vec<String> = self.abilities.abilities.keys().cloned().collect();
        let visible_abilities = ability_names.iter()
            .skip(self.scroll_offset)
            .take(15);

        for (i, ability_name) in visible_abilities.enumerate() {
            let actual_index = i + self.scroll_offset;
            let is_selected = actual_index == self.selected_item && self.state == CharacterScreenState::Abilities;
            
            if let Some(ability) = self.abilities.abilities.get(ability_name) {
                let can_learn = self.abilities.can_learn_ability(ability_name, &self.skills);
                
                let (fg, bg) = if is_selected {
                    (Color::Black, Color::White)
                } else if ability.level > 0 {
                    (Color::Green, Color::Black)
                } else if can_learn {
                    (Color::Yellow, Color::Black)
                } else {
                    (Color::DarkGrey, Color::Black)
                };

                let ability_text = if ability.level > 0 {
                    format!("{:<18} {}/{} [{}]", ability_name, ability.level, ability.max_level, 
                        match ability.ability_type {
                            AbilityType::Passive => "Passive",
                            AbilityType::Active => "Active",
                            AbilityType::Toggle => "Toggle",
                        })
                } else {
                    format!("{:<18} Not Learned (Cost: {})", ability_name, ability.cost)
                };

                commands.push(UIRenderCommand::DrawText {
                    x: 4,
                    y,
                    text: format!("{:<width$}", ability_text, width = 45),
                    fg,
                    bg,
                });

                if is_selected && self.show_tooltips {
                    commands.push(UIRenderCommand::DrawText {
                        x: 52,
                        y,
                        text: ability.description.clone(),
                        fg: Color::Cyan,
                        bg: Color::Black,
                    });
                }

                y += 1;
            }
        }

        commands
    }

    fn render_progression(&self, screen_width: i32, screen_height: i32) -> Vec<UIRenderCommand> {
        let mut commands = Vec::new();
        let start_y = 6;
        let mut y = start_y;

        commands.push(UIRenderCommand::DrawText {
            x: 4,
            y,
            text: "Character Progression:".to_string(),
            fg: Color::Yellow,
            bg: Color::Black,
        });
        y += 2;

        // Level and experience
        commands.push(UIRenderCommand::DrawText {
            x: 4,
            y,
            text: format!("Level: {}", self.progression.level),
            fg: Color::White,
            bg: Color::Black,
        });
        y += 1;

        commands.push(UIRenderCommand::DrawText {
            x: 4,
            y,
            text: format!("Experience: {} / {}", self.progression.experience, self.progression.experience_to_next),
            fg: Color::Cyan,
            bg: Color::Black,
        });
        y += 1;

        commands.push(UIRenderCommand::DrawText {
            x: 4,
            y,
            text: format!("Total Experience: {}", self.progression.total_experience),
            fg: Color::Cyan,
            bg: Color::Black,
        });
        y += 2;

        // Experience bar
        let exp_percentage = self.progression.experience_percentage();
        let exp_bar_width = 40;
        let filled_width = (exp_percentage / 100.0 * exp_bar_width as f32) as usize;
        let exp_bar = format!("{}{}",
            "█".repeat(filled_width),
            "░".repeat(exp_bar_width - filled_width)
        );

        commands.push(UIRenderCommand::DrawText {
            x: 4,
            y,
            text: format!("Progress to next level: {:.1}%", exp_percentage),
            fg: Color::Green,
            bg: Color::Black,
        });
        y += 1;

        commands.push(UIRenderCommand::DrawText {
            x: 4,
            y,
            text: exp_bar,
            fg: Color::Green,
            bg: Color::Black,
        });
        y += 2;

        // Available points summary
        commands.push(UIRenderCommand::DrawText {
            x: 4,
            y,
            text: "Available Points:".to_string(),
            fg: Color::White,
            bg: Color::Black,
        });
        y += 1;

        commands.push(UIRenderCommand::DrawText {
            x: 6,
            y,
            text: format!("Attribute Points: {}", self.progression.attribute_points),
            fg: if self.progression.attribute_points > 0 { Color::Green } else { Color::Grey },
            bg: Color::Black,
        });
        y += 1;

        commands.push(UIRenderCommand::DrawText {
            x: 6,
            y,
            text: format!("Skill Points: {}", self.progression.skill_points),
            fg: if self.progression.skill_points > 0 { Color::Green } else { Color::Grey },
            bg: Color::Black,
        });
        y += 1;

        commands.push(UIRenderCommand::DrawText {
            x: 6,
            y,
            text: format!("Ability Points: {}", self.progression.ability_points),
            fg: if self.progression.ability_points > 0 { Color::Green } else { Color::Grey },
            bg: Color::Black,
        });

        commands
    }

    fn render_stat_allocation(&self, screen_width: i32, screen_height: i32) -> Vec<UIRenderCommand> {
        let mut commands = Vec::new();
        let start_y = 6;
        let mut y = start_y;

        commands.push(UIRenderCommand::DrawText {
            x: 4,
            y,
            text: format!("Allocate Attribute Points (Available: {}):", self.progression.attribute_points),
            fg: Color::Yellow,
            bg: Color::Black,
        });
        y += 2;

        commands.push(UIRenderCommand::DrawText {
            x: 4,
            y,
            text: "Use +/- or Left/Right arrows to allocate points, ENTER to confirm".to_string(),
            fg: Color::Cyan,
            bg: Color::Black,
        });
        y += 2;

        let attributes = vec!["Strength", "Dexterity", "Constitution", "Intelligence", "Wisdom", "Charisma"];

        for (i, attr_name) in attributes.iter().enumerate() {
            let is_selected = i == self.selected_item;
            let current_base = self.attributes.base_values.get(&attr_name.to_string()).unwrap_or(&10);
            let pending = self.pending_attribute_points.get(&attr_name.to_string()).unwrap_or(&0);
            let new_total = current_base + pending;

            let (fg, bg) = if is_selected {
                (Color::Black, Color::White)
            } else {
                (Color::White, Color::Black)
            };

            let attr_text = if *pending > 0 {
                format!("{:<12} {} -> {} (+{})", attr_name, current_base, new_total, pending)
            } else {
                format!("{:<12} {}", attr_name, current_base)
            };

            commands.push(UIRenderCommand::DrawText {
                x: 4,
                y,
                text: format!("{:<width$}", attr_text, width = 35),
                fg,
                bg,
            });

            y += 1;
        }

        y += 1;
        let total_pending: i32 = self.pending_attribute_points.values().sum();
        if total_pending > 0 {
            commands.push(UIRenderCommand::DrawText {
                x: 4,
                y,
                text: format!("Points to allocate: {}", total_pending),
                fg: Color::Green,
                bg: Color::Black,
            });
        }

        commands
    }

    fn render_controls_help(&self, screen_width: i32, screen_height: i32) -> Vec<UIRenderCommand> {
        let mut commands = Vec::new();
        let controls_y = screen_height - 3;

        let controls_text = match self.state {
            CharacterScreenState::StatAllocation => "TAB:Switch +/-:Allocate ENTER:Confirm ESC:Cancel",
            _ => "TAB:Switch Tabs ↑↓:Navigate ENTER:Select ESC:Close",
        };

        commands.push(UIRenderCommand::DrawText {
            x: 4,
            y: controls_y,
            text: controls_text.to_string(),
            fg: Color::DarkGrey,
            bg: Color::Black,
        });

        commands
    }
}

impl UIComponent for CharacterScreen {
    fn render(&self, _x: i32, _y: i32, width: i32, height: i32) -> Vec<UIRenderCommand> {
        // This method signature doesn't provide access to World, so we use the other render method
        vec![]
    }

    fn handle_input(&mut self, input: char) -> bool {
        // Convert char to KeyCode for consistency
        let key = match input {
            '\n' => KeyCode::Enter,
            '\x1b' => KeyCode::Esc,
            '\t' => KeyCode::Tab,
            'k' | 'w' => KeyCode::Up,
            'j' | 's' => KeyCode::Down,
            'h' | 'a' => KeyCode::Left,
            'l' | 'd' => KeyCode::Right,
            '+' | '=' => KeyCode::Char('+'),
            '-' => KeyCode::Char('-'),
            c => KeyCode::Char(c),
        };

        // We need World access for proper handling, so this is a simplified version
        match key {
            KeyCode::Esc | KeyCode::Char('q') => {
                self.close();
                true
            }
            KeyCode::Tab => {
                self.next_tab();
                true
            }
            _ => false,
        }
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
    use specs::{World, WorldExt, Builder};
    use crate::components::{Player, Name, CombatStats};

    fn setup_test_world() -> (World, Entity) {
        let mut world = World::new();
        world.register::<Player>();
        world.register::<Name>();
        world.register::<CombatStats>();
        world.register::<Equipment>();
        world.register::<ItemBonuses>();

        let player = world.create_entity()
            .with(Player)
            .with(Name { name: "Hero".to_string() })
            .with(CombatStats { max_hp: 100, hp: 100, defense: 10, power: 15 })
            .build();

        (world, player)
    }

    #[test]
    fn test_character_screen_creation() {
        let screen = CharacterScreen::new();
        
        assert_eq!(screen.state, CharacterScreenState::Closed);
        assert!(screen.player_entity.is_none());
        assert_eq!(screen.selected_tab, 0);
        assert_eq!(screen.progression.level, 1);
    }

    #[test]
    fn test_character_screen_open_close() {
        let (world, player) = setup_test_world();
        let mut screen = CharacterScreen::new();
        
        assert!(!screen.is_open());
        
        screen.open(player);
        assert!(screen.is_open());
        assert_eq!(screen.player_entity, Some(player));
        assert_eq!(screen.state, CharacterScreenState::Overview);
        
        screen.close();
        assert!(!screen.is_open());
        assert_eq!(screen.state, CharacterScreenState::Closed);
    }

    #[test]
    fn test_character_attributes() {
        let mut attributes = CharacterAttributes::new();
        
        assert_eq!(attributes.get_total("Strength"), 10);
        assert_eq!(attributes.get_modifier("Strength"), 0);
        
        attributes.add_bonus("Strength".to_string(), 4);
        assert_eq!(attributes.get_total("Strength"), 14);
        assert_eq!(attributes.get_modifier("Strength"), 2);
        
        attributes.set_base("Strength".to_string(), 16);
        assert_eq!(attributes.get_total("Strength"), 20);
        assert_eq!(attributes.get_modifier("Strength"), 5);
    }

    #[test]
    fn test_character_skills() {
        let mut skills = CharacterSkills::new();
        
        assert_eq!(skills.get_skill_bonus("Melee Combat"), 0);
        
        // Add experience and check for level up
        let leveled_up = skills.add_experience("Melee Combat", 1000);
        assert!(leveled_up);
        
        if let Some(skill) = skills.skills.get("Melee Combat") {
            assert_eq!(skill.level, 2);
        }
        
        assert_eq!(skills.get_skill_bonus("Melee Combat"), 0); // Still 0 until level 5
    }

    #[test]
    fn test_character_abilities() {
        let mut abilities = CharacterAbilities::new();
        let skills = CharacterSkills::new();
        
        // Can't learn ability without meeting prerequisites
        assert!(!abilities.can_learn_ability("Power Attack", &skills));
        
        // Give ability points
        abilities.ability_points = 5;
        
        // Still can't learn without skill prerequisites
        assert!(!abilities.can_learn_ability("Power Attack", &skills));
    }

    #[test]
    fn test_character_progression() {
        let mut progression = CharacterProgression::new();
        
        assert_eq!(progression.level, 1);
        assert_eq!(progression.experience, 0);
        
        // Add experience but not enough to level up
        let leveled_up = progression.add_experience(500);
        assert!(!leveled_up);
        assert_eq!(progression.level, 1);
        
        // Add enough experience to level up
        let leveled_up = progression.add_experience(600);
        assert!(leveled_up);
        assert_eq!(progression.level, 2);
        assert!(progression.attribute_points > 0);
        assert!(progression.skill_points > 0);
        assert!(progression.ability_points > 0);
    }

    #[test]
    fn test_tab_navigation() {
        let mut screen = CharacterScreen::new();
        
        assert_eq!(screen.selected_tab, 0);
        assert_eq!(screen.state, CharacterScreenState::Closed);
        
        screen.state = CharacterScreenState::Overview; // Simulate opening
        
        screen.next_tab();
        assert_eq!(screen.selected_tab, 1);
        assert_eq!(screen.state, CharacterScreenState::Attributes);
        
        screen.previous_tab();
        assert_eq!(screen.selected_tab, 0);
        assert_eq!(screen.state, CharacterScreenState::Overview);
    }

    #[test]
    fn test_stat_allocation() {
        let mut screen = CharacterScreen::new();
        screen.progression.attribute_points = 5;
        screen.state = CharacterScreenState::StatAllocation;
        screen.selected_item = 0; // Strength
        
        // Allocate points
        screen.increase_selected();
        screen.increase_selected();
        
        assert_eq!(screen.progression.attribute_points, 3);
        assert_eq!(*screen.pending_attribute_points.get("Strength").unwrap(), 2);
        
        // Decrease allocation
        screen.decrease_selected();
        
        assert_eq!(screen.progression.attribute_points, 4);
        assert_eq!(*screen.pending_attribute_points.get("Strength").unwrap(), 1);
        
        // Apply changes
        let original_strength = screen.attributes.get_total("Strength");
        screen.apply_pending_attribute_points();
        
        assert_eq!(screen.attributes.get_total("Strength"), original_strength + 1);
        assert!(screen.pending_attribute_points.is_empty());
    }
}