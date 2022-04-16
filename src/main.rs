use clap::StructOpt;
use std::io::stdin;
use toipe::config::ToipeConfig;
use toipe::Toipe;
use toipe::ToipeError;

fn main() -> Result<(), ToipeError> {
    let config = ToipeConfig::parse();

    let mut toipe = Toipe::new(config)?;

    let stdin = stdin();

    loop {
        let stdin = stdin.lock();
        if let Ok((true, _)) = toipe.test(stdin) {
            toipe.restart().unwrap();
        } else {
            break;
        }
    }
    Ok(())
}
