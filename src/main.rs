use clap::StructOpt;
use std::io::stdin;
use toipe::config::ToipeConfig;
use toipe::Toipe;

fn main() {
    let config = ToipeConfig::parse();

    let res = Toipe::new(config);
    if let Err(e) = res {
        eprintln!("{}", e);
        std::process::exit(1);
    }

    let mut toipe = res.unwrap();

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
