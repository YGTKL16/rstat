mod cli;
mod io;
mod render;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Commands};

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Summary(args) => crate::io::run_summary(args),
    }
}
