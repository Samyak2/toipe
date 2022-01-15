extern crate termion;

use clap::StructOpt;
use std::io::stdin;
use toipe::config::ToipeConfig;
use toipe::Toipe;

fn main() {
    let config = ToipeConfig::parse();

    let mut toipe = Toipe::new(config).unwrap();

    let stdin = stdin();

    loop {
        let stdin = stdin.lock();
        if let Ok((true, _)) = toipe.test(stdin) {
            toipe.restart().unwrap();
        } else {
            break;
        }
    }
}
