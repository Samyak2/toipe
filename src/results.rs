use std::time::{Duration, Instant};

// TODO: document these fields
/// Stores stats from a typing test.
#[derive(Clone)]
pub struct ToipeResults {
    pub num_words: usize,
    pub num_chars_typed: usize,
    pub num_chars_text: usize,
    pub num_errors: usize,
    pub started_at: Instant,
    pub ended_at: Instant,
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
