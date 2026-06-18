use crate::cli::SpcArgs;
use crate::io::read_column;
use crate::render::{OutputFormat, print_spc};
use anyhow::{Context, Result};
use rstat_core::spc::calculate_xbar_r;

pub fn run(args: SpcArgs) -> Result<()> {
    let data = read_column(&args.file, &args.col).context("veri okunamadı")?;
    let stats = calculate_xbar_r(&data, args.subgroup_size).context("SPC hesaplanamadı")?;
    let fmt = OutputFormat::detect(args.format.as_deref());
    print_spc(&stats, fmt);
    Ok(())
}
