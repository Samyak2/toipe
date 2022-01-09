extern crate termion;

use toipe::textgen::WordSelector;

fn main() {
    let word_selector = WordSelector::default();
    // println!("{:?}", word_selector);

    let word = word_selector.new_word().unwrap();
    println!("{}", word);
}
