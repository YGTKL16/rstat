use crate::cli::SummaryArgs;
use crate::io::read_column;
use crate::render::{OutputFormat, print_summary};
use anyhow::{Context, Result};
use rstat_core::data::summary::summary;

pub fn run(args: SummaryArgs) -> Result<()> {
    let data = read_column(&args.file, &args.col)?;
    let stats = summary(&data).context("özet istatistik hesaplanamadı")?;
    let fmt = OutputFormat::detect(args.format.as_deref());
    print_summary(&stats, fmt);
    Ok(())
}
