//! Configuration for Toipe.
//!
//! Designed for command-line arguments using [`clap`], but can be used
//! as a library too.

use clap::Parser;

/// Main configuration for Toipe.
#[derive(Parser)]
#[clap(author, version, about)]
pub struct ToipeConfig {
    /// word list name or path to word list file.
    /// Available word lists:
    /// top250,
    /// os
    #[clap(short, long, default_value_t = String::from("top250"))]
    pub wordlist: String,
}
