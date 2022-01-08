use std::io::{stdout, StdinLock, Stdout, Write};

use termion::{
    clear, cursor,
    event::Key::{self, Ctrl},
    input::TermRead,
    raw::{IntoRawMode, RawTerminal},
    terminal_size,
};

pub struct Toipe {
    stdout: RawTerminal<Stdout>,
}

#[derive(Debug)]
pub struct ToipeError {
    msg: String,
}

impl From<std::io::Error> for ToipeError {
    fn from(error: std::io::Error) -> Self {
        ToipeError {
            msg: error.to_string(),
        }
    }
}

impl<'a> Toipe {
    pub fn new() -> Self {
        let stdout = stdout().into_raw_mode().unwrap();

        Toipe { stdout }
    }

    pub fn start(&mut self) -> Result<(), ToipeError> {
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

    pub fn flush(&mut self) -> Result<(), ToipeError> {
        self.stdout.flush()?;
        Ok(())
    }

    pub fn test(mut self, stdin: StdinLock<'a>) -> Result<(), ToipeError> {
        for key in stdin.keys() {
            let key = key?;

            match key {
                Ctrl(c) => match c {
                    'c' => {
                        break;
                    }
                    _ => {}
                },
                Key::Char(c) => {
                    write!(self.stdout, "{}", c)?;
                }
                _ => {}
            }

            // write!(self.stdout, "{:?}", key)?;
            self.flush()?;
        }

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
