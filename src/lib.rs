pub mod textgen;

use std::io::{stdout, StdinLock, Stdout, Write};

use termion::{
    clear, color, cursor,
    event::Key,
    input::TermRead,
    raw::{IntoRawMode, RawTerminal},
    style, terminal_size,
};
use textgen::WordSelector;

pub struct Toipe {
    stdout: RawTerminal<Stdout>,
    text: String,
}

#[derive(Debug)]
pub struct ToipeError {
    pub msg: String,
}

impl From<std::io::Error> for ToipeError {
    fn from(error: std::io::Error) -> Self {
        ToipeError {
            msg: error.to_string(),
        }
    }
}

impl<'a> Toipe {
    pub fn new() -> Result<Self, ToipeError> {
        let stdout = stdout().into_raw_mode().unwrap();

        let mut toipe = Toipe {
            stdout,
            text: "".to_string(),
        };

        toipe.reset_screen()?;

        toipe.gen_words()?;

        Ok(toipe)
    }

    fn reset_screen(&mut self) -> Result<(), ToipeError> {
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

    fn gen_words(&mut self) -> Result<(), ToipeError> {
        let word_selector = WordSelector::default();
        let words: Result<Vec<String>, _> = (0..10)
            .into_iter()
            .map(|_| word_selector.new_word())
            .collect();

        self.text = words?.join(" ");

        write!(
            self.stdout,
            "{}{}{}{}{}",
            style::Faint,
            cursor::Left(self.text.len() as u16 / 2),
            self.text,
            cursor::Left(self.text.len() as u16),
            style::NoFaint,
        )?;
        self.flush()?;

        Ok(())
    }

    pub fn flush(&mut self) -> Result<(), ToipeError> {
        self.stdout.flush()?;
        Ok(())
    }

    pub fn test(mut self, stdin: StdinLock<'a>) -> Result<(), ToipeError> {
        let mut input = Vec::<char>::new();
        let text: Vec<char> = self.text.chars().collect();
        let mut num_errors = 0;

        for key in stdin.keys() {
            match key? {
                Key::Ctrl(c) => match c {
                    'c' => {
                        break;
                    }
                    _ => {}
                },
                Key::Char(c) => {
                    input.push(c);

                    if input.len() >= text.len() {
                        break;
                    }

                    if text[input.len() - 1] == c {
                        write!(self.stdout, "{}", c)?;
                    } else {
                        write!(
                            self.stdout,
                            "{}{}{}{}{}",
                            color::Fg(color::Red),
                            style::Underline,
                            text[input.len() - 1],
                            color::Fg(color::Reset),
                            style::NoUnderline,
                        )?;
                        num_errors += 1;
                    }
                }
                Key::Backspace => {
                    let last_char = input.pop();
                    if let Some(_) = last_char {
                        write!(
                            self.stdout,
                            "{}{}{}{}{}",
                            style::Faint,
                            cursor::Left(1),
                            text[input.len()],
                            cursor::Left(1),
                            style::NoFaint,
                        )?;
                    }
                }
                _ => {}
            }

            // write!(self.stdout, "{:?}", key)?;
            self.flush()?;
        }

        write!(self.stdout, "Errors: {}", num_errors)?;

        Ok(())
    }
}

impl Drop for Toipe {
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
