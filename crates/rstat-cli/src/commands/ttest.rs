use anyhow::{Context, Result};
use rstat_core::dist::pvalue::Alternative;
use rstat_core::tests_stat::ttest::{self, Method};

use crate::cli::TtestArgs;
use crate::io::{read_column, read_two_columns};
use crate::render::{OutputFormat, print_ttest};

pub fn run(args: TtestArgs) -> Result<()> {
    let alt = Alternative::parse(&args.alt)
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    let result = match args.kind.as_str() {
        "one" => {
            let x = read_column(&args.file, &args.col)?;
            ttest::one_sample(&x, args.mu, alt, args.ci_level, args.alpha)
                .map_err(|e| anyhow::anyhow!("{e}"))?
        }
        "two" => {
            let col2 = args.col2.as_deref().context("two-sample için --col2 gerekli")?;
            let (a, b) = read_two_columns(&args.file, &args.col, col2)?;
            let method = if args.var == "pooled" { Method::Pooled } else { Method::Welch };
            ttest::two_sample(&a, &b, method, alt, args.ci_level, args.alpha)
                .map_err(|e| anyhow::anyhow!("{e}"))?
        }
        "paired" => {
            let col2 = args.col2.as_deref().context("paired için --col2 gerekli")?;
            let (a, b) = read_two_columns(&args.file, &args.col, col2)?;
            ttest::paired(&a, &b, alt, args.ci_level, args.alpha)
                .map_err(|e| anyhow::anyhow!("{e}"))?
        }
        other => anyhow::bail!("bilinmeyen test türü: {other}"),
    };

    let fmt = OutputFormat::detect(args.format.as_deref());
    print_ttest(&result, fmt);
    Ok(())
}
