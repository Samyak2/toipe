use toipe::book::BookSelector;
use toipe::textgen::WordSelector;

fn main() {
    let mut word_selector =
        BookSelector::from_string("This is a test.\nhello world!".to_string()).unwrap();

    let mut word = word_selector.new_word().unwrap();
    println!("{}", word);
    word = word_selector.new_word().unwrap();
    println!("{}", word);
}
