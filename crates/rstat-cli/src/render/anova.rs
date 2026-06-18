use super::OutputFormat;
use comfy_table::{Table, presets::UTF8_BORDERS_ONLY};
use rstat_core::result::AnovaResult;

pub fn print_anova(r: &AnovaResult, fmt: OutputFormat) {
    match fmt {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(r).unwrap());
        }
        OutputFormat::Csv => {
            println!("source,ss,df,ms,f,p_value,eta_squared,reject_null");
            println!(
                "between,{:.10},{:.0},{:.10},{:.10},{:.10},{:.10},{}",
                r.ss_between,
                r.df_between,
                r.ms_between,
                r.f_statistic,
                r.p_value,
                r.eta_squared,
                r.reject_null
            );
            println!(
                "within,{:.10},{:.0},{:.10},,,,,",
                r.ss_within, r.df_within, r.ms_within
            );
            let df_total = r.df_between + r.df_within;
            println!("total,{:.10},{:.0},,,,,,", r.ss_total, df_total);
        }
        OutputFormat::Table => {
            // Grup özetleri tablosu
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
            println!("One-Way ANOVA\n");
            println!("{grp}\n");

            // ANOVA tablosu
            let mut tbl = Table::new();
            tbl.load_preset(UTF8_BORDERS_ONLY);
            tbl.set_header(vec!["Kaynak", "SS", "df", "MS", "F", "p-değeri"]);
            tbl.add_row(vec![
                "Gruplar".to_string(),
                format!("{:.3}", r.ss_between),
                format!("{:.0}", r.df_between),
                format!("{:.3}", r.ms_between),
                format!("{:.3}", r.f_statistic),
                format!("{:.4e}", r.p_value),
            ]);
            tbl.add_row(vec![
                "Hata".to_string(),
                format!("{:.3}", r.ss_within),
                format!("{:.0}", r.df_within),
                format!("{:.3}", r.ms_within),
                "".to_string(),
                "".to_string(),
            ]);
            let df_total = r.df_between + r.df_within;
            tbl.add_row(vec![
                "Toplam".to_string(),
                format!("{:.3}", r.ss_total),
                format!("{:.0}", df_total),
                "".to_string(),
                "".to_string(),
                "".to_string(),
            ]);
            println!("{tbl}");

            // Omega-squared hesabı
            let omega_squared = {
                let num = r.ss_between - r.df_between * r.ms_within;
                let den = r.ss_total + r.ms_within;
                if den == 0.0 {
                    0.0
                } else {
                    (num / den).max(0.0)
                }
            };

            let karar = if r.reject_null {
                format!("REDDEDİLDİ (α={:.2})", r.alpha)
            } else {
                format!("REDDEDİLEMEDİ (α={:.2})", r.alpha)
            };
            println!(
                "\nη²={:.3}  ω²={:.3}  |  H0: {}",
                r.eta_squared, omega_squared, karar
            );
        }
    }
}
