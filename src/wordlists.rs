//! Built-in wordlists, system wordlist and utils for retrieving them.

/// Includes the Top 250 English words (by frequency of use).
/// [Source](https://www.wordfrequency.info/samples.asp)
/// (top 60K lemmas sample).
///
/// This static str has the contents of the word list file.
pub const TOP_250: &str = include_str!("word_lists/top250");

/// Path to the default word list file in Linux/Unix-based systems.
///
/// Note: the OS word list varies a lot from system to system and usually
/// has more than 100,000 words. This can lead to difficult and esoteric
/// words appearing in the test, reducing your typing speed.
pub const OS_WORDLIST_PATH: &str = "/usr/share/dict/words";

/// Get word list string by name.
///
/// Returns a static string containing the contents of the required word
/// list file. If the given name does not match any in-built word list,
/// returns [`None`].
pub fn get_word_list(name: &str) -> Option<&'static str> {
    match name {
        "top250" => Some(TOP_250),
        _ => None,
    }
}
