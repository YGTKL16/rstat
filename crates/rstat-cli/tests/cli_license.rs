use assert_cmd::Command;
use predicates::prelude::*;
use std::env;
use std::fs;

const VALID_LICENSE_JSON: &str = r#"{
  "licensee": {
    "name": "CLI Test User",
    "email": "cli_test@example.com",
    "order_id": "CLI-ORD-123"
  },
  "tier": "pro",
  "features": [
    "spc",
    "capability"
  ],
  "signature": "W3N39oKmvucuHaPH0/CYceOi8mmDc6a2EMt8LnhBuyUvzgo2e6GQGEmnaAdBMePEu8/uCUNdc8a4UhgnOMt1Cg=="
}"#;

#[test]
fn test_spc_with_valid_license_file_succeeds() {
    // 1. Create a temporary license file
    let mut temp_path = env::temp_dir();
    temp_path.push("rstat_license_test_spc.json");
    fs::write(&temp_path, VALID_LICENSE_JSON).unwrap();

    // 2. Run SPC command without RSTAT_PRO but with RSTAT_LICENSE_FILE
    let mut c = Command::cargo_bin("rstat-cli").unwrap();
    c.env_remove("RSTAT_PRO");
    c.env("RSTAT_LICENSE_FILE", &temp_path);
    c.args([
        "spc",
        "tests/fixtures/spc_data.csv",
        "--col",
        "value",
        "--subgroup-size",
        "5",
        "--format",
        "json",
    ])
    .assert()
    .success()
    .stdout(predicate::str::contains("grand_mean"));

    // Clean up
    let _ = fs::remove_file(temp_path);
}

#[test]
fn test_capability_with_valid_license_file_succeeds() {
    // 1. Create a temporary license file
    let mut temp_path = env::temp_dir();
    temp_path.push("rstat_license_test_cap.json");
    fs::write(&temp_path, VALID_LICENSE_JSON).unwrap();

    // 2. Run Capability command without RSTAT_PRO but with RSTAT_LICENSE_FILE
    let mut c = Command::cargo_bin("rstat-cli").unwrap();
    c.env_remove("RSTAT_PRO");
    c.env("RSTAT_LICENSE_FILE", &temp_path);
    c.args([
        "capability",
        "tests/fixtures/spc_data.csv",
        "--col",
        "value",
        "--lsl",
        "5.0",
        "--usl",
        "15.0",
        "--format",
        "json",
    ])
    .assert()
    .success()
    .stdout(predicate::str::contains("cp"));

    // Clean up
    let _ = fs::remove_file(temp_path);
}

#[test]
fn test_spc_with_invalid_license_file_fails() {
    // 1. Create a mutated/invalid license file
    let mut temp_path = env::temp_dir();
    temp_path.push("rstat_license_test_invalid.json");
    let invalid_json = VALID_LICENSE_JSON.replace("CLI Test User", "CLI Test User Mutated");
    fs::write(&temp_path, invalid_json).unwrap();

    // 2. Run SPC command without RSTAT_PRO but with RSTAT_LICENSE_FILE pointing to invalid license
    let mut c = Command::cargo_bin("rstat-cli").unwrap();
    c.env_remove("RSTAT_PRO");
    c.env("RSTAT_LICENSE_FILE", &temp_path);
    c.args([
        "spc",
        "tests/fixtures/spc_data.csv",
        "--col",
        "value",
        "--subgroup-size",
        "5",
        "--format",
        "json",
    ])
    .assert()
    .failure()
    .stderr(predicate::str::contains("lisans gerekli"));

    // Clean up
    let _ = fs::remove_file(temp_path);
}
