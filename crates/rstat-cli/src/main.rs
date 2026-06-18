mod cli;
mod commands;
mod io;
mod render;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Commands};

fn main() -> Result<()> {
    if rstat_license::try_load_license().is_some() {
        rstat_core::license::LICENSE_VERIFIED.store(true, std::sync::atomic::Ordering::Relaxed);
    }

    let cli = Cli::parse();
    match cli.command {
        Commands::Summary(args) => crate::commands::summary::run(args),
        Commands::Ttest(args) => crate::commands::ttest::run(args),
        Commands::Anova(args) => crate::commands::anova::run(args),
        Commands::Ci(args) => crate::commands::ci::run(args),
        Commands::Chisq(args) => crate::commands::chisq::run(args),
        Commands::Spc(args) => crate::commands::spc::run(args),
        Commands::Capability(args) => crate::commands::capability::run(args),
    }
}
