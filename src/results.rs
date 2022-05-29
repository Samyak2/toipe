use std::time::{Duration, Instant};

/// Stores stats from a typing test.
#[derive(Clone)]
pub struct ToipeResults {
    /// number of words in given text
    pub total_words: usize,
    /// number of chars typed including those typed before being cleared
    /// (by backspace or ctrl-w)
    pub total_chars_typed: usize,
    /// number of chars in given text
    pub total_chars_in_text: usize,
    /// number of wrongly typed characters including those that were cleared
    /// (by backspace or ctrl-w)
    pub total_char_errors: usize,
    /// number of chars in given text that were correctly typed at the end of the test
    pub final_chars_typed_correctly: usize,
    /// number of chars in given text that were wrongly typed at the end of the test
    pub final_uncorrected_errors: usize,
    pub started_at: Instant,
    pub ended_at: Instant,
}

impl ToipeResults {
    /// Duration of the test.
    ///
    /// i.e., the time between the user pressing the first key and them
    /// typing the last letter.
    pub fn duration(&self) -> Duration {
        self.ended_at.duration_since(self.started_at)
    }

    /// Percentage of letters that were typed correctly.
    pub fn accuracy(&self) -> f64 {
        (self.total_chars_typed as isize - self.total_char_errors as isize) as f64
            / self.total_chars_typed as f64
    }

    /// Speed in (correctly typed) words per minute.
    ///
    /// Measured as (number of correctly typed chars / 5 - number of uncorrected errors) / minute
    ///
    /// A "word" is considered to be 5 chars because:
    /// - chars/letters are typed, not whole words
    /// - a sentence with small words won't be disproportionately favoured
    ///
    /// Uncorrected errors are penalized to encourage correcting errors.
    pub fn wpm(&self) -> f64 {
        (self.final_chars_typed_correctly as f64 / 5.0 - self.final_uncorrected_errors as f64)
            .max(0.0) as f64
            / (self.duration().as_secs_f64() / 60.0)
    }
}
