mod app;
mod audio;
mod cli;
mod tap;
mod tempo;
mod ui;

use crate::cli::Cli;
use clap::Parser;

fn main() {
    println!("v0.1.2");
    let cli = Cli::parse();
    app::run(cli);
}
