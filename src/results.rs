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
        if self.total_chars_typed == 0 {
            return 0.0;
        }

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
            .max(0.0)
            / (self.duration().as_secs_f64() / 60.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_ulps_eq;

    #[test]
    fn sanity() {
        let started_at = Instant::now();
        let ended_at = started_at + Duration::new(10, 0);
        let results = ToipeResults {
            total_words: 0,
            total_chars_typed: 100,
            total_chars_in_text: 120,
            total_char_errors: 10,
            final_chars_typed_correctly: 80,
            final_uncorrected_errors: 2,
            started_at,
            ended_at,
        };

        assert_eq!(results.duration(), Duration::new(10, 0));

        assert_ulps_eq!(results.accuracy(), 0.9, max_ulps = 1);
        assert_ulps_eq!(results.wpm(), 84.0, max_ulps = 1);
    }

    #[test]
    fn accuracy() {
        fn get_toipe_results(total_chars_typed: usize, total_char_errors: usize) -> ToipeResults {
            ToipeResults {
                total_words: 0,
                total_chars_typed,
                total_chars_in_text: 0,
                total_char_errors,
                final_chars_typed_correctly: 0,
                final_uncorrected_errors: 0,
                started_at: Instant::now(),
                ended_at: Instant::now(),
            }
        }

        let max_ulps = 1;

        // no errors
        assert_ulps_eq!(
            get_toipe_results(100, 0).accuracy(),
            1.0,
            max_ulps = max_ulps
        );
        // nothing typed
        assert_ulps_eq!(get_toipe_results(0, 0).accuracy(), 0.0, max_ulps = max_ulps);
        // all wrong
        assert_ulps_eq!(
            get_toipe_results(100, 100).accuracy(),
            0.0,
            max_ulps = max_ulps
        );
        // half correct
        assert_ulps_eq!(
            get_toipe_results(100, 50).accuracy(),
            0.5,
            max_ulps = max_ulps
        );
        // more errors than correct
        assert_ulps_eq!(
            get_toipe_results(100, 150).accuracy(),
            -0.5,
            max_ulps = max_ulps
        );
    }

    #[test]
    fn wpm() {
        fn get_toipe_results(
            final_chars_typed_correctly: usize,
            final_uncorrected_errors: usize,
            duration: f64,
        ) -> ToipeResults {
            let started_at = Instant::now();
            let seconds = duration.round();
            let nanoseconds = (duration - seconds) * 1_000_000_000.0;
            let ended_at = started_at + Duration::new(seconds as u64, nanoseconds as u32);
            ToipeResults {
                total_words: 0,
                total_chars_typed: 0,
                total_chars_in_text: 0,
                total_char_errors: 0,
                final_chars_typed_correctly,
                final_uncorrected_errors,
                started_at,
                ended_at,
            }
        }

        let max_ulps = 1;
        assert_ulps_eq!(
            get_toipe_results(100, 5, 30.0).wpm(),
            30.0,
            max_ulps = max_ulps
        );
        assert_ulps_eq!(
            get_toipe_results(1000, 50, 30.0).wpm(),
            300.0,
            max_ulps = max_ulps
        );
        assert_ulps_eq!(
            get_toipe_results(200, 0, 30.0).wpm(),
            80.0,
            max_ulps = max_ulps
        );
        assert_ulps_eq!(
            get_toipe_results(200, 30, 30.0).wpm(),
            20.0,
            max_ulps = max_ulps
        );
        // too many errors - cancels out
        assert_ulps_eq!(
            get_toipe_results(200, 40, 30.0).wpm(),
            0.0,
            max_ulps = max_ulps
        );
        // no negative wpms
        assert_ulps_eq!(
            get_toipe_results(200, 50, 30.0).wpm(),
            0.0,
            max_ulps = max_ulps
        );
        assert_ulps_eq!(
            get_toipe_results(1, 0, 1.0).wpm(),
            12.0,
            max_ulps = max_ulps
        );
        // skdlhaslkd won't give you any score!
        assert_ulps_eq!(
            get_toipe_results(0, 10, 1.0).wpm(),
            0.0,
            max_ulps = max_ulps
        );
        assert_ulps_eq!(
            get_toipe_results(0, 0, 0.01).wpm(),
            0.0,
            max_ulps = max_ulps
        );
        // we don't consider the case of duration = 0 because that seems impossible
    }
}
