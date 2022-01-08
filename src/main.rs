extern crate termion;

use std::io::stdin;
use toipe::Toipe;


fn main() {
    let mut toipe = Toipe::new();

    toipe.start().unwrap();

    let stdin = stdin();
    let stdin = stdin.lock();

    toipe.test(stdin).unwrap();
}
