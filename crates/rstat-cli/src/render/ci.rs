use super::OutputFormat;
use comfy_table::{Table, presets::UTF8_BORDERS_ONLY};
use rstat_core::result::{MeanCiResult, ProportionCiResult, VarianceCiResult};

pub fn print_mean_ci(r: &MeanCiResult, fmt: OutputFormat) {
    match fmt {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(r).unwrap());
        }
        OutputFormat::Csv => {
            println!("n,mean,std,se,df,ci_level,ci_low,ci_high");
            println!(
                "{},{:.10},{:.10},{:.10},{:.10},{:.10},{:.10},{:.10}",
                r.n, r.mean, r.std, r.se, r.df, r.ci_level, r.ci[0], r.ci[1]
            );
        }
        OutputFormat::Table => {
            let mut tbl = Table::new();
            tbl.load_preset(UTF8_BORDERS_ONLY);
            tbl.set_header(vec!["İstatistik", "Değer"]);
            tbl.add_row(vec!["Tür", "Ortalama Güven Aralığı"]);
            tbl.add_row(vec!["n", &r.n.to_string()]);
            tbl.add_row(vec!["mean", &format!("{:.6}", r.mean)]);
            tbl.add_row(vec!["std", &format!("{:.6}", r.std)]);
            tbl.add_row(vec!["se", &format!("{:.6}", r.se)]);
            tbl.add_row(vec!["df", &format!("{:.1}", r.df)]);
            tbl.add_row(vec!["Güven Düzeyi", &format!("%{:.0}", r.ci_level * 100.0)]);
            tbl.add_row(vec![
                "Güven Aralığı",
                &format!("[{:.6}, {:.6}]", r.ci[0], r.ci[1]),
            ]);
            println!("{tbl}");
        }
    }
}

pub fn print_proportion_ci(r: &ProportionCiResult, fmt: OutputFormat) {
    match fmt {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(r).unwrap());
        }
        OutputFormat::Csv => {
            println!("successes,trials,p_hat,ci_level,method,ci_low,ci_high");
            println!(
                "{},{},{:.10},{:.10},{},{:.10},{:.10}",
                r.successes, r.trials, r.p_hat, r.ci_level, r.method, r.ci[0], r.ci[1]
            );
        }
        OutputFormat::Table => {
            let mut tbl = Table::new();
            tbl.load_preset(UTF8_BORDERS_ONLY);
            tbl.set_header(vec!["İstatistik", "Değer"]);
            tbl.add_row(vec!["Tür", "Oran Güven Aralığı"]);
            tbl.add_row(vec!["başarı (x)", &r.successes.to_string()]);
            tbl.add_row(vec!["deneme (n)", &r.trials.to_string()]);
            tbl.add_row(vec!["p_hat (oran)", &format!("{:.6}", r.p_hat)]);
            tbl.add_row(vec!["Yöntem", &r.method]);
            tbl.add_row(vec!["Güven Düzeyi", &format!("%{:.0}", r.ci_level * 100.0)]);
            tbl.add_row(vec![
                "Güven Aralığı",
                &format!("[{:.6}, {:.6}]", r.ci[0], r.ci[1]),
            ]);
            println!("{tbl}");
        }
    }
}

pub fn print_variance_ci(r: &VarianceCiResult, fmt: OutputFormat) {
    match fmt {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(r).unwrap());
        }
        OutputFormat::Csv => {
            println!(
                "n,variance,std_dev,df,ci_level,ci_var_low,ci_var_high,ci_std_low,ci_std_high"
            );
            println!(
                "{},{:.10},{:.10},{:.10},{:.10},{:.10},{:.10},{:.10},{:.10}",
                r.n,
                r.variance,
                r.std_dev,
                r.df,
                r.ci_level,
                r.ci_variance[0],
                r.ci_variance[1],
                r.ci_std_dev[0],
                r.ci_std_dev[1]
            );
        }
        OutputFormat::Table => {
            let mut tbl = Table::new();
            tbl.load_preset(UTF8_BORDERS_ONLY);
            tbl.set_header(vec!["İstatistik", "Değer"]);
            tbl.add_row(vec!["Tür", "Varyans & Std Sapma Güven Aralığı"]);
            tbl.add_row(vec!["n", &r.n.to_string()]);
            tbl.add_row(vec!["varyans (s²)", &format!("{:.6}", r.variance)]);
            tbl.add_row(vec!["std sapma (s)", &format!("{:.6}", r.std_dev)]);
            tbl.add_row(vec!["df", &format!("{:.1}", r.df)]);
            tbl.add_row(vec!["Güven Düzeyi", &format!("%{:.0}", r.ci_level * 100.0)]);
            tbl.add_row(vec![
                "Varyans GA",
                &format!("[{:.6}, {:.6}]", r.ci_variance[0], r.ci_variance[1]),
            ]);
            tbl.add_row(vec![
                "Std Sapma GA",
                &format!("[{:.6}, {:.6}]", r.ci_std_dev[0], r.ci_std_dev[1]),
            ]);
            println!("{tbl}");
        }
    }
}
