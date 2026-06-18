use super::OutputFormat;
use comfy_table::{Table, presets::UTF8_BORDERS_ONLY};
use rstat_core::result::ChiSqResult;

pub fn print_chisq(r: &ChiSqResult, fmt: OutputFormat) {
    match fmt {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(r).unwrap());
        }
        OutputFormat::Csv => {
            println!(
                "test,statistic,df,p_value,cramers_v,yates_corrected,warning_low_expected,alpha,reject_null"
            );
            let v_str = r.cramers_v.map(|v| format!("{v:.10}")).unwrap_or_default();
            println!(
                "{},{:.10},{:.0},{:.10},{},{},{},{:.4},{}",
                r.test,
                r.statistic,
                r.df,
                r.p_value,
                v_str,
                r.yates_corrected,
                r.warning_low_expected,
                r.alpha,
                r.reject_null
            );
        }
        OutputFormat::Table => {
            let title = if r.test == "chi-square-independence" {
                "Ki-Kare Bağımsızlık Testi (Chi-Square Test of Independence)"
            } else {
                "Ki-Kare Uyum İyiliği Testi (Chi-Square Goodness of Fit Test)"
            };
            println!("{title}\n");

            // Frekanslar Tablosu
            let mut grp = Table::new();
            grp.load_preset(UTF8_BORDERS_ONLY);
            grp.set_header(vec![
                "Kategori / Hücre",
                "Gözlenen (Observed)",
                "Beklenen (Expected)",
            ]);

            if r.test == "chi-square-goodness-of-fit" {
                for (idx, (&o, &e)) in r.observed[0].iter().zip(r.expected[0].iter()).enumerate() {
                    grp.add_row(vec![
                        format!("Kategori {}", idx + 1),
                        format!("{:.1}", o),
                        format!("{:.4}", e),
                    ]);
                }
            } else {
                for i in 0..r.observed.len() {
                    for j in 0..r.observed[i].len() {
                        grp.add_row(vec![
                            format!("Satır {}, Sütun {}", i + 1, j + 1),
                            format!("{:.1}", r.observed[i][j]),
                            format!("{:.4}", r.expected[i][j]),
                        ]);
                    }
                }
            }
            println!("{grp}\n");

            // Sonuç Tablosu
            let mut tbl = Table::new();
            tbl.load_preset(UTF8_BORDERS_ONLY);
            tbl.set_header(vec!["Metrik", "Değer"]);
            tbl.add_row(vec![
                "Ki-Kare İstatistiği (χ²)".to_string(),
                format!("{:.6}", r.statistic),
            ]);
            tbl.add_row(vec![
                "Serbestlik Derecesi (df)".to_string(),
                format!("{:.0}", r.df),
            ]);
            tbl.add_row(vec!["p-değeri".to_string(), format!("{:.6e}", r.p_value)]);

            if let Some(v) = r.cramers_v {
                tbl.add_row(vec!["Cramér's V".to_string(), format!("{:.6}", v)]);
            }

            tbl.add_row(vec![
                "Yates Düzeltmesi Uygulandı".to_string(),
                (if r.yates_corrected { "Evet" } else { "Hayır" }).to_string(),
            ]);

            let karar = if r.reject_null {
                format!("REDDEDİLDİ (α={:.2})", r.alpha)
            } else {
                format!("REDDEDİLEMEDİ (α={:.2})", r.alpha)
            };
            tbl.add_row(vec!["H0 Hipotezi".to_string(), karar]);

            println!("{tbl}");

            if r.warning_low_expected {
                println!("\nUYARI: Beklenen hücre değerlerinin bazısı 5.0'dan küçüktür.");
                println!("Test sonuçları bu veri seti için tam olarak güvenilir olmayabilir.");
            }
        }
    }
}
