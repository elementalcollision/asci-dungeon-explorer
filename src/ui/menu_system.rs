use crossterm::{
    event::{Event, KeyCode, KeyEvent},
    style::Color,
    terminal::{size, Clear, ClearType},
    cursor::{MoveTo, Hide, Show},
    ExecutableCommand,
};
use std::io::{stdout, Write};
use crate::ui::ui_components::{UIRenderCommand, UIComponent};

/// System for handling menu rendering and input
pub struct MenuSystem {
    pub width: i32,
    pub height: i32,
    pub background_color: Color,
}

impl MenuSystem {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let (width, height) = size()?;
        
        Ok(MenuSystem {
            width: width as i32,
            height: height as i32,
            background_color: Color::Black,
        })
    }

    pub fn update_size(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let (width, height) = size()?;
        self.width = width as i32;
        self.height = height as i32;
        Ok(())
    }

    pub fn clear_screen(&self) -> Result<(), Box<dyn std::error::Error>> {
        stdout().execute(Clear(ClearType::All))?;
        stdout().execute(MoveTo(0, 0))?;
        Ok(())
    }

    pub fn render_commands(&self, commands: &[UIRenderCommand]) -> Result<(), Box<dyn std::error::Error>> {
        for command in commands {
            self.execute_render_command(command)?;
        }
        stdout().flush()?;
        Ok(())
    }

    fn execute_render_command(&self, command: &UIRenderCommand) -> Result<(), Box<dyn std::error::Error>> {
        match command {
            UIRenderCommand::DrawText { x, y, text, fg, bg } => {
                if *x >= 0 && *y >= 0 && *x < self.width && *y < self.height {
                    stdout().execute(MoveTo(*x as u16, *y as u16))?;
                    print!("{}", crossterm::style::style(text).with(*fg).on(*bg));
                }
            }
            UIRenderCommand::DrawBox { x, y, width, height, border_color, fill_color } => {
                self.draw_box(*x, *y, *width, *height, *border_color, *fill_color)?;
            }
            UIRenderCommand::DrawLine { x1, y1, x2, y2, color, character } => {
                self.draw_line(*x1, *y1, *x2, *y2, *color, *character)?;
            }
            UIRenderCommand::SetCursor { x, y, visible } => {
                if *visible {
                    stdout().execute(Show)?;
                    stdout().execute(MoveTo(*x as u16, *y as u16))?;
                } else {
                    stdout().execute(Hide)?;
                }
            }
        }
        Ok(())
    }

    fn draw_box(&self, x: i32, y: i32, width: i32, height: i32, border_color: Color, fill_color: Color) -> Result<(), Box<dyn std::error::Error>> {
        // Draw top border
        if y >= 0 && y < self.height {
            stdout().execute(MoveTo(x as u16, y as u16))?;
            let top_line = format!("┌{}┐", "─".repeat((width - 2).max(0) as usize));
            print!("{}", crossterm::style::style(top_line).with(border_color).on(fill_color));
        }

        // Draw sides and fill
        for row in 1..height - 1 {
            let current_y = y + row;
            if current_y >= 0 && current_y < self.height {
                stdout().execute(MoveTo(x as u16, current_y as u16))?;
                let side_line = format!("│{}│", " ".repeat((width - 2).max(0) as usize));
                print!("{}", crossterm::style::style(side_line).with(border_color).on(fill_color));
            }
        }

        // Draw bottom border
        let bottom_y = y + height - 1;
        if bottom_y >= 0 && bottom_y < self.height && height > 1 {
            stdout().execute(MoveTo(x as u16, bottom_y as u16))?;
            let bottom_line = format!("└{}┘", "─".repeat((width - 2).max(0) as usize));
            print!("{}", crossterm::style::style(bottom_line).with(border_color).on(fill_color));
        }

        Ok(())
    }

    fn draw_line(&self, x1: i32, y1: i32, x2: i32, y2: i32, color: Color, character: char) -> Result<(), Box<dyn std::error::Error>> {
        // Simple line drawing - only horizontal and vertical lines for now
        if y1 == y2 {
            // Horizontal line
            let start_x = x1.min(x2);
            let end_x = x1.max(x2);
            if y1 >= 0 && y1 < self.height {
                for x in start_x..=end_x {
                    if x >= 0 && x < self.width {
                        stdout().execute(MoveTo(x as u16, y1 as u16))?;
                        print!("{}", crossterm::style::style(character).with(color));
                    }
                }
            }
        } else if x1 == x2 {
            // Vertical line
            let start_y = y1.min(y2);
            let end_y = y1.max(y2);
            if x1 >= 0 && x1 < self.width {
                for y in start_y..=end_y {
                    if y >= 0 && y < self.height {
                        stdout().execute(MoveTo(x1 as u16, y as u16))?;
                        print!("{}", crossterm::style::style(character).with(color));
                    }
                }
            }
        }
        Ok(())
    }
}

/// Menu renderer for drawing menus
pub struct MenuRenderer {
    pub system: MenuSystem,
}

impl MenuRenderer {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(MenuRenderer {
            system: MenuSystem::new()?,
        })
    }

    pub fn render_menu<T: UIComponent>(&mut self, menu: &T) -> Result<(), Box<dyn std::error::Error>> {
        self.system.update_size()?;
        self.system.clear_screen()?;
        
        let commands = menu.render(0, 0, self.system.width, self.system.height);
        self.system.render_commands(&commands)?;
        
        Ok(())
    }

    pub fn render_centered_text(&self, text: &str, y: i32, color: Color) -> Result<(), Box<dyn std::error::Error>> {
        let x = (self.system.width - text.len() as i32) / 2;
        let command = UIRenderCommand::DrawText {
            x,
            y,
            text: text.to_string(),
            fg: color,
            bg: Color::Black,
        };
        self.system.render_commands(&[command])?;
        Ok(())
    }

    pub fn render_title(&self, title: &str, subtitle: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
        // Render main title
        self.render_centered_text(title, 2, Color::Yellow)?;
        
        // Render subtitle if provided
        if let Some(sub) = subtitle {
            self.render_centered_text(sub, 4, Color::White)?;
        }
        
        // Draw separator line
        let line_y = if subtitle.is_some() { 6 } else { 4 };
        let line_width = title.len().max(subtitle.map(|s| s.len()).unwrap_or(0));
        let line_x = (self.system.width - line_width as i32) / 2;
        
        let command = UIRenderCommand::DrawLine {
            x1: line_x,
            y1: line_y,
            x2: line_x + line_width as i32 - 1,
            y2: line_y,
            color: Color::DarkGrey,
            character: '─',
        };
        self.system.render_commands(&[command])?;
        
        Ok(())
    }

    pub fn get_screen_center(&self) -> (i32, i32) {
        (self.system.width / 2, self.system.height / 2)
    }

    pub fn get_screen_size(&self) -> (i32, i32) {
        (self.system.width, self.system.height)
    }
}

/// Input handler for menus
pub struct MenuInput;

impl MenuInput {
    pub fn read_key() -> Result<Option<KeyEvent>, Box<dyn std::error::Error>> {
        if crossterm::event::poll(std::time::Duration::from_millis(100))? {
            match crossterm::event::read()? {
                Event::Key(key_event) => Ok(Some(key_event)),
                _ => Ok(None),
            }
        } else {
            Ok(None)
        }
    }

    pub fn key_to_char(key: KeyCode) -> Option<char> {
        match key {
            KeyCode::Char(c) => Some(c),
            KeyCode::Enter => Some('\n'),
            KeyCode::Tab => Some('\t'),
            KeyCode::Backspace => Some('\x08'),
            KeyCode::Esc => Some('\x1b'),
            KeyCode::Up => Some('k'),
            KeyCode::Down => Some('j'),
            KeyCode::Left => Some('h'),
            KeyCode::Right => Some('l'),
            _ => None,
        }
    }

    pub fn is_quit_key(key: KeyCode) -> bool {
        matches!(key, KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q'))
    }

    pub fn is_select_key(key: KeyCode) -> bool {
        matches!(key, KeyCode::Enter | KeyCode::Char(' '))
    }

    pub fn is_navigation_key(key: KeyCode) -> bool {
        matches!(key, 
            KeyCode::Up | KeyCode::Down | KeyCode::Left | KeyCode::Right |
            KeyCode::Char('w') | KeyCode::Char('s') | KeyCode::Char('a') | KeyCode::Char('d') |
            KeyCode::Char('k') | KeyCode::Char('j') | KeyCode::Char('h') | KeyCode::Char('l')
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_menu_input_key_conversion() {
        assert_eq!(MenuInput::key_to_char(KeyCode::Char('a')), Some('a'));
        assert_eq!(MenuInput::key_to_char(KeyCode::Enter), Some('\n'));
        assert_eq!(MenuInput::key_to_char(KeyCode::Tab), Some('\t'));
        assert_eq!(MenuInput::key_to_char(KeyCode::Up), Some('k'));
        assert_eq!(MenuInput::key_to_char(KeyCode::Down), Some('j'));
    }

    #[test]
    fn test_menu_input_key_checks() {
        assert!(MenuInput::is_quit_key(KeyCode::Esc));
        assert!(MenuInput::is_quit_key(KeyCode::Char('q')));
        assert!(MenuInput::is_quit_key(KeyCode::Char('Q')));
        assert!(!MenuInput::is_quit_key(KeyCode::Char('a')));

        assert!(MenuInput::is_select_key(KeyCode::Enter));
        assert!(MenuInput::is_select_key(KeyCode::Char(' ')));
        assert!(!MenuInput::is_select_key(KeyCode::Char('a')));

        assert!(MenuInput::is_navigation_key(KeyCode::Up));
        assert!(MenuInput::is_navigation_key(KeyCode::Char('w')));
        assert!(MenuInput::is_navigation_key(KeyCode::Char('j')));
        assert!(!MenuInput::is_navigation_key(KeyCode::Char('x')));
    }
}