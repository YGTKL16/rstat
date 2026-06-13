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
    /// t-testi (one-sample / two-sample / paired)
    Ttest(TtestArgs),
}

#[derive(Parser)]
pub struct TtestArgs {
    /// Girdi CSV dosyası (verilmezse stdin)
    pub file: Option<std::path::PathBuf>,

    /// Test türü: one, two, paired
    #[arg(long, value_parser = ["one", "two", "paired"])]
    pub kind: String,

    /// Birinci kolon (ad veya 0-tabanlı indeks)
    #[arg(long, default_value = "0")]
    pub col: String,

    /// İkinci kolon (two/paired için zorunlu)
    #[arg(long)]
    pub col2: Option<String>,

    /// one-sample için H0 ortalaması
    #[arg(long, default_value = "0.0")]
    pub mu: f64,

    /// Varyans varsayımı (yalnızca two): welch (varsayılan) | pooled
    #[arg(long, default_value = "welch", value_parser = ["welch", "pooled"])]
    pub var: String,

    /// Alternatif hipotez
    #[arg(long, default_value = "two-sided", value_parser = ["two-sided", "less", "greater"])]
    pub alt: String,

    /// Güven düzeyi (CI için)
    #[arg(long, default_value = "0.95")]
    pub ci_level: f64,

    /// Anlamlılık düzeyi
    #[arg(long, default_value = "0.05")]
    pub alpha: f64,

    /// Çıktı formatı: table, json, csv
    #[arg(long)]
    pub format: Option<String>,
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
