use anyhow::{Context, Result, bail};
use rstat_core::data::summary::summary;
use crate::cli::SummaryArgs;
use crate::render::{OutputFormat, print_summary};

pub fn run_summary(args: SummaryArgs) -> Result<()> {
    let data = read_column(&args.file, &args.col)?;
    let stats = summary(&data).context("özet istatistik hesaplanamadı")?;
    let fmt = OutputFormat::detect(args.format.as_deref());
    print_summary(&stats, fmt);
    Ok(())
}

fn read_column(file: &Option<std::path::PathBuf>, col: &str) -> Result<Vec<f64>> {
    let reader: Box<dyn std::io::Read> = match file {
        Some(path) => Box::new(
            std::fs::File::open(path)
                .with_context(|| format!("dosya açılamadı: {}", path.display()))?,
        ),
        None => Box::new(std::io::stdin()),
    };

    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .from_reader(reader);

    // Kolon indeksini belirle (isim veya sayı)
    let col_idx = if let Ok(n) = col.parse::<usize>() {
        n
    } else {
        let headers = rdr.headers()?.clone();
        headers
            .iter()
            .position(|h| h == col)
            .with_context(|| format!("kolon bulunamadı: '{col}'"))?
    };

    let mut values: Vec<f64> = Vec::new();
    for (i, result) in rdr.records().enumerate() {
        let record = result.with_context(|| format!("satır {i} okunamadı"))?;
        let cell = record.get(col_idx)
            .with_context(|| format!("satır {i}'de kolon {col_idx} yok"))?
            .trim();
        if cell.is_empty() || cell.eq_ignore_ascii_case("na") || cell.eq_ignore_ascii_case("nan") {
            continue; // NA değerleri atla
        }
        let v: f64 = cell.parse()
            .with_context(|| format!("satır {i}, '{cell}' sayıya dönüştürülemedi"))?;
        if v.is_nan() {
            bail!("satır {i}: NaN değer");
        }
        values.push(v);
    }

    if values.is_empty() {
        bail!("kolon boş veya tüm değerler NA");
    }

    Ok(values)
}
