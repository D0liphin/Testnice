mod cli;
mod command;
mod log;
mod nix_ext;
mod tui;
mod util;

use clap::Parser;
use cli::Cli;
use command::Exec;

fn main() {
    let cli = Cli::parse();
    let result = match cli.command {
        cli::Command::Flood(command) => command.exec(),
        cli::Command::Tui(command) => command.exec(),
    };
    if let Err(e) = result {
        println!("{}", format_err!("{e}"));
        return;
    }
}
