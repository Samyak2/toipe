extern crate termion;

use std::io::stdin;
use toipe::Toipe;

fn main() {
    let toipe = Toipe::new().unwrap();

    let stdin = stdin();
    let stdin = stdin.lock();

    toipe.test(stdin).unwrap();
}
