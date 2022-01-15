use clap::Parser;

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
