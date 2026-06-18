use super::OutputFormat;
use comfy_table::{Table, presets::UTF8_BORDERS_ONLY};
use rstat_core::spc::SpcResult;

pub fn print_spc(r: &SpcResult, fmt: OutputFormat) {
    match fmt {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(r).unwrap());
        }
        OutputFormat::Csv => {
            println!("subgroup_index,mean,range,xbar_lcl,xbar_cl,xbar_ucl,r_lcl,r_cl,r_ucl");
            for s in &r.subgroups {
                println!(
                    "{},{},{},{},{},{},{},{},{}",
                    s.index,
                    s.mean,
                    s.range,
                    r.xbar_limits.lcl,
                    r.xbar_limits.cl,
                    r.xbar_limits.ucl,
                    r.r_limits.lcl,
                    r.r_limits.cl,
                    r.r_limits.ucl
                );
            }
        }
        OutputFormat::Table => {
            println!("İstatistiki Süreç Kontrolü (SPC) X-bar & R Analizi\n");

            // Limits table
            let mut lim = Table::new();
            lim.load_preset(UTF8_BORDERS_ONLY);
            lim.set_header(vec!["Metrik", "LCL", "CL (Merkez)", "UCL"]);
            lim.add_row(vec![
                "X-bar (Ortalama) Grafiği",
                &format!("{:.6}", r.xbar_limits.lcl),
                &format!("{:.6}", r.xbar_limits.cl),
                &format!("{:.6}", r.xbar_limits.ucl),
            ]);
            lim.add_row(vec![
                "R (Genişlik) Grafiği",
                &format!("{:.6}", r.r_limits.lcl),
                &format!("{:.6}", r.r_limits.cl),
                &format!("{:.6}", r.r_limits.ucl),
            ]);
            println!("{lim}\n");

            println!("Parametreler:");
            println!("  Alt Grup Boyutu: {}", r.subgroup_size);
            println!("  Alt Grup Sayısı: {}", r.subgroups.len());
            println!("  Tahmini Sigma (R-bar / d2): {:.6}\n", r.estimated_sigma);

            // Subgroups and ASCII Chart
            let mut tbl = Table::new();
            tbl.load_preset(UTF8_BORDERS_ONLY);
            tbl.set_header(vec![
                "Grup",
                "Ortalama",
                "Genişlik",
                "X-bar ASCII Grafik (L=LCL, C=CL, U=UCL, *=Normal, !=İhlal)",
            ]);

            for s in &r.subgroups {
                let mut chars = vec![' '; 41];
                chars[10..20].fill('-');
                chars[21..30].fill('-');
                chars[10] = 'L';
                chars[20] = 'C';
                chars[30] = 'U';

                let lcl = r.xbar_limits.lcl;
                let ucl = r.xbar_limits.ucl;
                let idx = if ucl > lcl {
                    let pct = (s.mean - lcl) / (ucl - lcl);
                    let pos = 10.0 + pct * 20.0;
                    pos.round() as isize
                } else {
                    20
                };
                let idx = idx.clamp(0, 40) as usize;
                let point_char = if s.mean < lcl || s.mean > ucl {
                    '!'
                } else {
                    '*'
                };
                chars[idx] = point_char;
                let bar: String = chars.into_iter().collect();

                tbl.add_row(vec![
                    s.index.to_string(),
                    format!("{:.6}", s.mean),
                    format!("{:.6}", s.range),
                    bar,
                ]);
            }
            println!("{tbl}");

            // Violations check
            if !r.violations.is_empty() {
                println!("\nİhlaller (Western Electric Kuralları):");
                for v in &r.violations {
                    println!("  - Alt Grup {}: {}", v.subgroup_index, v.description);
                }
            } else {
                println!(
                    "\nSüreç kontrol altında: Herhangi bir Western Electric kural ihlali tespit edilmedi."
                );
            }
        }
    }
}
