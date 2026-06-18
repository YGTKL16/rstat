use crate::error::StatError;
use std::sync::atomic::{AtomicBool, Ordering};

pub static LICENSE_VERIFIED: AtomicBool = AtomicBool::new(false);

#[cfg(test)]
pub static ENV_MUTEX: std::sync::Mutex<()> = std::sync::Mutex::new(());

pub fn check_license() -> Result<(), StatError> {
    if LICENSE_VERIFIED.load(Ordering::Relaxed)
        || std::env::var("RSTAT_PRO")
            .map(|v| v == "1")
            .unwrap_or(false)
    {
        Ok(())
    } else {
        Err(StatError::LicenseRequired(
            "SPC ve Proses Yeterlilik modülleri için RSTAT_PRO=1 lisans anahtarı tanımlanmalıdır."
                .into(),
        ))
    }
}
