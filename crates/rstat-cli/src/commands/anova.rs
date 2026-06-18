use crate::cli::AnovaArgs;
use crate::io::{read_long_groups, read_wide_groups};
use crate::render::{OutputFormat, print_anova};
use anyhow::{Result, bail};
use rstat_core::tests_stat::anova;

pub fn run(args: AnovaArgs) -> Result<()> {
    let groups: Vec<Vec<f64>> = if let Some(cols_spec) = &args.cols {
        read_wide_groups(&args.file, cols_spec)?
    } else if let (Some(val_col), Some(grp_col)) = (&args.value, &args.group) {
        let long_groups = read_long_groups(&args.file, val_col, grp_col)?;
        long_groups.into_iter().map(|(_, v)| v).collect()
    } else {
        bail!(
            "ANOVA için ya --cols (geniş format) ya da --value ve --group (uzun format) belirtilmelidir"
        );
    };

    let group_refs: Vec<&[f64]> = groups.iter().map(|g| g.as_slice()).collect();

    let result = anova::one_way(&group_refs, args.alpha).map_err(|e| anyhow::anyhow!("{e}"))?;

    let fmt = OutputFormat::detect(args.format.as_deref());
    print_anova(&result, fmt);
    Ok(())
}
