use crossterm::{event::KeyCode, style::Color};
use serde::{Serialize, Deserialize};
use crate::ui::{
    ui_components::{UIComponent, UIRenderCommand, UIPanel, UIButton, UIText, UIList, TextAlignment},
    menu_system::{MenuRenderer, MenuInput},
};

/// Main menu state and options
#[derive(Debug, Clone, PartialEq)]
pub enum MainMenuState {
    MainMenu,
    NewGame,
    LoadGame,
    Options,
    Credits,
    Quit,
}

/// Menu options for the main menu
#[derive(Debug, Clone, PartialEq)]
pub enum MenuOption {
    NewGame,
    LoadGame,
    Options,
    Credits,
    Quit,
}

impl MenuOption {
    pub fn to_string(&self) -> String {
        match self {
            MenuOption::NewGame => "New Game".to_string(),
            MenuOption::LoadGame => "Load Game".to_string(),
            MenuOption::Options => "Options".to_string(),
            MenuOption::Credits => "Credits".to_string(),
            MenuOption::Quit => "Quit".to_string(),
        }
    }

    pub fn all_options() -> Vec<MenuOption> {
        vec![
            MenuOption::NewGame,
            MenuOption::LoadGame,
            MenuOption::Options,
            MenuOption::Credits,
            MenuOption::Quit,
        ]
    }
}

/// Main menu component
pub struct MainMenu {
    pub state: MainMenuState,
    pub selected_option: usize,
    pub options: Vec<MenuOption>,
    pub title: String,
    pub subtitle: String,
    pub version: String,
    pub show_cursor: bool,
    pub animation_frame: usize,
    pub last_key: Option<KeyCode>,
}

impl MainMenu {
    pub fn new() -> Self {
        MainMenu {
            state: MainMenuState::MainMenu,
            selected_option: 0,
            options: MenuOption::all_options(),
            title: "ASCII DUNGEON EXPLORER".to_string(),
            subtitle: "A Roguelike Adventure".to_string(),
            version: "v0.1.0".to_string(),
            show_cursor: true,
            animation_frame: 0,
            last_key: None,
        }
    }

    pub fn with_title(mut self, title: String, subtitle: String) -> Self {
        self.title = title;
        self.subtitle = subtitle;
        self
    }

    pub fn with_version(mut self, version: String) -> Self {
        self.version = version;
        self
    }

    pub fn get_selected_option(&self) -> Option<&MenuOption> {
        self.options.get(self.selected_option)
    }

    pub fn select_next(&mut self) {
        self.selected_option = (self.selected_option + 1) % self.options.len();
    }

    pub fn select_previous(&mut self) {
        self.selected_option = if self.selected_option == 0 {
            self.options.len() - 1
        } else {
            self.selected_option - 1
        };
    }

    pub fn activate_selected(&mut self) -> MainMenuState {
        if let Some(option) = self.get_selected_option() {
            match option {
                MenuOption::NewGame => MainMenuState::NewGame,
                MenuOption::LoadGame => MainMenuState::LoadGame,
                MenuOption::Options => MainMenuState::Options,
                MenuOption::Credits => MainMenuState::Credits,
                MenuOption::Quit => MainMenuState::Quit,
            }
        } else {
            MainMenuState::MainMenu
        }
    }

    pub fn handle_key(&mut self, key: KeyCode) -> bool {
        self.last_key = Some(key);
        
        match key {
            KeyCode::Up | KeyCode::Char('w') | KeyCode::Char('k') => {
                self.select_previous();
                true
            }
            KeyCode::Down | KeyCode::Char('s') | KeyCode::Char('j') => {
                self.select_next();
                true
            }
            KeyCode::Enter | KeyCode::Char(' ') => {
                self.state = self.activate_selected();
                true
            }
            KeyCode::Esc | KeyCode::Char('q') => {
                self.state = MainMenuState::Quit;
                true
            }
            _ => false,
        }
    }

    pub fn update(&mut self) {
        self.animation_frame = (self.animation_frame + 1) % 60;
        self.show_cursor = (self.animation_frame / 30) == 0;
    }

    pub fn reset(&mut self) {
        self.state = MainMenuState::MainMenu;
        self.selected_option = 0;
        self.last_key = None;
    }

    fn render_title(&self, width: i32, height: i32) -> Vec<UIRenderCommand> {
        let mut commands = Vec::new();
        
        // ASCII art title
        let title_art = vec![
            "  ▄▄▄       ██████  ▄████▄   ██▓ ██▓",
            " ▒████▄   ▒██    ▒ ▒██▀ ▀█  ▓██▒▓██▒",
            " ▒██  ▀█▄ ░ ▓██▄   ▒▓█    ▄ ▒██▒▒██▒",
            " ░██▄▄▄▄██  ▒   ██▒▒▓▓▄ ▄██▒░██░░██░",
            "  ▓█   ▓██▒██████▒▒▒ ▓███▀ ░░██░░██░",
            "  ▒▒   ▓▒█▒ ▒▓▒ ▒ ░░ ░▒ ▒  ░░▓  ░▓  ",
            "   ▒   ▒▒ ░ ░▒  ░ ░  ░  ▒    ▒ ░ ▒ ░",
            "   ░   ▒  ░  ░  ░  ░         ▒ ░ ▒ ░",
            "       ░  ░     ░  ░ ░       ░   ░  ",
            "                  ░              ",
        ];

        let title_start_y = 2;
        for (i, line) in title_art.iter().enumerate() {
            let x = (width - line.len() as i32) / 2;
            let y = title_start_y + i as i32;
            
            commands.push(UIRenderCommand::DrawText {
                x,
                y,
                text: line.to_string(),
                fg: Color::Yellow,
                bg: Color::Black,
            });
        }

        // Subtitle
        let subtitle_y = title_start_y + title_art.len() as i32 + 1;
        let subtitle_x = (width - self.subtitle.len() as i32) / 2;
        commands.push(UIRenderCommand::DrawText {
            x: subtitle_x,
            y: subtitle_y,
            text: self.subtitle.clone(),
            fg: Color::Cyan,
            bg: Color::Black,
        });

        // Version
        let version_x = width - self.version.len() as i32 - 2;
        let version_y = height - 2;
        commands.push(UIRenderCommand::DrawText {
            x: version_x,
            y: version_y,
            text: self.version.clone(),
            fg: Color::DarkGrey,
            bg: Color::Black,
        });

        commands
    }

    fn render_menu_options(&self, width: i32, height: i32) -> Vec<UIRenderCommand> {
        let mut commands = Vec::new();
        
        let menu_start_y = height / 2;
        let menu_width = 20;
        let menu_x = (width - menu_width) / 2;

        // Draw menu background panel
        let panel = UIPanel::new(
            "".to_string(),
            menu_x - 2,
            menu_start_y - 1,
            menu_width + 4,
            self.options.len() as i32 + 2,
        ).with_colors(Color::DarkGrey, Color::Black, Color::White);
        
        commands.extend(panel.render());

        // Draw menu options
        for (i, option) in self.options.iter().enumerate() {
            let y = menu_start_y + i as i32;
            let is_selected = i == self.selected_option;
            
            let (fg, bg, prefix) = if is_selected {
                (Color::Black, Color::White, "> ")
            } else {
                (Color::White, Color::Black, "  ")
            };

            let option_text = format!("{}{}", prefix, option.to_string());
            let x = menu_x;

            commands.push(UIRenderCommand::DrawText {
                x,
                y,
                text: format!("{:<width$}", option_text, width = menu_width as usize),
                fg,
                bg,
            });

            // Add cursor animation for selected item
            if is_selected && self.show_cursor {
                commands.push(UIRenderCommand::DrawText {
                    x: menu_x + menu_width - 2,
                    y,
                    text: "<".to_string(),
                    fg: Color::Yellow,
                    bg,
                });
            }
        }

        commands
    }

    fn render_instructions(&self, width: i32, height: i32) -> Vec<UIRenderCommand> {
        let mut commands = Vec::new();
        
        let instructions = vec![
            "Use ↑/↓ or W/S to navigate",
            "Press ENTER or SPACE to select",
            "Press ESC or Q to quit",
        ];

        let instructions_start_y = height - instructions.len() as i32 - 4;
        
        for (i, instruction) in instructions.iter().enumerate() {
            let x = (width - instruction.len() as i32) / 2;
            let y = instructions_start_y + i as i32;
            
            commands.push(UIRenderCommand::DrawText {
                x,
                y,
                text: instruction.to_string(),
                fg: Color::DarkGrey,
                bg: Color::Black,
            });
        }

        commands
    }

    fn render_debug_info(&self, width: i32, height: i32) -> Vec<UIRenderCommand> {
        let mut commands = Vec::new();
        
        if let Some(key) = &self.last_key {
            let debug_text = format!("Last key: {:?}", key);
            commands.push(UIRenderCommand::DrawText {
                x: 2,
                y: height - 1,
                text: debug_text,
                fg: Color::DarkGrey,
                bg: Color::Black,
            });
        }

        let frame_text = format!("Frame: {}", self.animation_frame);
        commands.push(UIRenderCommand::DrawText {
            x: 2,
            y: height - 2,
            text: frame_text,
            fg: Color::DarkGrey,
            bg: Color::Black,
        });

        commands
    }
}

impl UIComponent for MainMenu {
    fn render(&self, _x: i32, _y: i32, width: i32, height: i32) -> Vec<UIRenderCommand> {
        let mut commands = Vec::new();

        // Clear background
        for y in 0..height {
            commands.push(UIRenderCommand::DrawText {
                x: 0,
                y,
                text: " ".repeat(width as usize),
                fg: Color::White,
                bg: Color::Black,
            });
        }

        // Render different components
        commands.extend(self.render_title(width, height));
        commands.extend(self.render_menu_options(width, height));
        commands.extend(self.render_instructions(width, height));
        
        // Only show debug info in debug builds
        #[cfg(debug_assertions)]
        commands.extend(self.render_debug_info(width, height));

        // Hide cursor
        commands.push(UIRenderCommand::SetCursor {
            x: 0,
            y: 0,
            visible: false,
        });

        commands
    }

    fn handle_input(&mut self, input: char) -> bool {
        match input {
            'k' | 'w' => {
                self.select_previous();
                true
            }
            'j' | 's' => {
                self.select_next();
                true
            }
            '\n' | ' ' => {
                self.state = self.activate_selected();
                true
            }
            '\x1b' | 'q' => {
                self.state = MainMenuState::Quit;
                true
            }
            _ => false,
        }
    }

    fn is_focused(&self) -> bool {
        true // Main menu is always focused when active
    }

    fn set_focus(&mut self, _focused: bool) {
        // Main menu focus doesn't change
    }
}

/// Menu runner for handling the main menu loop
pub struct MainMenuRunner {
    pub menu: MainMenu,
    pub renderer: MenuRenderer,
}

impl MainMenuRunner {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(MainMenuRunner {
            menu: MainMenu::new(),
            renderer: MenuRenderer::new()?,
        })
    }

    pub fn run(&mut self) -> Result<MainMenuState, Box<dyn std::error::Error>> {
        loop {
            // Update menu animation
            self.menu.update();

            // Render menu
            self.renderer.render_menu(&self.menu)?;

            // Handle input
            if let Some(key_event) = MenuInput::read_key()? {
                if self.menu.handle_key(key_event.code) {
                    // Check if we should exit the menu
                    match self.menu.state {
                        MainMenuState::MainMenu => continue,
                        _ => return Ok(self.menu.state.clone()),
                    }
                }
            }

            // Small delay to prevent excessive CPU usage
            std::thread::sleep(std::time::Duration::from_millis(16)); // ~60 FPS
        }
    }

    pub fn show_message(&mut self, message: &str, duration_ms: u64) -> Result<(), Box<dyn std::error::Error>> {
        let (width, height) = self.renderer.get_screen_size();
        
        // Render current menu
        self.renderer.render_menu(&self.menu)?;
        
        // Overlay message
        let message_panel = UIPanel::new(
            "Message".to_string(),
            width / 4,
            height / 2 - 2,
            width / 2,
            5,
        ).with_colors(Color::Yellow, Color::DarkBlue, Color::White);
        
        let mut commands = message_panel.render();
        
        let (inner_x, inner_y, inner_w, _inner_h) = message_panel.inner_bounds();
        let message_text = UIText::new(
            message.to_string(),
            inner_x,
            inner_y + 1,
            inner_w,
        ).with_alignment(TextAlignment::Center)
         .with_colors(Color::White, Color::DarkBlue);
        
        commands.extend(message_text.render(0, 0, 0, 0));
        
        self.renderer.system.render_commands(&commands)?;
        
        // Wait for specified duration
        std::thread::sleep(std::time::Duration::from_millis(duration_ms));
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_main_menu_creation() {
        let menu = MainMenu::new();
        
        assert_eq!(menu.state, MainMenuState::MainMenu);
        assert_eq!(menu.selected_option, 0);
        assert_eq!(menu.options.len(), 5);
        assert_eq!(menu.title, "ASCII DUNGEON EXPLORER");
    }

    #[test]
    fn test_menu_navigation() {
        let mut menu = MainMenu::new();
        
        assert_eq!(menu.selected_option, 0);
        
        menu.select_next();
        assert_eq!(menu.selected_option, 1);
        
        menu.select_previous();
        assert_eq!(menu.selected_option, 0);
        
        // Test wrap-around
        menu.select_previous();
        assert_eq!(menu.selected_option, menu.options.len() - 1);
        
        menu.select_next();
        assert_eq!(menu.selected_option, 0);
    }

    #[test]
    fn test_menu_activation() {
        let mut menu = MainMenu::new();
        
        // Test New Game activation
        menu.selected_option = 0;
        assert_eq!(menu.activate_selected(), MainMenuState::NewGame);
        
        // Test Quit activation
        menu.selected_option = 4; // Quit is last option
        assert_eq!(menu.activate_selected(), MainMenuState::Quit);
    }

    #[test]
    fn test_menu_key_handling() {
        let mut menu = MainMenu::new();
        
        assert_eq!(menu.selected_option, 0);
        
        // Test navigation keys
        assert!(menu.handle_key(KeyCode::Down));
        assert_eq!(menu.selected_option, 1);
        
        assert!(menu.handle_key(KeyCode::Up));
        assert_eq!(menu.selected_option, 0);
        
        // Test selection key
        assert!(menu.handle_key(KeyCode::Enter));
        assert_eq!(menu.state, MainMenuState::NewGame);
        
        // Reset and test quit key
        menu.reset();
        assert!(menu.handle_key(KeyCode::Esc));
        assert_eq!(menu.state, MainMenuState::Quit);
    }

    #[test]
    fn test_menu_options() {
        let options = MenuOption::all_options();
        
        assert_eq!(options.len(), 5);
        assert_eq!(options[0], MenuOption::NewGame);
        assert_eq!(options[4], MenuOption::Quit);
        
        assert_eq!(MenuOption::NewGame.to_string(), "New Game");
        assert_eq!(MenuOption::Quit.to_string(), "Quit");
    }

    #[test]
    fn test_menu_customization() {
        let menu = MainMenu::new()
            .with_title("Custom Game".to_string(), "Custom Subtitle".to_string())
            .with_version("v1.0.0".to_string());
        
        assert_eq!(menu.title, "Custom Game");
        assert_eq!(menu.subtitle, "Custom Subtitle");
        assert_eq!(menu.version, "v1.0.0");
    }

    #[test]
    fn test_menu_reset() {
        let mut menu = MainMenu::new();
        
        menu.selected_option = 3;
        menu.state = MainMenuState::Options;
        menu.last_key = Some(KeyCode::Enter);
        
        menu.reset();
        
        assert_eq!(menu.state, MainMenuState::MainMenu);
        assert_eq!(menu.selected_option, 0);
        assert_eq!(menu.last_key, None);
    }
}