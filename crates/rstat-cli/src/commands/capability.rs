use crate::cli::CapabilityArgs;
use crate::io::read_column;
use crate::render::{OutputFormat, print_capability};
use anyhow::{Context, Result};
use rstat_core::capability::calculate_capability;

pub fn run(args: CapabilityArgs) -> Result<()> {
    let data = read_column(&args.file, &args.col).context("veri okunamadı")?;
    let stats = calculate_capability(&data, args.subgroup_size, args.lsl, args.usl, args.target)
        .context("Proses yeterlilik analizi hesaplanamadı")?;
    let fmt = OutputFormat::detect(args.format.as_deref());
    print_capability(&stats, fmt);
    Ok(())
}
