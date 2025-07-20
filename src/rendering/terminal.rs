use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent},
    execute, queue,
    style::{self, Color, SetBackgroundColor, SetForegroundColor},
    terminal::{self, ClearType},
    Result as CrosstermResult,
};
use std::io::{stdout, Write};

/// A wrapper around terminal functionality to provide a clean interface
pub struct Terminal {
    width: u16,
    height: u16,
    stdout: std::io::Stdout,
}

impl Terminal {
    /// Create a new terminal instance
    pub fn new() -> CrosstermResult<Self> {
        let (width, height) = terminal::size()?;
        Ok(Terminal {
            width,
            height,
            stdout: stdout(),
        })
    }

    /// Initialize the terminal for rendering
    pub fn init(&mut self) -> CrosstermResult<()> {
        terminal::enable_raw_mode()?;
        execute!(
            self.stdout,
            terminal::EnterAlternateScreen,
            cursor::Hide,
            terminal::Clear(ClearType::All)
        )?;
        Ok(())
    }

    /// Clean up the terminal when the program exits
    pub fn cleanup(&mut self) -> CrosstermResult<()> {
        terminal::disable_raw_mode()?;
        execute!(
            self.stdout,
            style::ResetColor,
            cursor::Show,
            terminal::LeaveAlternateScreen
        )?;
        Ok(())
    }

    /// Clear the entire screen
    pub fn clear(&mut self) -> CrosstermResult<()> {
        execute!(self.stdout, terminal::Clear(ClearType::All))?;
        Ok(())
    }

    /// Clear a specific line
    pub fn clear_line(&mut self, y: u16) -> CrosstermResult<()> {
        execute!(
            self.stdout,
            cursor::MoveTo(0, y),
            terminal::Clear(ClearType::CurrentLine)
        )?;
        Ok(())
    }

    /// Move the cursor to a specific position
    pub fn move_cursor(&mut self, x: u16, y: u16) -> CrosstermResult<()> {
        execute!(self.stdout, cursor::MoveTo(x, y))?;
        Ok(())
    }

    /// Draw a single character at the current cursor position
    pub fn draw_char(&mut self, c: char, fg: Color, bg: Color) -> CrosstermResult<()> {
        queue!(
            self.stdout,
            SetForegroundColor(fg),
            SetBackgroundColor(bg),
            style::Print(c)
        )?;
        Ok(())
    }

    /// Draw a character at a specific position
    pub fn draw_char_at(&mut self, x: u16, y: u16, c: char, fg: Color, bg: Color) -> CrosstermResult<()> {
        queue!(
            self.stdout,
            cursor::MoveTo(x, y),
            SetForegroundColor(fg),
            SetBackgroundColor(bg),
            style::Print(c)
        )?;
        Ok(())
    }

    /// Draw text at a specific position
    pub fn draw_text(&mut self, x: u16, y: u16, text: &str, fg: Color, bg: Color) -> CrosstermResult<()> {
        queue!(
            self.stdout,
            cursor::MoveTo(x, y),
            SetForegroundColor(fg),
            SetBackgroundColor(bg),
            style::Print(text)
        )?;
        Ok(())
    }

    /// Draw text centered horizontally at a specific y position
    pub fn draw_text_centered(&mut self, y: u16, text: &str, fg: Color, bg: Color) -> CrosstermResult<()> {
        let x = self.width.saturating_sub(text.len() as u16) / 2;
        self.draw_text(x, y, text, fg, bg)?;
        Ok(())
    }

    /// Draw a box with a border
    pub fn draw_box(&mut self, x: u16, y: u16, width: u16, height: u16, fg: Color, bg: Color) -> CrosstermResult<()> {
        // Draw the corners
        self.draw_char_at(x, y, '┌', fg, bg)?;
        self.draw_char_at(x + width - 1, y, '┐', fg, bg)?;
        self.draw_char_at(x, y + height - 1, '└', fg, bg)?;
        self.draw_char_at(x + width - 1, y + height - 1, '┘', fg, bg)?;

        // Draw the horizontal edges
        for i in 1..width - 1 {
            self.draw_char_at(x + i, y, '─', fg, bg)?;
            self.draw_char_at(x + i, y + height - 1, '─', fg, bg)?;
        }

        // Draw the vertical edges
        for i in 1..height - 1 {
            self.draw_char_at(x, y + i, '│', fg, bg)?;
            self.draw_char_at(x + width - 1, y + i, '│', fg, bg)?;
        }

        Ok(())
    }

    /// Fill a rectangle with a specific character and colors
    pub fn fill_rect(&mut self, x: u16, y: u16, width: u16, height: u16, c: char, fg: Color, bg: Color) -> CrosstermResult<()> {
        for j in 0..height {
            for i in 0..width {
                self.draw_char_at(x + i, y + j, c, fg, bg)?;
            }
        }
        Ok(())
    }

    /// Draw a horizontal line
    pub fn draw_horizontal_line(&mut self, x: u16, y: u16, width: u16, fg: Color, bg: Color) -> CrosstermResult<()> {
        for i in 0..width {
            self.draw_char_at(x + i, y, '─', fg, bg)?;
        }
        Ok(())
    }

    /// Draw a vertical line
    pub fn draw_vertical_line(&mut self, x: u16, y: u16, height: u16, fg: Color, bg: Color) -> CrosstermResult<()> {
        for i in 0..height {
            self.draw_char_at(x, y + i, '│', fg, bg)?;
        }
        Ok(())
    }

    /// Flush the output buffer to the terminal
    pub fn flush(&mut self) -> CrosstermResult<()> {
        self.stdout.flush()?;
        Ok(())
    }

    /// Check if a key is pressed and return the key event
    pub fn poll_key(&self, timeout_ms: u64) -> CrosstermResult<Option<KeyEvent>> {
        if event::poll(std::time::Duration::from_millis(timeout_ms))? {
            if let Event::Key(key_event) = event::read()? {
                return Ok(Some(key_event));
            }
        }
        Ok(None)
    }

    /// Get the terminal size
    pub fn size(&self) -> (u16, u16) {
        (self.width, self.height)
    }

    /// Update the stored terminal size
    pub fn update_size(&mut self) -> CrosstermResult<()> {
        let (width, height) = terminal::size()?;
        self.width = width;
        self.height = height;
        Ok(())
    }
}

/// A helper function to execute code with a terminal
pub fn with_terminal<F, T>(f: F) -> CrosstermResult<T>
where
    F: FnOnce(&mut Terminal) -> CrosstermResult<T>,
{
    let mut terminal = Terminal::new()?;
    terminal.init()?;
    
    let result = f(&mut terminal);
    
    terminal.cleanup()?;
    result
}