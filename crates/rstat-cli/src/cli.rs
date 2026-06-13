use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "rstat", version, about = "Pipeline-friendly istatistik CLI")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Özet istatistikler (n, mean, std, min, Q1, median, Q3, max)
    Summary(SummaryArgs),
}

#[derive(Parser)]
pub struct SummaryArgs {
    /// Girdi CSV dosyası (verilmezse stdin)
    pub file: Option<std::path::PathBuf>,

    /// Kullanılacak kolon adı veya indeksi (0-tabanlı)
    #[arg(long, default_value = "0")]
    pub col: String,

    /// Çıktı formatı: table, json, csv
    #[arg(long)]
    pub format: Option<String>,
}
