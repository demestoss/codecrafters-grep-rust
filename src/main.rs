use std::process;

use clap::Parser;
use grepr::Command;

fn main() {
    let command = Command::parse();
    match command.invoke() {
        Ok(code) => process::exit(code),
        Err(e) => {
            eprintln!("{e}");
            process::exit(1)
        }
    }
}
