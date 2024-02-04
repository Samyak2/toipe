use std::path::PathBuf;

use toipe::{
    word_selector::{ascii_raw::AsciiSortedWordSelector, WordSelector},
    wordlists::OS_WORDLIST_PATH,
};

fn main() {
    let mut word_selector =
        AsciiSortedWordSelector::from_path(PathBuf::from(OS_WORDLIST_PATH)).unwrap();

    let word = word_selector.new_word().unwrap();
    println!("{}", word);
}
