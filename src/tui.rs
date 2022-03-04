//! Utilities for the terminal UI of toipe.

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

/// Describes something that has a printable length.
///
/// For example, a string containing color characters has a different
/// length when printed than the number of bytes or chars in it.
pub trait HasLength {
    /// number of char widths taken when printed on the terminal
    fn length(&self) -> usize;
}

/// Holds some text that is to be printed on the terminal.
///
/// This provides an abstraction for
/// - retrieving the number of actual characters when printed on the
///   terminal through the [`HasLength`] trait.
/// - for formatting the text through the various `with_*` methods.
///
/// Usually, this is used in the slice form as `&[Text]` since a
/// single [`Text`] only holds one string with all of it formatted in
/// the same way. For example, you cannot format one part of a [`Text`]
/// with green color while the rest is in red. You should instead use a
/// slice of [`Text`]s with each formatting in a different way.
pub struct Text {
    /// the raw text
    text: String,
    /// actual number of char width taken when printed on the terminal
    length: usize,
}

impl Text {
    /// Constructs a new Text from a raw string
    ///
    /// NOTE: ensure that this string does not itself have formatting
    /// characters, zero-width characters or multi-width characters.
    pub fn new(text: String) -> Self {
        let length = text.len();
        Self { text, length }
    }

    /// the raw text
    pub fn text(&self) -> &String {
        &self.text
    }

    /// adds faint style to the text
    pub fn with_faint(mut self) -> Self {
        self.text = format!("{}{}{}", style::Faint, self.text, style::NoFaint);
        self
    }

    /// adds underline to the text
    pub fn with_underline(mut self) -> Self {
        self.text = format!("{}{}{}", style::Underline, self.text, style::Reset);
        self
    }

    /// adds given color to the text
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

/// NOTE: note to be confused with `.len()` which provides the number
/// of elements in the slice.
impl HasLength for [Text] {
    fn length(&self) -> usize {
        self.iter().map(|t| t.length()).sum()
    }
}

impl From<String> for Text {
    /// Constructs a new Text from a raw string
    ///
    /// NOTE: ensure that this string does not itself have formatting
    /// characters, zero-width characters or multi-width characters.
    fn from(text: String) -> Self {
        Self::new(text)
    }
}

impl From<&str> for Text {
    /// Constructs a new Text from a raw string
    ///
    /// NOTE: ensure that this string does not itself have formatting
    /// characters, zero-width characters or multi-width characters.
    fn from(text: &str) -> Self {
        Self::new(text.to_string())
    }
}

impl From<char> for Text {
    /// Constructs a new Text from a character
    ///
    /// NOTE: ensure that this character is itself not a formatting
    /// character, zero-width character or a multi-width character.
    fn from(c: char) -> Self {
        Self::new(c.to_string())
    }
}

/// Displays the raw string as-is. No surprises here.
impl Display for Text {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.text)
    }
}

/// terminal UI of toipe
pub struct ToipeTui {
    stdout: RawTerminal<Stdout>,
}

type MaybeError = Result<(), ToipeError>;

impl ToipeTui {
    /// Initializes stdout in raw mode for the TUI.
    ///
    /// NOTE: does not clear the screen when initialized.
    pub fn new() -> Self {
        Self {
            stdout: stdout().into_raw_mode().unwrap(),
        }
    }

    // TODO: make this private
    /// Flushes stdout
    pub fn flush(&mut self) -> MaybeError {
        self.stdout.flush()?;
        Ok(())
    }

    /// Resets the TUI.
    ///
    /// Clears screen, moves cursor to the center and changes cursor to
    /// a blinking bar.
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

    /// Displays a single line of text.
    ///
    /// - A line of text is described by a slice of [`Text`]s, they are
    ///   concatenated and displayed on the same line.
    ///
    /// - The line is displayed on whatever Y-position the cursor is
    ///   currently on.
    ///
    /// - The line is centered horizontally.
    pub fn display_a_line(&mut self, text: &[Text]) -> MaybeError {
        self.display_a_line_raw(text)?;
        self.flush()?;

        Ok(())
    }

    /// Same as [`display_a_line`] but without the flush.
    fn display_a_line_raw(&mut self, text: &[Text]) -> MaybeError {
        let len = text.length() as u16;
        write!(self.stdout, "{}", cursor::Left(len / 2),)?;
        for t in text {
            self.display_raw_text(t)?;
        }
        write!(self.stdout, "{}", cursor::Left(len),)?;

        Ok(())
    }

    /// Displays multiple lines of text.
    ///
    /// - A line of text is described by a slice of [`Text`]s, they are
    ///   concatenated and displayed on the same line.
    ///
    /// - The lines are centered vertically and each line itself is
    ///   centered horizontally.
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

    /// Displays a [`Text`].
    pub fn display_raw_text(&mut self, text: &Text) -> MaybeError {
        write!(self.stdout, "{}", text)?;
        Ok(())
    }

    /// Hides the cursor.
    pub fn hide_cursor(&mut self) -> MaybeError {
        write!(self.stdout, "{}", cursor::Hide)?;
        self.flush()?;
        Ok(())
    }

    /// Shows the cursor.
    pub fn show_cursor(&mut self) -> MaybeError {
        write!(self.stdout, "{}", cursor::Show)?;
        self.flush()?;
        Ok(())
    }

    /// Replaces the text previous to the cursor with given text.
    ///
    /// Last N characters are replaced with given text. N is the number
    /// of characters in the given text.
    ///
    /// The text is described by a slice of [`Text`].
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
