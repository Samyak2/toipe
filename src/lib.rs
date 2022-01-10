pub mod textgen;

use std::io::{stdout, StdinLock, Stdout, Write};
use std::time::{Duration, Instant};

use termion::input::Keys;
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
    words: Vec<String>,
    word_selector: WordSelector,
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

        let word_selector = WordSelector::default();

        let mut toipe = Toipe {
            stdout,
            text: "".to_string(),
            words: Vec::new(),
            word_selector,
        };

        toipe.restart()?;

        Ok(toipe)
    }

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

    pub fn flush(&mut self) -> Result<(), ToipeError> {
        self.stdout.flush()?;
        Ok(())
    }

    pub fn test(&mut self, stdin: StdinLock<'a>) -> Result<(bool, ToipeResults), ToipeError> {
        let mut input = Vec::<char>::new();
        let text: Vec<char> = self.text.chars().collect();
        let mut num_errors = 0;

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

        let key = keys.next().unwrap()?;

        let started_at = Instant::now();

        let res = process_key(key)?;

        if res {
            for key in &mut keys {
                let res = process_key(key?)?;
                if !res {
                    break;
                }
            }
        }

        let ended_at = Instant::now();

        let results = ToipeResults {
            num_chars: input.len(),
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
            results.num_errors, results.num_chars
        );
        write!(
            self.stdout,
            "{}{}{}",
            cursor::Goto(sizex / 2, sizey / 2 - 1),
            cursor::Left(line.len() as u16 / 2),
            line,
        )?;

        let line = format!(
            "Speed: {}{:.1} cpm{} (characters per minute)",
            color::Fg(color::Green),
            results.cpm(),
            color::Fg(color::Reset)
        );
        let zerowidths = format!("{}{}", color::Fg(color::Green), color::Fg(color::Reset));
        write!(
            self.stdout,
            "{}{}{}",
            cursor::Goto(sizex / 2, sizey / 2),
            cursor::Left((line.len() - zerowidths.len()) as u16 / 2),
            line,
        )?;

        write!(self.stdout, "{}", cursor::Hide)?;
        self.flush()?;

        let mut to_restart = false;

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

#[derive(Clone)]
pub struct ToipeResults {
    num_chars: usize,
    num_errors: usize,
    started_at: Instant,
    ended_at: Instant,
}

impl ToipeResults {
    fn duration(&self) -> Duration {
        self.ended_at.duration_since(self.started_at)
    }

    pub fn accuracy(&self) -> f64 {
        1.0 - (self.num_errors as f64 / self.num_chars as f64)
    }

    pub fn cpm(&self) -> f64 {
        self.num_chars as f64 / (self.duration().as_secs_f64() / 60.0)
    }
}
