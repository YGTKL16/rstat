pub mod verify;

use std::env;
use std::fs;
use std::path::PathBuf;
pub use verify::{LicenseFile, LicenseVerifier, Licensee};

/// Helper to resolve the expected license file path.
pub fn get_license_path() -> Option<PathBuf> {
    if let Some(val) = env::var_os("RSTAT_LICENSE_FILE").filter(|v| !v.is_empty()) {
        return Some(PathBuf::from(val));
    }
    dirs::config_dir().map(|p| p.join("rstat").join("license.json"))
}

/// Attempts to load and verify the license file.
pub fn try_load_license() -> Option<LicenseFile> {
    let path = get_license_path()?;
    if !path.exists() {
        return None;
    }
    let content = fs::read_to_string(path).ok()?;
    let license: LicenseFile = serde_json::from_str(&content).ok()?;
    if LicenseVerifier::verify(&license).is_ok() {
        Some(license)
    } else {
        None
    }
}
