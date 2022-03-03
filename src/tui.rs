use std::{
    fmt::Display,
    io::{stdout, Stdout, Write},
};

use termion::{
    clear,
    color::{self, Color},
    cursor,
    raw::{IntoRawMode, RawTerminal},
    style, terminal_size,
};

use crate::ToipeError;

// TODO: document this
pub trait HasLength {
    fn length(&self) -> usize;
}

// TODO: document this
pub struct Text {
    text: String,
    length: usize,
}

impl Text {
    pub fn new(text: String) -> Self {
        let length = text.len();
        Self { text, length }
    }

    pub fn text(&self) -> &String {
        &self.text
    }

    pub fn with_faint(mut self) -> Self {
        self.text = format!("{}{}{}", style::Faint, self.text, style::NoFaint);
        self
    }

    pub fn with_underline(mut self) -> Self {
        self.text = format!("{}{}{}", style::Underline, self.text, style::Reset);
        self
    }

    pub fn with_color<C>(mut self, color: C) -> Self
    where
        C: Color,
    {
        self.text = format!(
            "{}{}{}",
            color::Fg(color),
            self.text,
            color::Fg(color::Reset)
        );
        self
    }
}

impl HasLength for Text {
    fn length(&self) -> usize {
        self.length
    }
}

impl HasLength for [Text] {
    fn length(&self) -> usize {
        self.iter().map(|t| t.length()).sum()
    }
}

impl From<String> for Text {
    fn from(text: String) -> Self {
        Self::new(text)
    }
}

impl From<&str> for Text {
    fn from(text: &str) -> Self {
        Self::new(text.to_string())
    }
}

impl From<char> for Text {
    fn from(c: char) -> Self {
        Self::new(c.to_string())
    }
}

impl Display for Text {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.text)
    }
}

/// terminal UI of toipe
pub struct ToipeTui {
    // TODO: make this private
    pub stdout: RawTerminal<Stdout>,
}

type MaybeError = Result<(), ToipeError>;

// TODO: document these functions
impl ToipeTui {
    pub fn new() -> Self {
        Self {
            stdout: stdout().into_raw_mode().unwrap(),
        }
    }

    // TODO: make this private
    pub fn flush(&mut self) -> MaybeError {
        self.stdout.flush()?;
        Ok(())
    }

    pub fn reset_screen(&mut self) -> MaybeError {
        let (sizex, sizey) = terminal_size()?;

        write!(
            self.stdout,
            "{}{}{}",
            clear::All,
            cursor::Goto(sizex / 2, sizey / 2),
            cursor::BlinkingBar
        )?;
        self.flush()?;

        Ok(())
    }

    pub fn display_a_line(&mut self, text: &[Text]) -> MaybeError {
        self.display_a_line_raw(text)?;
        self.flush()?;

        Ok(())
    }

    fn display_a_line_raw(&mut self, text: &[Text]) -> MaybeError {
        let len = text.length() as u16;
        write!(self.stdout, "{}", cursor::Left(len / 2),)?;
        for t in text {
            self.display_raw_text(t)?;
        }
        write!(self.stdout, "{}", cursor::Left(len),)?;

        Ok(())
    }

    pub fn display_lines(&mut self, lines: &[&[Text]]) -> MaybeError {
        let (sizex, sizey) = terminal_size()?;

        let mut line_no = 0;
        let line_offset = lines.len() as u16 / 2;

        for line in lines {
            write!(
                self.stdout,
                "{}",
                cursor::Goto(sizex / 2, sizey / 2 + line_no - line_offset)
            )?;
            self.display_a_line_raw(line)?;
            line_no += 1;
        }
        self.flush()?;

        Ok(())
    }

    pub fn display_raw_text(&mut self, text: &Text) -> MaybeError {
        write!(self.stdout, "{}", text)?;
        Ok(())
    }

    pub fn hide_cursor(&mut self) -> MaybeError {
        write!(self.stdout, "{}", cursor::Hide)?;
        self.flush()?;
        Ok(())
    }

    pub fn show_cursor(&mut self) -> MaybeError {
        write!(self.stdout, "{}", cursor::Show)?;
        self.flush()?;
        Ok(())
    }

    pub fn replace_text(&mut self, texts: &[Text]) -> MaybeError {
        let len = texts.length() as u16;

        write!(self.stdout, "{}", cursor::Left(len))?;
        for text in texts {
            self.display_raw_text(text)?;
        }
        write!(self.stdout, "{}", cursor::Left(len))?;

        Ok(())
    }
}

impl Drop for ToipeTui {
    /// Resets terminal.
    ///
    /// Clears screen and sets the cursor to a non-blinking block.
    ///
    /// TODO: reset cursor to whatever it was before Toipe was started.
    fn drop(&mut self) {
        write!(
            self.stdout,
            "{}{}{}",
            clear::All,
            cursor::SteadyBlock,
            cursor::Goto(1, 1)
        )
        .expect("Could not reset terminal while exiting");
        self.flush().expect("Could not flush stdout while exiting");
    }
}
