use assert_cmd::Command;
use predicates::prelude::*;

fn cmd() -> Command {
    let mut c = Command::cargo_bin("rstat-cli").unwrap();
    c.env("RSTAT_PRO", "1");
    c
}

#[test]
fn test_capability_without_license_fails() {
    let mut c = Command::cargo_bin("rstat-cli").unwrap();
    c.args([
        "capability",
        "tests/fixtures/capability_data.csv",
        "--col",
        "value",
        "--subgroup-size",
        "3",
        "--lsl",
        "1.8",
        "--usl",
        "3.0",
        "--format",
        "json",
    ])
    .assert()
    .failure()
    .stderr(predicate::str::contains("lisans gerekli"));
}

#[test]
fn test_capability_json() {
    let out = cmd()
        .args([
            "capability",
            "tests/fixtures/capability_data.csv",
            "--col",
            "value",
            "--subgroup-size",
            "3",
            "--lsl",
            "1.8",
            "--usl",
            "3.0",
            "--format",
            "json",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: serde_json::Value = serde_json::from_slice(&out).unwrap();

    let m = v["mean"].as_f64().unwrap();
    assert!((m - 2.366667).abs() < 1e-5, "mean={}", m);

    let os = v["overall_sigma"].as_f64().unwrap();
    assert!((os - 0.222539).abs() < 1e-5, "overall_sigma={}", os);

    let ws = v["within_sigma"].as_f64().unwrap();
    assert!((ws - 0.177200).abs() < 1e-5, "within_sigma={}", ws);

    let cp = v["cp"].as_f64().unwrap();
    assert!((cp - 1.128667).abs() < 1e-5, "cp={}", cp);

    let cpk = v["cpk"].as_f64().unwrap();
    assert!((cpk - 1.065963).abs() < 1e-5, "cpk={}", cpk);

    let pp = v["pp"].as_f64().unwrap();
    assert!((pp - 0.898717).abs() < 1e-5, "pp={}", pp);

    let ppk = v["ppk"].as_f64().unwrap();
    assert!((ppk - 0.848788).abs() < 1e-5, "ppk={}", ppk);

    let ppm_lcl = v["ppm_lcl"].as_f64().unwrap();
    assert!((ppm_lcl - 5442.566).abs() < 1e-1, "ppm_lcl={}", ppm_lcl);

    let ppm_usl = v["ppm_usl"].as_f64().unwrap();
    assert!((ppm_usl - 2214.045).abs() < 1e-1, "ppm_usl={}", ppm_usl);

    let ppm_total = v["ppm_total"].as_f64().unwrap();
    assert!(
        (ppm_total - 7656.612).abs() < 1e-1,
        "ppm_total={}",
        ppm_total
    );

    let skew = v["skewness"].as_f64().unwrap();
    assert!((skew - 0.113301).abs() < 1e-5, "skewness={}", skew);

    assert_eq!(v["normality_warning"], false);
}

#[test]
fn test_capability_csv() {
    cmd()
        .args([
            "capability",
            "tests/fixtures/capability_data.csv",
            "--col", "value",
            "--subgroup-size", "3",
            "--lsl", "1.8",
            "--usl", "3.0",
            "--format", "csv",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("cp,cpk,pp,ppk,ppm_lcl,ppm_usl,ppm_total,within_sigma,overall_sigma,mean,skewness,normality_warning"))
        .stdout(predicate::str::contains("1.128666"));
}

#[test]
fn test_capability_table() {
    cmd()
        .args([
            "capability",
            "tests/fixtures/capability_data.csv",
            "--col",
            "value",
            "--subgroup-size",
            "3",
            "--lsl",
            "1.8",
            "--usl",
            "3.0",
            "--format",
            "table",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Proses Yeterlilik Analizi"))
        .stdout(predicate::str::contains("Grup İçi Standart Sapma"))
        .stdout(predicate::str::contains("Cpk"))
        .stdout(predicate::str::contains("Ppk"));
}

#[test]
fn test_capability_missing_limits_fails() {
    cmd()
        .args([
            "capability",
            "tests/fixtures/capability_data.csv",
            "--col",
            "value",
            "--subgroup-size",
            "3",
        ])
        .assert()
        .failure();
}
