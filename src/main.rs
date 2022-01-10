extern crate termion;

use std::io::stdin;
use clap::StructOpt;
use toipe::Toipe;
use toipe::config::ToipeConfig;

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
