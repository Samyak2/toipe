//! Toipe is a terminal-based typing test application.
//!
//! Please see the [README](https://github.com/Samyak2/toipe/) for
//! installation and usage instructions.
//!
//! Toipe provides an API to invoke it from another application or
//! library. This documentation describes the API and algorithms used
//! internally.
//!
//! See [`RawWordSelector`] if you're looking for the word selection
//! algorithm.

pub mod config;
pub mod textgen;
pub mod wordlists;

use std::io::{stdout, StdinLock, Stdout, Write};
use std::path::PathBuf;
use std::time::{Duration, Instant};

use config::ToipeConfig;
use termion::input::Keys;
use termion::{
    clear, color, cursor,
    event::Key,
    input::TermRead,
    raw::{IntoRawMode, RawTerminal},
    style, terminal_size,
};
use textgen::{RawWordSelector, WordSelector};
use wordlists::{get_word_list, OS_WORDLIST_PATH};

/// Typing test terminal UI and logic.
pub struct Toipe {
    stdout: RawTerminal<Stdout>,
    text: String,
    words: Vec<String>,
    word_selector: Box<dyn WordSelector>,
}

/// Represents any error caught in Toipe.
#[derive(Debug)]
pub struct ToipeError {
    pub msg: String,
}

/// Converts [`std::io::Error`] to [`ToipeError`].
///
/// This keeps only the error message.
///
/// TODO: there must be a better way to keep information from the
/// original error.
impl From<std::io::Error> for ToipeError {
    fn from(error: std::io::Error) -> Self {
        ToipeError {
            msg: error.to_string(),
        }
    }
}

impl<'a> Toipe {
    /// Initializes a new typing test on the standard output.
    ///
    /// See [`ToipeConfig`] for configuration options.
    ///
    /// Puts `stdout` in raw mode and initializes the word selector.
    /// Also invokes [`Toipe::restart()`].
    pub fn new(config: ToipeConfig) -> Result<Self, ToipeError> {
        let stdout = stdout().into_raw_mode().unwrap();

        let word_selector: Box<dyn WordSelector> =
            if let Some(word_list) = get_word_list(config.wordlist.as_str()) {
                Box::new(RawWordSelector::from_string(word_list.to_string())?)
            } else if config.wordlist == "os" {
                Box::new(RawWordSelector::from_path(PathBuf::from(OS_WORDLIST_PATH))?)
            } else {
                Box::new(RawWordSelector::from_path(PathBuf::from(config.wordlist))?)
            };

        let mut toipe = Toipe {
            stdout,
            text: "".to_string(),
            words: Vec::new(),
            word_selector,
        };

        toipe.restart()?;

        Ok(toipe)
    }

    /// Make the terminal ready for the next typing test.
    ///
    /// Clears the screen, generates new words and displays them on the
    /// UI.
    pub fn restart(&mut self) -> Result<(), ToipeError> {
        self.reset_screen()?;

        self.words = self.word_selector.new_words(10)?;
        self.text = self.words.join(" ");

        self.show_words()?;

        Ok(())
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

    fn show_words(&mut self) -> Result<(), ToipeError> {
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

    fn flush(&mut self) -> Result<(), ToipeError> {
        self.stdout.flush()?;
        Ok(())
    }

    /// Start typing test by monitoring input keys.
    ///
    /// Must only be invoked after [`Toipe::restart()`].
    ///
    /// If the test completes successfully, returns a boolean indicating
    /// whether the user wants to do another test and the
    /// [`ToipeResults`] for this test.
    pub fn test(&mut self, stdin: StdinLock<'a>) -> Result<(bool, ToipeResults), ToipeError> {
        let mut input = Vec::<char>::new();
        let text: Vec<char> = self.text.chars().collect();
        let mut num_errors = 0;
        let mut num_chars_typed = 0;

        let mut process_key = |key: Key| -> Result<bool, ToipeError> {
            match key {
                Key::Ctrl(c) => match c {
                    'c' => {
                        return Ok(false);
                    }
                    _ => {}
                },
                Key::Char(c) => {
                    input.push(c);

                    if input.len() >= text.len() {
                        return Ok(false);
                    }

                    num_chars_typed += 1;

                    if text[input.len() - 1] == c {
                        write!(
                            self.stdout,
                            "{}{}{}",
                            color::Fg(color::LightGreen),
                            c,
                            color::Fg(color::Reset)
                        )?;
                    } else {
                        write!(
                            self.stdout,
                            "{}{}{}{}{}",
                            color::Fg(color::Red),
                            style::Underline,
                            text[input.len() - 1],
                            style::Reset,
                            color::Fg(color::Reset),
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
                            cursor::Left(1),
                            style::Faint,
                            text[input.len()],
                            style::Reset,
                            cursor::Left(1),
                        )?;
                    }
                }
                _ => {}
            }

            // write!(self.stdout, "{:?}", key)?;
            self.flush()?;

            Ok(true)
        };

        let mut keys = stdin.keys();

        // read first key
        let key = keys.next().unwrap()?;
        // start the timer
        let started_at = Instant::now();
        // process first key
        let res = process_key(key)?;

        // process other keys if first key wasn't exit
        if res {
            for key in &mut keys {
                let res = process_key(key?)?;
                // stop if key was exit (ctrl-c)
                if !res {
                    break;
                }
            }
        }

        // stop the timer
        let ended_at = Instant::now();

        let results = ToipeResults {
            num_words: self.words.len(),
            num_chars_typed,
            num_chars_text: input.len(),
            num_errors,
            started_at,
            ended_at,
        };

        let to_restart = self.display_results(results.clone(), keys)?;

        Ok((to_restart, results))
    }

    fn display_results(
        &mut self,
        results: ToipeResults,
        mut keys: Keys<StdinLock>,
    ) -> Result<bool, ToipeError> {
        self.reset_screen()?;

        let (sizex, sizey) = terminal_size()?;

        let line = format!("Accuracy: {:.1}%", results.accuracy() * 100.0);
        write!(
            self.stdout,
            "{}{}{}{}{}",
            cursor::Goto(sizex / 2, sizey / 2 - 2),
            cursor::Left(line.len() as u16 / 2),
            color::Fg(color::Blue),
            line,
            color::Fg(color::Reset),
        )?;

        let line = format!(
            "Mistakes: {} out of {} characters",
            results.num_errors, results.num_chars_text
        );
        write!(
            self.stdout,
            "{}{}{}",
            cursor::Goto(sizex / 2, sizey / 2 - 1),
            cursor::Left(line.len() as u16 / 2),
            line,
        )?;

        let line = format!(
            "Speed: {}{:.1} wpm{} (words per minute)",
            color::Fg(color::Green),
            results.wpm(),
            color::Fg(color::Reset)
        );
        // do not consider length of formatting characters
        let zerowidths = format!("{}{}", color::Fg(color::Green), color::Fg(color::Reset));
        write!(
            self.stdout,
            "{}{}{}",
            cursor::Goto(sizex / 2, sizey / 2),
            cursor::Left((line.len() - zerowidths.len()) as u16 / 2),
            line,
        )?;

        let line = format!(
            "Speed: {}{:.1} cpm{} (characters per minute)",
            color::Fg(color::Cyan),
            results.cpm(),
            color::Fg(color::Reset)
        );
        // do not consider length of formatting characters
        let zerowidths = format!("{}{}", color::Fg(color::Cyan), color::Fg(color::Reset));
        write!(
            self.stdout,
            "{}{}{}",
            cursor::Goto(sizex / 2, sizey / 2 + 1),
            cursor::Left((line.len() - zerowidths.len()) as u16 / 2),
            line,
        )?;

        // no cursor on results page
        write!(self.stdout, "{}", cursor::Hide)?;
        self.flush()?;

        let mut to_restart = false;
        // press 'r' to restart
        match keys.next().unwrap()? {
            Key::Char('r') => to_restart = true,
            _ => {}
        }

        write!(self.stdout, "{}", cursor::Show)?;
        self.flush()?;

        Ok(to_restart)
    }
}

impl Drop for Toipe {
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

/// Stores stats from a typing test.
#[derive(Clone)]
pub struct ToipeResults {
    num_words: usize,
    num_chars_typed: usize,
    num_chars_text: usize,
    num_errors: usize,
    started_at: Instant,
    ended_at: Instant,
}

impl ToipeResults {
    /// Number of correctly typed letters
    pub fn num_correct_chars(&self) -> usize {
        self.num_chars_typed - self.num_errors
    }

    /// Duration of the test.
    ///
    /// i.e., the time between the user pressing the first key and them
    /// typing the last letter.
    pub fn duration(&self) -> Duration {
        self.ended_at.duration_since(self.started_at)
    }

    /// Percentage of letters that were typed correctly.
    pub fn accuracy(&self) -> f64 {
        self.num_correct_chars() as f64 / self.num_chars_typed as f64
    }

    /// Speed in (correctly typed) characters per minute.
    pub fn cpm(&self) -> f64 {
        self.num_correct_chars() as f64 / (self.duration().as_secs_f64() / 60.0)
    }

    /// Speed in (correctly typed) words per minute.
    ///
    /// Measured as `cpm / (chars per word)` where `chars per word` is
    /// measured as `(number of chars) / (number of words)`.
    ///
    /// Note: this is only an approximation because "correctly typed
    /// words" is ambiguous when there could be a mistake in only one or
    /// two letters of a word.
    pub fn wpm(&self) -> f64 {
        self.cpm() / (self.num_chars_text as f64 / self.num_words as f64)
    }
}
