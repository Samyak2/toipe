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
pub mod results;
pub mod textgen;
pub mod tui;
pub mod wordlists;

use std::io::StdinLock;
use std::path::PathBuf;
use std::time::Instant;

use config::ToipeConfig;
use results::ToipeResults;
use termion::input::Keys;
use termion::{color, event::Key, input::TermRead};
use textgen::{RawWordSelector, WordSelector};
use tui::{Text, ToipeTui};
use wordlists::{get_word_list, OS_WORDLIST_PATH};

/// Typing test terminal UI and logic.
pub struct Toipe {
    tui: ToipeTui,
    text: Vec<Text>,
    words: Vec<String>,
    word_selector: Box<dyn WordSelector>,
    config: ToipeConfig,
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
    /// Initializes the word selector.
    /// Also invokes [`Toipe::restart()`].
    pub fn new(config: ToipeConfig) -> Result<Self, ToipeError> {
        let word_selector: Box<dyn WordSelector> =
            if let Some(word_list) = get_word_list(config.wordlist.as_str()) {
                Box::new(RawWordSelector::from_string(word_list.to_string())?)
            } else if config.wordlist == "os" {
                Box::new(RawWordSelector::from_path(PathBuf::from(OS_WORDLIST_PATH))?)
            } else {
                Box::new(RawWordSelector::from_path(PathBuf::from(
                    config.wordlist.clone(),
                ))?)
            };

        let mut toipe = Toipe {
            tui: ToipeTui::new(),
            words: Vec::new(),
            text: Vec::new(),
            word_selector,
            config,
        };

        toipe.restart()?;

        Ok(toipe)
    }

    /// Make the terminal ready for the next typing test.
    ///
    /// Clears the screen, generates new words and displays them on the
    /// UI.
    pub fn restart(&mut self) -> Result<(), ToipeError> {
        self.tui.reset_screen()?;

        self.words = self.word_selector.new_words(self.config.num_words)?;

        self.show_words()?;

        Ok(())
    }

    fn show_words(&mut self) -> Result<(), ToipeError> {
        self.text = self.tui.display_words(&self.words)?;
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
        let original_text = self.text.iter().fold(Vec::<char>::new(), |mut chars, text| {
            chars.extend(text.text().chars());
            chars
        });
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

                    if input.len() >= original_text.len() {
                        return Ok(false);
                    }

                    num_chars_typed += 1;

                    if original_text[input.len() - 1] == c {
                        self.tui
                            .display_raw_text(&Text::from(c).with_color(color::LightGreen))?;
                        self.tui.move_to_next_char()?;
                    } else {
                        self.tui.display_raw_text(
                            &Text::from(original_text[input.len() - 1])
                                .with_underline()
                                .with_color(color::Red),
                        )?;
                        self.tui.move_to_next_char()?;
                        num_errors += 1;
                    }
                }
                Key::Backspace => {
                    let last_char = input.pop();
                    if let Some(_) = last_char {
                        self.tui
                            .replace_text(Text::from(original_text[input.len()]).with_faint())?;
                    }
                }
                _ => {}
            }

            self.tui.flush()?;

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
        self.tui.reset_screen()?;

        self.tui.display_lines::<&[Text], _>(&[
            &[
                Text::from(format!("Accuracy: {:.1}%", results.accuracy() * 100.0))
                    .with_color(color::Blue),
            ],
            &[Text::from(format!(
                "Mistakes: {} out of {} characters",
                results.num_errors, results.num_chars_text
            ))],
            &[
                Text::from("Speed: "),
                Text::from(format!("{:.1} wpm", results.wpm())).with_color(color::Green),
                Text::from(" (words per minute)"),
            ],
            &[
                Text::from("Press "),
                Text::from("r").with_color(color::Blue),
                Text::from(" to restart, "),
                Text::from("q").with_color(color::Blue),
                Text::from(" to quit."),
            ],
        ])?;
        // no cursor on results page
        self.tui.hide_cursor()?;

        let mut to_restart = false;
        // press 'r' to restart
        match keys.next().unwrap()? {
            Key::Char('r') => to_restart = true,
            _ => {}
        }

        self.tui.show_cursor()?;

        Ok(to_restart)
    }
}
