use thiserror::Error;

#[derive(Debug, Error)]
pub enum StatError {
    #[error("veri boş")]
    EmptyData,

    #[error("yetersiz veri: en az {required} değer gerekli, {got} verildi")]
    InsufficientData { required: usize, got: usize },

    #[error("kolon uzunlukları eşit değil: {a} ve {b}")]
    LengthMismatch { a: usize, b: usize },

    #[error("geçersiz parametre: {0}")]
    InvalidParameter(String),

    #[error("sayısal hata: {0}")]
    Numerical(String),
}
