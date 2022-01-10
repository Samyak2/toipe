use clap::Parser;

#[derive(Parser)]
#[clap(author, version, about)]
pub struct ToipeConfig {
    /// path to file containing list of words
    #[clap(short, long, default_value_t = String::from("/usr/share/dict/words"))]
    pub wordlist_path: String,
}
