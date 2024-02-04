pub mod ascii_raw;

use std::io;

/// Describes a thing that provides new words.
pub trait WordSelector {
    /// Returns a new word.
    fn new_word(&mut self) -> Result<String, io::Error>;

    /// Returns a [`Vec`] containing `num_words` words.
    fn new_words(&mut self, num_words: usize) -> Result<Vec<String>, io::Error> {
        (0..num_words).map(|_| self.new_word()).collect()
    }
}
