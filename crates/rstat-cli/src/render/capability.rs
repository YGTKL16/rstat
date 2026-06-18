use super::OutputFormat;
use comfy_table::{Table, presets::UTF8_BORDERS_ONLY};
use rstat_core::capability::CapabilityResult;

pub fn print_capability(r: &CapabilityResult, fmt: OutputFormat) {
    match fmt {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(r).unwrap());
        }
        OutputFormat::Csv => {
            println!(
                "cp,cpk,pp,ppk,ppm_lcl,ppm_usl,ppm_total,within_sigma,overall_sigma,mean,skewness,normality_warning"
            );
            println!(
                "{},{},{},{},{},{},{},{},{},{},{},{}",
                r.cp.map(|v| format!("{:.10}", v)).unwrap_or_default(),
                r.cpk,
                r.pp.map(|v| format!("{:.10}", v)).unwrap_or_default(),
                r.ppk,
                r.ppm_lcl,
                r.ppm_usl,
                r.ppm_total,
                r.within_sigma,
                r.overall_sigma,
                r.mean,
                r.skewness,
                r.normality_warning
            );
        }
        OutputFormat::Table => {
            println!("Proses Yeterlilik Analizi (Process Capability)\n");

            let mut tbl = Table::new();
            tbl.load_preset(UTF8_BORDERS_ONLY);
            tbl.set_header(vec!["Metrik", "Değer", "Açıklama"]);

            tbl.add_row(vec![
                "Ortalama (Mean)",
                &format!("{:.6}", r.mean),
                "Proses merkez eğilimi",
            ]);
            tbl.add_row(vec![
                "Grup İçi Standart Sapma (Within Sigma)",
                &format!("{:.6}", r.within_sigma),
                "R-bar / d2 ile tahmin edilen kısa dönem değişkenlik",
            ]);
            tbl.add_row(vec![
                "Genel Standart Sapma (Overall Sigma)",
                &format!("{:.6}", r.overall_sigma),
                "Örneklem standart sapması ile hesaplanan uzun dönem değişkenlik",
            ]);
            tbl.add_row(vec![
                "Çarpıklık (Skewness)",
                &format!("{:.6}", r.skewness),
                "Normallik varsayımı kontrolü",
            ]);

            if let Some(cp) = r.cp {
                tbl.add_row(vec![
                    "Cp",
                    &format!("{:.4}", cp),
                    "Kısa dönem potansiyel yeterlilik (Grup İçi varyasyon)",
                ]);
            }
            tbl.add_row(vec![
                "Cpk",
                &format!("{:.4}", r.cpk),
                "Kısa dönem gerçekleşen yeterlilik",
            ]);

            if let Some(pp) = r.pp {
                tbl.add_row(vec![
                    "Pp",
                    &format!("{:.4}", pp),
                    "Uzun dönem potansiyel performans (Genel varyasyon)",
                ]);
            }
            tbl.add_row(vec![
                "Ppk",
                &format!("{:.4}", r.ppk),
                "Uzun dönem gerçekleşen performans",
            ]);

            tbl.add_row(vec![
                "PPM < LSL (Milyonda Hata)",
                &format!("{:.2}", r.ppm_lcl),
                "LSL altındaki beklenen hatalı parça sayısı",
            ]);
            tbl.add_row(vec![
                "PPM > USL (Milyonda Hata)",
                &format!("{:.2}", r.ppm_usl),
                "USL üstündeki beklenen hatalı parça sayısı",
            ]);
            tbl.add_row(vec![
                "Toplam PPM (Milyonda Hata)",
                &format!("{:.2}", r.ppm_total),
                "Toplam beklenen hatalı parça sayısı",
            ]);

            println!("{tbl}");

            println!("\nDeğerlendirme:");
            if r.cpk >= 1.33 {
                println!("  - Süreç yeterli seviyededir (Cpk >= 1.33).");
            } else {
                println!(
                    "  - Süreç yetersiz seviyededir (Cpk < 1.33). Değişkenlik düşürülmeli veya sınırlar incelenmeli."
                );
            }

            if r.normality_warning {
                println!(
                    "\n[UYARI] Verinin çarpıklığı (skewness = {:.3}) yüksek (> 1.0).",
                    r.skewness
                );
                println!(
                    "        Normal dağılım varsayımı geçerli olmayabilir. PPM tahminleri yanıltıcı olabilir."
                );
            }
        }
    }
}
