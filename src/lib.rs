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
use wordlists::{BuiltInWordlist, OS_WORDLIST_PATH};

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

impl std::fmt::Display for ToipeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.msg.as_str())
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
            if let Some(wordlist_path) = config.wordlist_file.clone() {
                Box::new(RawWordSelector::from_path(PathBuf::from(wordlist_path))?)
            } else if let Some(word_list) = config.wordlist.contents() {
                Box::new(RawWordSelector::from_string(word_list.to_string())?)
            } else if let BuiltInWordlist::OS = config.wordlist {
                Box::new(RawWordSelector::from_path(PathBuf::from(OS_WORDLIST_PATH))?)
            } else {
                // this should never happen!
                // TODO: somehow enforce this at compile time?
                return Err(ToipeError {
                    msg: "Undefined word list or path.".to_string(),
                });
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

        self.tui.display_lines_bottom(&[&[
            Text::from("ctrl-r").with_color(color::Blue),
            Text::from(" to restart, ").with_faint(),
            Text::from("ctrl-c").with_color(color::Blue),
            Text::from(" to quit ").with_faint(),
        ]])?;

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
        let original_text = self
            .text
            .iter()
            .fold(Vec::<char>::new(), |mut chars, text| {
                chars.extend(text.text().chars());
                chars
            });
        let mut num_errors = 0;
        let mut num_chars_typed = 0;

        enum TestStatus {
            // last key press did not quit/restart - more keys to be entered
            NotDone,
            // last letter was typed
            Done,
            // user wants to quit test
            Quit,
            // user wants to restart test
            Restart,
        }

        impl TestStatus {
            fn to_process_more_keys(&self) -> bool {
                matches!(self, TestStatus::NotDone)
            }

            fn to_display_results(&self) -> bool {
                matches!(self, TestStatus::Done)
            }

            fn to_restart(&self) -> bool {
                matches!(self, TestStatus::Restart)
            }
        }

        let mut process_key = |key: Key| -> Result<TestStatus, ToipeError> {
            match key {
                Key::Ctrl('c') => {
                    return Ok(TestStatus::Quit);
                }
                Key::Ctrl('r') => {
                    return Ok(TestStatus::Restart);
                }
                Key::Ctrl('w') => {
                    // delete last word
                    while !matches!(input.last(), Some(' ') | None) {
                        if input.pop().is_some() {
                            self.tui.replace_text(
                                Text::from(original_text[input.len()]).with_faint(),
                            )?;
                        }
                    }
                }
                Key::Char(c) => {
                    input.push(c);

                    if input.len() >= original_text.len() {
                        return Ok(TestStatus::Done);
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
                    if input.pop().is_some() {
                        self.tui
                            .replace_text(Text::from(original_text[input.len()]).with_faint())?;
                    }
                }
                _ => {}
            }

            self.tui.flush()?;

            Ok(TestStatus::NotDone)
        };

        let mut keys = stdin.keys();

        // read first key
        let key = keys.next().unwrap()?;
        // start the timer
        let started_at = Instant::now();
        // process first key
        let mut status = process_key(key)?;

        if status.to_process_more_keys() {
            for key in &mut keys {
                status = process_key(key?)?;
                if !status.to_process_more_keys() {
                    break;
                }
            }
        }

        // stop the timer
        let ended_at = Instant::now();

        let (final_chars_typed_correctly, final_uncorrected_errors) =
            input.iter().zip(original_text.iter()).fold(
                (0, 0),
                |(total_chars_typed_correctly, total_uncorrected_errors),
                 (typed_char, orig_char)| {
                    if typed_char == orig_char {
                        (total_chars_typed_correctly + 1, total_uncorrected_errors)
                    } else {
                        (total_chars_typed_correctly, total_uncorrected_errors + 1)
                    }
                },
            );

        let results = ToipeResults {
            total_words: self.words.len(),
            total_chars_typed: num_chars_typed,
            total_chars_in_text: input.len(),
            total_char_errors: num_errors,
            final_chars_typed_correctly,
            final_uncorrected_errors,
            started_at,
            ended_at,
        };

        let to_restart = if status.to_display_results() {
            self.display_results(results.clone(), keys)?
        } else {
            status.to_restart()
        };

        Ok((to_restart, results))
    }

    fn display_results(
        &mut self,
        results: ToipeResults,
        mut keys: Keys<StdinLock>,
    ) -> Result<bool, ToipeError> {
        self.tui.reset_screen()?;

        self.tui.display_lines::<&[Text], _>(&[
            &[Text::from(format!(
                "Took {}s for {} words of {}",
                results.duration().as_secs(),
                results.total_words,
                self.config.text_name(),
            ))],
            &[
                Text::from(format!("Accuracy: {:.1}%", results.accuracy() * 100.0))
                    .with_color(color::Blue),
            ],
            &[Text::from(format!(
                "Mistakes: {} out of {} characters",
                results.total_char_errors, results.total_chars_in_text
            ))],
            &[
                Text::from("Speed: "),
                Text::from(format!("{:.1} wpm", results.wpm())).with_color(color::Green),
                Text::from(" (words per minute)"),
            ],
        ])?;
        self.tui.display_lines_bottom(&[&[
            Text::from("ctrl-r").with_color(color::Blue),
            Text::from(" to restart, ").with_faint(),
            Text::from("ctrl-c").with_color(color::Blue),
            Text::from(" to quit ").with_faint(),
        ]])?;
        // no cursor on results page
        self.tui.hide_cursor()?;

        // TODO: make this a bit more general
        // perhaps use a `known_keys_pressed` flag?
        let mut to_restart: Option<bool> = None;
        while to_restart.is_none() {
            match keys.next().unwrap()? {
                // press 'r' to restart
                Key::Ctrl('r') => to_restart = Some(true),
                // press 'q' to quit
                Key::Ctrl('c') => to_restart = Some(false),
                _ => {}
            }
        }

        self.tui.show_cursor()?;

        Ok(to_restart.unwrap_or(false))
    }
}
