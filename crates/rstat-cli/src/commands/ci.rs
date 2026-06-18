use crate::cli::CiArgs;
use crate::io::read_column;
use crate::render::{OutputFormat, print_mean_ci, print_proportion_ci, print_variance_ci};
use anyhow::{Result, bail};
use rstat_core::interval::ci;

pub fn run(args: CiArgs) -> Result<()> {
    let fmt = OutputFormat::detect(args.format.as_deref());

    match args.ci_type.as_str() {
        "mean" => {
            let data = read_column(&args.file, &args.col)?;
            let result = ci::mean_ci(&data, args.level).map_err(|e| anyhow::anyhow!("{e}"))?;
            print_mean_ci(&result, fmt);
        }
        "variance" => {
            let data = read_column(&args.file, &args.col)?;
            let result = ci::variance_ci(&data, args.level).map_err(|e| anyhow::anyhow!("{e}"))?;
            print_variance_ci(&result, fmt);
        }
        "proportion" => {
            let (successes, trials) = if let (Some(s), Some(t)) = (args.successes, args.trials) {
                (s, t)
            } else {
                let data = read_column(&args.file, &args.col)?;
                let t = data.len() as u64;
                let s = data.iter().filter(|&&x| x == 1.0).count() as u64;
                (s, t)
            };

            let result = ci::proportion_ci(successes, trials, args.level, &args.method)
                .map_err(|e| anyhow::anyhow!("{e}"))?;
            print_proportion_ci(&result, fmt);
        }
        other => bail!("bilinmeyen güven aralığı türü: {other}"),
    }

    Ok(())
}
