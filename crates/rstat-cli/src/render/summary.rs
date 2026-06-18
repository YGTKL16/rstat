use super::OutputFormat;
use comfy_table::{Table, presets::UTF8_BORDERS_ONLY};
use rstat_core::result::SummaryStats;

pub fn print_summary(stats: &SummaryStats, fmt: OutputFormat) {
    match fmt {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(stats).unwrap());
        }
        OutputFormat::Csv => {
            println!("n,mean,std,min,q1,median,q3,max");
            println!(
                "{},{},{},{},{},{},{},{}",
                stats.n,
                stats.mean,
                stats.std,
                stats.min,
                stats.q1,
                stats.median,
                stats.q3,
                stats.max
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
