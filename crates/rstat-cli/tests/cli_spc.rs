use assert_cmd::Command;
use predicates::prelude::*;

fn cmd() -> Command {
    let mut c = Command::cargo_bin("rstat-cli").unwrap();
    c.env("RSTAT_PRO", "1");
    c
}

#[test]
fn test_spc_without_license_fails() {
    let mut c = Command::cargo_bin("rstat-cli").unwrap();
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
}

#[test]
fn test_spc_json() {
    let out = cmd()
        .args([
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
        .get_output()
        .stdout
        .clone();

    let v: serde_json::Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["subgroup_size"], 5);

    let gm = v["grand_mean"].as_f64().unwrap();
    assert!((gm - 10.790).abs() < 1e-3, "grand_mean={}", gm);

    let mr = v["mean_range"].as_f64().unwrap();
    assert!((mr - 4.2).abs() < 1e-3, "mean_range={}", mr);

    let sigma = v["estimated_sigma"].as_f64().unwrap();
    assert!((sigma - 1.80567).abs() < 1e-5, "estimated_sigma={}", sigma);
}

#[test]
fn test_spc_csv() {
    cmd()
        .args([
            "spc",
            "tests/fixtures/spc_data.csv",
            "--col",
            "value",
            "--subgroup-size",
            "5",
            "--format",
            "csv",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "subgroup_index,mean,range,xbar_lcl,xbar_cl,xbar_ucl,r_lcl,r_cl,r_ucl",
        ))
        .stdout(predicate::str::contains("0,10.06,"));
}

#[test]
fn test_spc_table() {
    cmd()
        .args([
            "spc",
            "tests/fixtures/spc_data.csv",
            "--col",
            "value",
            "--subgroup-size",
            "5",
            "--format",
            "table",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("İstatistiki Süreç Kontrolü"))
        .stdout(predicate::str::contains("X-bar (Ortalama) Grafiği"))
        .stdout(predicate::str::contains("Alt Grup Boyutu"))
        .stdout(predicate::str::contains("Tahmini Sigma"));
}

#[test]
fn test_spc_invalid_subgroup_fails() {
    cmd()
        .args([
            "spc",
            "tests/fixtures/spc_data.csv",
            "--col",
            "value",
            "--subgroup-size",
            "12",
        ])
        .assert()
        .failure();
}
