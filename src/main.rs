extern crate termion;

use std::io::stdin;
use toipe::Toipe;

fn main() {
    let mut toipe = Toipe::new().unwrap();

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
