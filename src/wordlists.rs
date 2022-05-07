//! Built-in wordlists, system wordlist and utils for retrieving them.

use clap::ArgEnum;

/// Word lists with top English words.
///
/// See [variants](#variants) for details on each word list.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ArgEnum)]
pub enum BuiltInWordlist {
    /// Source: [wordfrequency.info](https://www.wordfrequency.info/samples.asp) (top 60K lemmas sample).
    Top250,

    /// Source: [wordfrequency.info](https://www.wordfrequency.info/samples.asp) (top 60K lemmas sample).
    Top500,

    /// Source: [wordfrequency.info](https://www.wordfrequency.info/samples.asp) (top 60K lemmas sample).
    Top1000,

    /// Source: [wordfrequency.info](https://www.wordfrequency.info/samples.asp) (top 60K lemmas sample).
    Top2500,

    /// Source: [wordfrequency.info](https://www.wordfrequency.info/samples.asp) (top 60K lemmas sample).
    Top5000,

    /// Source: [Monkeytype](https://github.com/monkeytypegame/monkeytype/blob/89f160f664a9e24a6d5a99f12ce0bd5a1b093b2a/frontend/static/languages/english_10k.json)
    /// (English 10k list)
    Top10000,

    /// Source: [Monkeytype](https://github.com/monkeytypegame/monkeytype/blob/89f160f664a9e24a6d5a99f12ce0bd5a1b093b2a/frontend/static/languages/english_25k.json)
    /// (English 25k list)
    Top25000,

    /// Source: [Monkeytype](https://github.com/monkeytypegame/monkeytype/blob/89f160f664a9e24a6d5a99f12ce0bd5a1b093b2a/frontend/static/languages/english_commonly_misspelled.json)
    /// (Commonly misspelled English list)
    CommonlyMisspelled,

    /// The operating system's builtin word list.
    ///
    /// See [`OS_WORDLIST_PATH`].
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
            Self::Top10000 => Some(include_str!("word_lists/top10000")),
            Self::Top25000 => Some(include_str!("word_lists/top25000")),
            Self::CommonlyMisspelled => Some(include_str!("word_lists/commonly_misspelled")),
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
