//! Configuration for Toipe.
//!
//! Designed for command-line arguments using [`clap`], but can be used
//! as a library too.

use clap::Parser;

const CLI_HELP: &str = "A trusty terminal typing tester.

Keyboard shortcuts:
ctrl-c: quit
ctrl-r: restart test with a new set of words
ctrc-w: delete last word
";

/// Main configuration for Toipe.
#[derive(Parser)]
#[clap(author, version, about = CLI_HELP)]
pub struct ToipeConfig {
    /// Word list name or path to word list file.
    /// Available word lists:
    /// top250,
    /// os
    #[clap(short, long, default_value_t = String::from("top250"))]
    pub wordlist: String,
    /// Number of words to show on each test.
    #[clap(short, long, default_value_t = 30)]
    pub num_words: usize,
}
