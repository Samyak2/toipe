const TOP_250: &'static str = include_str!("word_lists/top250");

pub const OS_WORDLIST_PATH: &str = "/usr/share/dict/words";

pub fn get_word_list(name: &str) -> Option<&'static str> {
    match name {
        "top250" => Some(TOP_250),
        _ => None,
    }
}
