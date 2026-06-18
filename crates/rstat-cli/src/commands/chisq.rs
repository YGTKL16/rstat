use anyhow::{Context, Result};
use rstat_core::tests_stat::chisq;

use crate::cli::ChisqArgs;
use crate::io::{read_column, read_contingency_table_long, read_matrix};
use crate::render::{OutputFormat, print_chisq};

pub fn run(args: ChisqArgs) -> Result<()> {
    let result = match args.kind.as_str() {
        "independence" => {
            let table = match (&args.col1, &args.col2) {
                (Some(c1), Some(c2)) => read_contingency_table_long(&args.file, c1, c2)?,
                (None, None) => read_matrix(&args.file)?,
                _ => {
                    anyhow::bail!(
                        "Bağımsızlık testi uzun formatı için hem --col1 hem de --col2 belirtilmeli, geniş matris formatı için ikisi de boş bırakılmalı"
                    );
                }
            };

            chisq::independence_test(&table, !args.no_yates, args.alpha)
                .map_err(|e| anyhow::anyhow!("{e}"))?
        }
        "gof" => {
            let c1 = args
                .col1
                .as_deref()
                .context("Uyum iyiliği testi için gözlenen kolon (--col1) belirtilmeli")?;
            let c2 = args
                .col2
                .as_deref()
                .context("Uyum iyiliği testi için beklenen kolon (--col2) belirtilmeli")?;

            let observed = read_column(&args.file, c1)?;
            let expected = read_column(&args.file, c2)?;

            chisq::goodness_of_fit_test(&observed, &expected, args.alpha)
                .map_err(|e| anyhow::anyhow!("{e}"))?
        }
        other => anyhow::bail!("bilinmeyen test türü: {other}"),
    };

    let fmt = OutputFormat::detect(args.format.as_deref());
    print_chisq(&result, fmt);
    Ok(())
}
