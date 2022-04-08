//! Built-in wordlists, system wordlist and utils for retrieving them.

use clap::ArgEnum;

/// Word lists with top English words (by frequency)
///
/// [Source](https://www.wordfrequency.info/samples.asp) (top 60K lemmas sample).
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ArgEnum)]
pub enum BuiltInWordlist {
    Top250,
    Top500,
    Top1000,
    Top2500,
    Top5000,
    // The operating system's builtin word list
    OS,
}

impl BuiltInWordlist {
    /// Contents of the word list as a static string.
    ///
    /// Note: BuiltInWordlist::OS returns a None since we only know the path of it.
    /// Reading the file can take time (and memory) as the file can be large.
    pub fn contents(&self) -> Option<&'static str> {
        match self {
            Self::Top250 => Some(include_str!("word_lists/top250")),
            Self::Top500 => Some(include_str!("word_lists/top500")),
            Self::Top1000 => Some(include_str!("word_lists/top1000")),
            Self::Top2500 => Some(include_str!("word_lists/top2500")),
            Self::Top5000 => Some(include_str!("word_lists/top5000")),
            Self::OS => None,
        }
    }
}

/// Path to the default word list file in Linux/Unix-based systems.
///
/// Note: the OS word list varies a lot from system to system and usually
/// has more than 100,000 words. This can lead to difficult and esoteric
/// words appearing in the test, reducing your typing speed.
pub const OS_WORDLIST_PATH: &str = "/usr/share/dict/words";
