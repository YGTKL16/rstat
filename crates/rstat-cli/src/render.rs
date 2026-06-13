use comfy_table::{Table, presets::UTF8_BORDERS_ONLY};
use rstat_core::result::{SummaryStats, TTestResult};

pub enum OutputFormat {
    Table,
    Json,
    Csv,
}

impl OutputFormat {
    /// TTY'de tablo, pipe'ta JSON
    pub fn detect(flag: Option<&str>) -> Self {
        match flag {
            Some("json") => Self::Json,
            Some("csv") => Self::Csv,
            Some("table") => Self::Table,
            _ => {
                if atty::is(atty::Stream::Stdout) {
                    Self::Table
                } else {
                    Self::Json
                }
            }
        }
    }
}

pub fn print_ttest(r: &TTestResult, fmt: OutputFormat) {
    match fmt {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(r).unwrap());
        }
        OutputFormat::Csv => {
            println!("test,method,alternative,statistic,df,p_value,mean_diff,ci_low,ci_high,cohens_d,reject_null");
            let ci_low = r.ci.map(|c| c[0].to_string()).unwrap_or_default();
            let ci_high = r.ci.map(|c| c[1].to_string()).unwrap_or_default();
            println!(
                "{},{},{},{:.10},{:.6},{:.10},{},{},{},{},{}",
                r.test, r.method, r.alternative,
                r.statistic, r.df, r.p_value,
                r.mean_diff.map(|v| format!("{v:.10}")).unwrap_or_default(),
                ci_low, ci_high,
                r.cohens_d.map(|v| format!("{v:.10}")).unwrap_or_default(),
                r.reject_null,
            );
        }
        OutputFormat::Table => {
            // Grup tablosu
            let mut grp = Table::new();
            grp.load_preset(UTF8_BORDERS_ONLY);
            grp.set_header(vec!["Grup", "n", "Mean", "Std"]);
            for g in &r.groups {
                grp.add_row(vec![
                    g.name.clone(),
                    g.n.to_string(),
                    format!("{:.6}", g.mean),
                    format!("{:.6}", g.std),
                ]);
            }

            // Sonuç tablosu
            let mut tbl = Table::new();
            tbl.load_preset(UTF8_BORDERS_ONLY);
            tbl.set_header(vec!["", ""]);
            tbl.add_row(vec!["Test", r.test]);
            tbl.add_row(vec!["Yöntem", r.method]);
            tbl.add_row(vec!["Alternatif", &r.alternative]);
            tbl.add_row(vec!["t", &format!("{:.6}", r.statistic)]);
            tbl.add_row(vec!["df", &format!("{:.4}", r.df)]);
            tbl.add_row(vec!["p-değeri", &format!("{:.6e}", r.p_value)]);
            if let Some(d) = r.mean_diff {
                tbl.add_row(vec!["Ortalama fark", &format!("{:.6}", d)]);
            }
            if let Some(ci) = r.ci {
                tbl.add_row(vec![
                    &format!("%{:.0} GA", r.ci_level * 100.0),
                    &format!("[{:.6}, {:.6}]", ci[0], ci[1]),
                ]);
            }
            if let Some(d) = r.cohens_d {
                tbl.add_row(vec!["Cohen's d", &format!("{:.6}", d)]);
            }
            let karar = if r.reject_null {
                format!("REDDEDİLDİ (α={:.2})", r.alpha)
            } else {
                format!("REDDEDİLEMEDİ (α={:.2})", r.alpha)
            };
            tbl.add_row(vec!["H0", &karar]);

            println!("{grp}");
            println!("{tbl}");
        }
    }
}

pub fn print_summary(stats: &SummaryStats, fmt: OutputFormat) {
    match fmt {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(stats).unwrap());
        }
        OutputFormat::Csv => {
            println!("n,mean,std,min,q1,median,q3,max");
            println!(
                "{},{},{},{},{},{},{},{}",
                stats.n, stats.mean, stats.std, stats.min,
                stats.q1, stats.median, stats.q3, stats.max
            );
        }
        OutputFormat::Table => {
            let mut table = Table::new();
            table.load_preset(UTF8_BORDERS_ONLY);
            table.set_header(vec!["İstatistik", "Değer"]);
            table.add_row(vec!["n", &stats.n.to_string()]);
            table.add_row(vec!["mean", &format!("{:.6}", stats.mean)]);
            table.add_row(vec!["std", &format!("{:.6}", stats.std)]);
            table.add_row(vec!["min", &format!("{:.6}", stats.min)]);
            table.add_row(vec!["Q1", &format!("{:.6}", stats.q1)]);
            table.add_row(vec!["median", &format!("{:.6}", stats.median)]);
            table.add_row(vec!["Q3", &format!("{:.6}", stats.q3)]);
            table.add_row(vec!["max", &format!("{:.6}", stats.max)]);
            println!("{table}");
        }
    }
}
